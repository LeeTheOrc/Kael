use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MessageRole {
    #[default]
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RequestType {
    #[default]
    Chat,
    Schedule,
    Email,
    Code,
    Vision,
    Install,
    Search,
    FileOperation,
    System,
}

impl RequestType {
    pub fn from_message(message: &str) -> Self {
        let msg_lower = message.to_lowercase();
        
        if msg_lower.contains("image") || msg_lower.contains("picture") || msg_lower.contains("photo") {
            return RequestType::Vision;
        }
        if msg_lower.contains("install") || msg_lower.contains("download") {
            return RequestType::Install;
        }
        if msg_lower.contains("schedule") || msg_lower.contains("calendar") || msg_lower.contains("meeting") {
            return RequestType::Schedule;
        }
        if msg_lower.contains("email") || msg_lower.contains("mail") || msg_lower.contains("send") {
            return RequestType::Email;
        }
        if msg_lower.contains("code") || msg_lower.contains("program") || msg_lower.contains("debug") {
            return RequestType::Code;
        }
        if msg_lower.contains("search") || msg_lower.contains("what is") || msg_lower.contains("how to") || msg_lower.contains("?") {
            return RequestType::Search;
        }
        
        RequestType::Chat
    }
    
    pub fn target_ai(&self) -> &str {
        match self {
            RequestType::Chat => "Director",
            RequestType::Schedule | RequestType::Email => "Director-PA",
            RequestType::Code => "Programmer",
            RequestType::Vision => "Vision",
            RequestType::Install => "Director-Install",
            RequestType::Search => "Director-Search",
            _ => "Director",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_type: Option<RequestType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AiMode {
    #[default]
    Director,
    Programmer,
    Vision,
}

impl AiMode {
    fn display_name(&self) -> &str {
        match self {
            AiMode::Director => "Director / PA",
            AiMode::Programmer => "Programmer",
            AiMode::Vision => "Vision",
        }
    }
    
    fn icon(&self) -> &str {
        match self {
            AiMode::Director => "🎯",
            AiMode::Programmer => "💻",
            AiMode::Vision => "👁️",
        }
    }
    
    fn model_name(&self) -> &str {
        match self {
            AiMode::Director => "dolphin3.0-mistral-7b-q4",
            AiMode::Programmer => "dolphin3.0-coder-7b-q4",
            AiMode::Vision => "llava-7b-q4",
        }
    }
}

pub struct KaelApp {
    messages: Vec<ChatMessage>,
    input_text: String,
    current_ai: AiMode,
    is_loading: bool,
    chat_history: VecDeque<(AiMode, Vec<ChatMessage>)>,
    ollama_available: bool,
    runtime: Runtime,
    ollama_url: String,
}

impl KaelApp {
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        
        let ollama_url = "http://localhost:11434".to_string();
        let ollama_available = runtime.block_on(check_ollama(&ollama_url));
        
        let mut app = Self {
            messages: Vec::new(),
            input_text: String::new(),
            current_ai: AiMode::Director,
            is_loading: false,
            chat_history: VecDeque::new(),
            ollama_available,
            runtime,
            ollama_url,
        };
        
        let welcome_msg = if app.ollama_available {
            "Hello! I'm Kael, your AI assistant.\n\nI can help you with:\n• General conversation and tasks\n• Programming and code help\n• Image analysis\n• Internet search\n• Installing applications\n\nAll messages go through the Director AI first, which routes them to the appropriate sub-AI. Just send me a message!"
        } else {
            "Hello! I'm Kael, your AI assistant.\n\n⚠️ Ollama not detected - running in demo mode.\n\nTo enable real AI:\n1. Install Ollama: https://ollama.ai\n2. Run: ollama pull dolphin3.0\n3. Restart Kael\n\nFor now, I can show you the interface!"
        };
        
        app.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: welcome_msg.to_string(),
            timestamp: chrono::Utc::now(),
            request_type: Some(RequestType::Chat),
        });
        
        app
    }
    
    fn switch_ai(&mut self, ai: AiMode) {
        if self.current_ai != ai {
            self.chat_history.push_front((self.current_ai, std::mem::take(&mut self.messages)));
            self.messages = self.chat_history
                .iter()
                .find(|(mode, _)| *mode == ai)
                .map(|(_, msgs)| msgs.clone())
                .unwrap_or_else(Vec::new);
            
            if self.messages.is_empty() {
                let welcome = match ai {
                    AiMode::Director => "You are now chatting with Director. I'm here to help with general tasks, questions, and I can also search the internet, install apps, and manage your schedule/email.",
                    AiMode::Programmer => "You are now chatting with Programmer. I'm here to help with coding, debugging, and technical questions.",
                    AiMode::Vision => "You are now chatting with Vision. Send me images to analyze them.",
                };
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: welcome.to_string(),
                    timestamp: chrono::Utc::now(),
                    request_type: Some(RequestType::Chat),
                });
            }
            
            self.current_ai = ai;
        }
    }
    
    fn send_message(&mut self) {
        let content = self.input_text.trim();
        if content.is_empty() {
            return;
        }
        
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
            request_type: None,
        });
        
        self.input_text.clear();
        self.is_loading = true;
    }
    
    fn generate_response(&mut self) {
        let last_message = self.messages.last().map(|m| m.content.clone()).unwrap_or_default();
        let request_type = RequestType::from_message(&last_message);
        
        // Build messages for Ollama
        let messages_json: Vec<serde_json::Value> = self.messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant", 
                        MessageRole::System => "system",
                    },
                    "content": m.content
                })
            })
            .collect();
        
        let response = if self.ollama_available {
            self.call_ollama(&messages_json, self.current_ai.model_name())
        } else {
            self.demo_response(&last_message, request_type)
        };
        
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: response,
            timestamp: chrono::Utc::now(),
            request_type: Some(request_type),
        });
        
        self.is_loading = false;
    }
    
    fn call_ollama(&self, messages: &[serde_json::Value], model: &str) -> String {
        #[derive(Serialize)]
        struct Request<'a> {
            model: &'a str,
            messages: &'a [serde_json::Value],
            stream: bool,
        }
        
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap();
        
        let request = Request {
            model,
            messages,
            stream: false,
        };
        
        match client.post(format!("{}/api/chat", self.ollama_url))
            .json(&request)
            .send()
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    #[derive(Deserialize)]
                    struct Response {
                        message: MessageContent,
                    }
                    #[derive(Deserialize)]
                    struct MessageContent {
                        content: String,
                    }
                    
                    match resp.json::<Response>() {
                        Ok(r) => r.message.content,
                        Err(e) => format!("Failed to parse response: {}", e),
                    }
                } else {
                    format!("Ollama error: {}", resp.status())
                }
            }
            Err(e) => format!("Connection error: {}", e),
        }
    }
    
    fn demo_response(&self, message: &str, request_type: RequestType) -> String {
        match request_type {
            RequestType::Search => {
                format!(
                    "🔍 **Search Request Detected**\n\nI'd search the internet for: \"{}\"\n\nThis feature requires Ollama to be running with internet access.",
                    message
                )
            }
            RequestType::Install => {
                format!(
                    "📦 **Install Request Detected**\n\nI'd help you install an application.\n\nFrom your message, it seems you want to install something. Here's how it would work:\n\n1. **Programmer AI** would determine what you need\n2. **Director** would check your package manager\n3. I'd show you the command and ask for confirmation\n4. Then run it in the terminal\n\n**Demo mode** - Ollama not running"
                )
            }
            RequestType::Schedule | RequestType::Email => {
                "📅 **Schedule/Email Request Detected**\n\nI'd help you with calendar events, emails, and reminders.\n\nThis uses the **PA Protocol**.\n\n**Demo mode**".to_string()
            }
            RequestType::Code => {
                format!(
                    "💻 **Code Request Detected**\n\nRouting to **Programmer AI**...\n\n```rust\n// Example\nfn help() {{\n    // Code help here\n}}\n\n// Your question: {}\n\n// Demo mode",
                    message.chars().take(50).collect::<String>()
                )
            }
            RequestType::Vision => {
                "👁️ **Vision Request Detected**\n\nI'd analyze any images you upload.\n\nTo use Vision, click the 📎 button and select an image.\n\n**Demo mode**".to_string()
            }
            _ => {
                format!(
                    "💬 **Chat Message**\n\nReceived: \"{}\"\n\n**Request Type:** {:?}\n**Routing to:** {}\n\nAll messages go through the **Director AI** first, which then routes to the appropriate sub-AI.\n\n**Demo mode** - Ollama not running",
                    message.chars().take(100).collect::<String>(),
                    request_type,
                    request_type.target_ai()
                )
            }
        }
    }
}

