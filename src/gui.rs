use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::runtime::Runtime;

use crate::ai::{Terminal, Vault};

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
    terminal: Terminal,
    sudo_set: bool,
    selected_image: Option<String>,
    vault: Option<Vault>,
}

impl KaelApp {
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        
        let ollama_url = "http://localhost:11434".to_string();
        let ollama_available = runtime.block_on(check_ollama(&ollama_url));
        
        // Initialize vault (RAG + LoRA + Chat history)
        let vault = match Vault::new() {
            Ok(v) => {
                println!("Vault initialized successfully");
                Some(v)
            }
            Err(e) => {
                eprintln!("Failed to initialize vault: {}", e);
                None
            }
        };
        
        let mut app = Self {
            messages: Vec::new(),
            input_text: String::new(),
            current_ai: AiMode::Director,
            is_loading: false,
            chat_history: VecDeque::new(),
            ollama_available,
            runtime,
            ollama_url,
            terminal: Terminal::new(),
            sudo_set: false,
            selected_image: None,
            vault,
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
        
        // Handle special commands
        if content.starts_with("/setsudo ") {
            let password = content.strip_prefix("/setsudo ").unwrap().trim();
            if password.is_empty() {
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: "Usage: /setsudo <password>\n\nThis sets your sudo password for terminal commands that require it.".to_string(),
                    timestamp: chrono::Utc::now(),
                    request_type: Some(RequestType::System),
                });
            } else {
                self.terminal.set_sudo_password(password.to_string());
                self.sudo_set = true;
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: "✅ Sudo password set! Terminal commands requiring sudo will now work automatically.".to_string(),
                    timestamp: chrono::Utc::now(),
                    request_type: Some(RequestType::System),
                });
            }
            self.input_text.clear();
            return;
        }
        
        if content == "/clearsudo" {
            self.terminal.clear_sudo_password();
            self.sudo_set = false;
            self.messages.push(ChatMessage {
                role: MessageRole::Assistant,
                content: "✅ Sudo password cleared for security.".to_string(),
                timestamp: chrono::Utc::now(),
                request_type: Some(RequestType::System),
            });
            self.input_text.clear();
            return;
        }
        
        if content == "/help" {
            let help = r#"Kael Commands:
/setsudo <password> - Set your sudo password for terminal commands
/clearsudo          - Clear stored sudo password (recommended after use)
/terminal <command>  - Run a terminal command directly
/learn <text>       - Teach Kael something (saves to knowledge base)
/recall <query>     - Search your saved knowledge
/stats             - Show database statistics
/history           - Show chat history

Examples:
/setsudo mypassword123
/terminal sudo pacman -S firefox
/learn The project is called Kael and it's written in Rust
/recall Kael project
"#;
            self.messages.push(ChatMessage {
                role: MessageRole::Assistant,
                content: help.to_string(),
                timestamp: chrono::Utc::now(),
                request_type: Some(RequestType::System),
            });
            self.input_text.clear();
            return;
        }
        
        if content.starts_with("/terminal ") {
            let command = content.strip_prefix("/terminal ").unwrap();
            let result = self.terminal.execute(command);
            
            let response = if result.success {
                format!("✅ Command executed successfully:\n\n{}", result.stdout)
            } else if result.needs_sudo {
                format!("⚠️ Sudo required but no password set.\n\nUse /setsudo <password> to set your sudo password.\n\nError: {}", result.stderr)
            } else {
                format!("❌ Command failed:\n\n{}", result.stderr)
            };
            
            self.messages.push(ChatMessage {
                role: MessageRole::Assistant,
                content: response,
                timestamp: chrono::Utc::now(),
                request_type: Some(RequestType::System),
            });
            self.input_text.clear();
            return;
        }
        
        if content.starts_with("/image ") {
            let path = content.strip_prefix("/image ").unwrap().trim();
            
            if path.is_empty() {
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: "Usage: /image /path/to/image.png\n\nExample: /image ~/Pictures/screenshot.png".to_string(),
                    timestamp: chrono::Utc::now(),
                    request_type: Some(RequestType::Vision),
                });
                self.input_text.clear();
                return;
            }
            
            // Check if file exists
            let path_obj = std::path::Path::new(path);
            if !path_obj.exists() {
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: format!("❌ File not found: {}\n\nPlease check the path and try again.", path),
                    timestamp: chrono::Utc::now(),
                    request_type: Some(RequestType::Vision),
                });
                self.input_text.clear();
                return;
            }
            
            // Check if it's a valid image
            let extension = path_obj.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();
            
            if !["png", "jpg", "jpeg", "gif", "bmp", "webp"].contains(&extension.as_str()) {
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: format!("Unsupported format: {}\n\nSupported: PNG, JPG, JPEG, GIF, BMP, WebP", extension),
                    timestamp: chrono::Utc::now(),
                    request_type: Some(RequestType::Vision),
                });
                self.input_text.clear();
                return;
            }
            
            self.selected_image = Some(path.to_string());
            
            // Get image info
            let file_name = path_obj.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            
            let file_size = std::fs::metadata(path)
                .map(|m| m.len())
                .unwrap_or(0);
            
            let size_str = if file_size > 1024 * 1024 {
                format!("{:.1} MB", file_size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.1} KB", file_size as f64 / 1024.0)
            };
            
            self.messages.push(ChatMessage {
                role: MessageRole::User,
                content: format!("[Image: {} ({})]\n\nWhat do you see in this image?", file_name, size_str),
                timestamp: chrono::Utc::now(),
                request_type: Some(RequestType::Vision),
            });
            
            self.input_text.clear();
            self.is_loading = true;
            return;
        }
        
        // Vault/RAG commands
        if content.starts_with("/learn ") || content.starts_with("/add ") {
            let what = content.strip_prefix("/learn ").or_else(|| content.strip_prefix("/add ")).unwrap().trim();
            
            if let Some(ref vault) = self.vault {
                match vault.add_knowledge(what, what, "user_input", "director") {
                    Ok(id) => {
                        self.messages.push(ChatMessage {
                            role: MessageRole::Assistant,
                            content: format!("✅ Learned (ID: {})", id),
                            timestamp: chrono::Utc::now(),
                            request_type: Some(RequestType::System),
                        });
                    }
                    Err(e) => {
                        self.messages.push(ChatMessage {
                            role: MessageRole::Assistant,
                            content: format!("❌ Failed to learn: {}", e),
                            timestamp: chrono::Utc::now(),
                            request_type: Some(RequestType::System),
                        });
                    }
                }
            } else {
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: "Vault not available. Database error.".to_string(),
                    timestamp: chrono::Utc::now(),
                    request_type: Some(RequestType::System),
                });
            }
            self.input_text.clear();
            return;
        }
        
        if content.starts_with("/search ") || content.starts_with("/recall ") {
            let query = content.strip_prefix("/search ").or_else(|| content.strip_prefix("/recall ")).unwrap().trim();
            
            if let Some(ref vault) = self.vault {
                match vault.search_knowledge(query, "director") {
                    Ok(docs) => {
                        if docs.is_empty() {
                            self.messages.push(ChatMessage {
                                role: MessageRole::Assistant,
                                content: format!("No results found for: \"{}\"", query),
                                timestamp: chrono::Utc::now(),
                                request_type: Some(RequestType::System),
                            });
                        } else {
                            let results: String = docs.iter().take(3).enumerate()
                                .map(|(i, d)| format!("{}. {}\n{}\n", i+1, d.title, &d.content.chars().take(200).collect::<String>()))
                                .collect();
                            self.messages.push(ChatMessage {
                                role: MessageRole::Assistant,
                                content: format!("Found {} results:\n\n{}", docs.len(), results),
                                timestamp: chrono::Utc::now(),
                                request_type: Some(RequestType::System),
                            });
                        }
                    }
                    Err(e) => {
                        self.messages.push(ChatMessage {
                            role: MessageRole::Assistant,
                            content: format!("❌ Search failed: {}", e),
                            timestamp: chrono::Utc::now(),
                            request_type: Some(RequestType::System),
                        });
                    }
                }
            }
            self.input_text.clear();
            return;
        }
        
        if content == "/stats" {
            if let Some(ref vault) = self.vault {
                match vault.get_stats() {
                    Ok(stats) => {
                        self.messages.push(ChatMessage {
                            role: MessageRole::Assistant,
                            content: format!("📊 Kael Stats:\n\n• Chat messages: {}\n• Knowledge documents: {}\n• LoRA configs: {}", 
                                stats.chat_messages, stats.rag_documents, stats.lora_configs),
                            timestamp: chrono::Utc::now(),
                            request_type: Some(RequestType::System),
                        });
                    }
                    Err(e) => {
                        self.messages.push(ChatMessage {
                            role: MessageRole::Assistant,
                            content: format!("❌ Stats error: {}", e),
                            timestamp: chrono::Utc::now(),
                            request_type: Some(RequestType::System),
                        });
                    }
                }
            } else {
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: "Vault not available".to_string(),
                    timestamp: chrono::Utc::now(),
                    request_type: Some(RequestType::System),
                });
            }
            self.input_text.clear();
            return;
        }
        
        if content == "/history" || content == "/memories" {
            if let Some(ref vault) = self.vault {
                match vault.get_chat_history("director", 10) {
                    Ok(history) => {
                        if history.is_empty() {
                            self.messages.push(ChatMessage {
                                role: MessageRole::Assistant,
                                content: "No chat history yet.".to_string(),
                                timestamp: chrono::Utc::now(),
                                request_type: Some(RequestType::System),
                            });
                        } else {
                            let mems: String = history.iter().take(5).map(|h| {
                                format!("{}: {}", h.role, &h.content.chars().take(100).collect::<String>())
                            }).collect::<Vec<_>>().join("\n");
                            self.messages.push(ChatMessage {
                                role: MessageRole::Assistant,
                                content: format!("Recent memories:\n\n{}", mems),
                                timestamp: chrono::Utc::now(),
                                request_type: Some(RequestType::System),
                            });
                        }
                    }
                    Err(e) => {
                        self.messages.push(ChatMessage {
                            role: MessageRole::Assistant,
                            content: format!("Error: {}", e),
                            timestamp: chrono::Utc::now(),
                            request_type: Some(RequestType::System),
                        });
                    }
                }
            }
            self.input_text.clear();
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
                let pm = self.terminal.check_package_manager();
                format!(
                    "📦 **Install Request Detected**\n\nI'd help you install an application.\n\nYour system uses: **{}**\n\nTo install, use terminal command:\n```\n/terminal sudo {} install <package>\n```\n\nSet your sudo password first:\n```\n/setsudo <your-password>\n```\n\n**Demo mode** - Ollama not running",
                    pm, pm
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
                    "💬 **Chat Message**\n\nReceived: \"{}\"\n\n**Request Type:** {:?}\n**Routing to:** {}\n\nAll messages go through the **Director AI** first, which then routes to the appropriate sub-AI.\n\n**Demo mode** - Ollama not running\n\n---\n\n**Terminal Commands:**\n• `/setsudo <password>` - Set sudo password\n• `/terminal <command>` - Run command directly\n• `/help` - Show all commands",
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
                ui.label(egui::RichText::new("Terminal:").color(egui::Color32::GRAY));
                if self.sudo_set {
                    ui.label(egui::RichText::new("🔑 Sudo Ready").color(egui::Color32::from_rgb(16, 185, 129)));
                } else {
                    ui.label(egui::RichText::new("⚠️ No Sudo").color(egui::Color32::from_rgb(251, 191, 36)));
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
                        .desired_width(ui.available_width() - 160.0);
                    
                    ui.add(text_edit);
                    
                    // Image button - shows status
                    let image_btn_text = if let Some(ref img) = self.selected_image {
                        let name = std::path::Path::new(img)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Image".to_string());
                        if name.len() > 15 {
                            format!("📷 {}", &name[..15])
                        } else {
                            format!("📷 {}", name)
                        }
                    } else {
                        "📷".to_string()
                    };
                    
                    if ui.button(image_btn_text).clicked() {
                        // Show file path input dialog
                        self.messages.push(ChatMessage {
                            role: MessageRole::Assistant,
                            content: "To add an image, drag and drop it onto this window, or send the image path using:\n\n`/image /path/to/image.png`\n\nSupported formats: PNG, JPG, JPEG, GIF, BMP".to_string(),
                            timestamp: chrono::Utc::now(),
                            request_type: Some(RequestType::Vision),
                        });
                    }
                    
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