async fn check_ollama(url: &str) -> bool {
    match reqwest::get(format!("{}/api/tags", url)).await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

impl eframe::App for KaelApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_loading {
            self.generate_response();
            ctx.request_repaint();
        }
        
        egui::SidePanel::left("sidebar")
            .default_width(220.0)
            .show(ctx, |ui| {
                ui.heading("🤖 Kael");
                ui.separator();
                
                ui.label(egui::RichText::new("Status:").color(egui::Color32::GRAY));
                if self.ollama_available {
                    ui.label(egui::RichText::new("🟢 Ollama Connected").color(egui::Color32::from_rgb(16, 185, 129)));
                } else {
                    ui.label(egui::RichText::new("🔴 Demo Mode").color(egui::Color32::from_rgb(239, 68, 68)));
                }
                
                ui.separator();
                ui.label("Select AI:");
                
                let ai_buttons = [
                    (AiMode::Director, "Director / PA"),
                    (AiMode::Programmer, "Programmer"),
                    (AiMode::Vision, "Vision"),
                ];
                
                for (ai, name) in ai_buttons {
                    let is_selected = self.current_ai == ai;
                    let button_text = format!("{} {}", ai.icon(), name);
                    
                    if ui.selectable_label(is_selected, button_text).clicked() {
                        self.switch_ai(ai);
                    }
                }
                
                ui.separator();
                ui.label(egui::RichText::new("AI Flow:").color(egui::Color32::GRAY));
                ui.label(egui::RichText::new("All → Director → Sub-AI").small());
                
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.separator();
                    ui.label(egui::RichText::new("v0.2.0").color(egui::Color32::GRAY));
                });
            });
        
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for message in &self.messages {
                            self.render_message(ui, message);
                        }
                    });
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    let text_edit = egui::TextEdit::singleline(&mut self.input_text)
                        .hint_text(format!("Message {}...", self.current_ai.display_name()))
                        .desired_width(ui.available_width() - 120.0);
                    
                    ui.add(text_edit);
                    
                    if ui.button("📎").clicked() {}
                    
                    if ui.add_enabled(!self.is_loading, egui::Button::new("Send ➤"))
                        .clicked() 
                    {
                        self.send_message();
                    }
                });
                
                ui.add_space(5.0);
                
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Status:").color(egui::Color32::GRAY));
                    if self.is_loading {
                        ui.spinner();
                        ui.label(egui::RichText::new("Processing...").color(egui::Color32::from_rgb(251, 191, 36)));
                    } else if self.ollama_available {
                        ui.label(egui::RichText::new("Ready").color(egui::Color32::from_rgb(16, 185, 129)));
                    } else {
                        ui.label(egui::RichText::new("Demo Mode").color(egui::Color32::from_rgb(251, 191, 36)));
                    }
                    
                    ui.separator();
                    
                    if ui.button("Clear 🗑️").clicked() {
                        self.messages.clear();
                        self.messages.push(ChatMessage {
                            role: MessageRole::Assistant,
                            content: "Chat cleared. How can I help you?".to_string(),
                            timestamp: chrono::Utc::now(),
                            request_type: Some(RequestType::Chat),
                        });
                    }
                });
            });
    }
}

impl KaelApp {
    fn render_message(&self, ui: &mut egui::Ui, message: &ChatMessage) {
        let is_user = message.role == MessageRole::User;
        
        let align = if is_user { egui::Align::Max } else { egui::Align::Min };
        
        ui.with_layout(egui::Layout::right_to_left(align), |ui| {
            ui.add_space(10.0);
            
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let avatar = match message.role {
                        MessageRole::User => "👤",
                        MessageRole::Assistant => "🤖",
                        MessageRole::System => "⚙️",
                    };
                    ui.label(avatar);
                    
                    let name = if is_user { "You" } else { "Kael" };
                    let color = if is_user {
                        egui::Color32::from_rgb(16, 163, 127)
                    } else {
                        egui::Color32::from_rgb(139, 92, 246)
                    };
                    
                    ui.label(egui::RichText::new(name).color(color));
                    
                    if let Some(rt) = message.request_type {
                        if !is_user {
                            ui.separator();
                            ui.label(egui::RichText::new(format!("{:?}", rt))
                                .small()
                                .color(egui::Color32::GRAY));
                        }
                    }
                });
                
                ui.separator();
                ui.label(&message.content);
            });
        });
        
        ui.add_space(5.0);
    }
}
