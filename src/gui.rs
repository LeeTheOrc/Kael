use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::runtime::Runtime;

use crate::ai::{AiTrainingSystem, LlamaEngine, ModelDownloader, Terminal, TrainingManager, Vault};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LeftPanel {
    #[default]
    Chat,
    Calendar,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RightPanel {
    #[default]
    Tasks,
    Projects,
}

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
    Terminal,
}

impl RequestType {
    pub fn from_message(message: &str) -> Self {
        let msg_lower = message.to_lowercase();
        
        if msg_lower.contains("terminal") || msg_lower.contains("console") || msg_lower.contains("command line") {
            return RequestType::Terminal;
        }
        if msg_lower.contains("image") || msg_lower.contains("picture") || msg_lower.contains("photo") || msg_lower.contains("screenshot") {
            return RequestType::Vision;
        }
        if msg_lower.contains("install") || msg_lower.contains("download") || msg_lower.contains("sudo") || msg_lower.contains("apt") || msg_lower.contains("pacman") {
            return RequestType::Install;
        }
        if msg_lower.contains("schedule") || msg_lower.contains("calendar") || msg_lower.contains("meeting") || msg_lower.contains("appointment") || msg_lower.contains("remind") || msg_lower.contains("event") || msg_lower.contains("tomorrow") || msg_lower.contains("today") || msg_lower.contains("next week") {
            return RequestType::Schedule;
        }
        if msg_lower.contains("email") || msg_lower.contains("mail") || msg_lower.contains("send") || msg_lower.contains("inbox") || msg_lower.contains("message") {
            return RequestType::Email;
        }
        if msg_lower.contains("code") || msg_lower.contains("program") || msg_lower.contains("function") || msg_lower.contains("debug") || msg_lower.contains("error") || msg_lower.contains("compile") || msg_lower.contains("script") || msg_lower.contains("rust") || msg_lower.contains("python") || msg_lower.contains("javascript") || msg_lower.contains("git") {
            return RequestType::Code;
        }
        if msg_lower.contains("search") || msg_lower.contains("find") || msg_lower.contains("look up") || msg_lower.contains("what is") || msg_lower.contains("who is") || msg_lower.contains("how to") || msg_lower.contains("?") {
            return RequestType::Search;
        }
        
        RequestType::Chat
    }
    
    pub fn target_ai(&self) -> &str {
        match self {
            RequestType::Chat => "Director",
            RequestType::Schedule => "Director (PA Protocol)",
            RequestType::Email => "Director (PA Protocol)", 
            RequestType::Code => "Programmer",
            RequestType::Vision => "Vision",
            RequestType::Install => "Director (Install Protocol)",
            RequestType::Search => "Director (Search)",
            RequestType::Terminal => "Terminal",
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
    Terminal,
}

impl AiMode {
    fn display_name(&self) -> &str {
        match self {
            AiMode::Director => "Director / PA",
            AiMode::Programmer => "Programmer",
            AiMode::Vision => "Vision",
            AiMode::Terminal => "Terminal",
        }
    }
    
    fn icon(&self) -> &str {
        match self {
            AiMode::Director => "🎯",
            AiMode::Programmer => "💻",
            AiMode::Vision => "👁️",
            AiMode::Terminal => "📟",
        }
    }
}

pub struct KaelApp {
    messages: Vec<ChatMessage>,
    input_text: String,
    current_ai: AiMode,
    is_loading: bool,
    chat_history: VecDeque<(AiMode, Vec<ChatMessage>)>,
    llama_engine: LlamaEngine,
    runtime: Runtime,
    model_downloader: ModelDownloader,
    training_manager: TrainingManager,
    ollama_url: String,
    terminal: Terminal,
    sudo_set: bool,
    selected_image: Option<String>,
    vault: Option<Vault>,
    terminal_output: String,
    terminal_input: String,
    downloading: bool,
    download_progress: String,
    left_panel: LeftPanel,
    right_panel: RightPanel,
    uploaded_file: Option<String>,
}

impl KaelApp {
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        let llama_engine = LlamaEngine::new();
        let model_downloader = ModelDownloader::new();
        let training_manager = TrainingManager::new();
        let ollama_url = "http://localhost:11434".to_string();
        
        // List available models 
        let _models = LlamaEngine::list_available_models();
        
        let vault = match Vault::new() {
            Ok(v) => Some(v),
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
            llama_engine,
            runtime,
            model_downloader,
            training_manager,
            ollama_url,
            terminal: Terminal::new(),
            sudo_set: false,
            selected_image: None,
            vault,
            terminal_output: String::from("Welcome to Kael Terminal\nType commands and press Enter to execute.\nUse 'sudo' commands - password will be prompted.\n\n$ "),
            terminal_input: String::new(),
            downloading: false,
            download_progress: String::new(),
            left_panel: LeftPanel::Chat,
            right_panel: RightPanel::Tasks,
            uploaded_file: None,
        };
        
        // Check if models exist, if not show download option
        let models = ModelDownloader::list_downloaded_models();
        let welcome_msg = if !models.is_empty() {
            format!(
                "Hello! I'm Kael, your AI assistant.\n\n✅ Models loaded: {}\n\nI'll learn from you over time through RAG/Lora.\nWhen knowledge grows large enough, I'll 'bake' it into smarter models.\n\nJust chat naturally - tell me what you need!",
                models.iter().map(|(t, _)| t.to_string()).collect::<Vec<_>>().join(", ")
            )
        } else {
            format!(
                "Hello! I'm Kael, your AI assistant.\n\n⚠️ No models downloaded yet.\n\nClick '⬇️ Download Models' in the sidebar to get started.\n\nI'll use the smallest Dolphin models for Director/Programmer\nand LLaVA for Vision.\n\nI'll learn from you through RAG/SQL and get smarter over time!"
            )
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
            // For now, all AI modes use the director model (vision not supported in llama-gguf)
            let ai_type = match ai {
                AiMode::Director => "director",
                AiMode::Programmer => "director",  // Use director for now
                AiMode::Vision => "director",       // Use director for now - vision not supported
                AiMode::Terminal => "director",
            };
            
            // Check if we need to load a different model
            let current_model = self.llama_engine.get_model_path()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));
            
            let need_load = match &current_model {
                Some(m) => m.as_str() != format!("{}.gguf", ai_type),
                None => true,
            };
            
            // Load appropriate model if needed
            if need_load && ModelDownloader::model_exists(ai_type) {
                self.llama_engine.load_ai_model(ai_type).ok();
            }
            
            self.chat_history.push_front((self.current_ai, std::mem::take(&mut self.messages)));
            self.messages = self.chat_history
                .iter()
                .find(|(mode, _)| *mode == ai)
                .map(|(_, msgs)| msgs.clone())
                .unwrap_or_else(Vec::new);
            
            if self.messages.is_empty() {
                let welcome = match ai {
                    AiMode::Director => "You are now chatting with Director. Just tell me what you need!",
                    AiMode::Programmer => "You are now chatting with Programmer. What would you like to code?",
                    AiMode::Vision => "You are now chatting with Vision. Send me images to analyze.",
                    AiMode::Terminal => "Terminal mode - type commands below.",
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
        let content: String = self.input_text.trim().to_string();
        if content.is_empty() {
            return;
        }
        
        // Auto-detect intent from message
        let request_type = RequestType::from_message(&content);
        
        // Handle terminal mode directly
        if self.current_ai == AiMode::Terminal {
            self.handle_terminal_command(&content);
            self.input_text.clear();
            return;
        }
        
        // Check for image references
        if content.starts_with("/image ") || request_type == RequestType::Vision {
            self.handle_image_message(&content);
            return;
        }
        
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
            request_type: Some(request_type),
        });
        
        self.input_text.clear();
        self.is_loading = true;
    }
    
    fn handle_terminal_command(&mut self, command: &str) {
        let result = self.terminal.execute(command);
        
        let response = if result.success {
            if result.stdout.is_empty() && result.stderr.is_empty() {
                format!("$ {}\n(no output)\n\n$ ", command)
            } else {
                format!("$ {}\n{}\n\n$ ", command, result.stdout)
            }
        } else if result.needs_sudo {
            format!("$ {}\n⚠️ Sudo required. Please set your sudo password in settings or use: sudo {}\n\n$ ", command, command.strip_prefix("sudo ").unwrap_or(command))
        } else {
            format!("$ {}\n❌ {}\n$ ", command, result.stderr)
        };
        
        self.terminal_output.push_str(&response);
    }
    
    fn handle_image_message(&mut self, content: &str) {
        let path = content.strip_prefix("/image ").unwrap_or(content).trim();
        
        if path.is_empty() {
            self.messages.push(ChatMessage {
                role: MessageRole::Assistant,
                content: "Please provide an image path, e.g., /image ~/picture.png".to_string(),
                timestamp: chrono::Utc::now(),
                request_type: Some(RequestType::Vision),
            });
            self.input_text.clear();
            return;
        }
        
        let path_obj = std::path::Path::new(path);
        if !path_obj.exists() {
            self.messages.push(ChatMessage {
                role: MessageRole::Assistant,
                content: format!("File not found: {}", path),
                timestamp: chrono::Utc::now(),
                request_type: Some(RequestType::Vision),
            });
            self.input_text.clear();
            return;
        }
        
        self.selected_image = Some(path.to_string());
        
        let file_name = path_obj.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Image".to_string());
        
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: format!("[Analyzing image: {}]", file_name),
            timestamp: chrono::Utc::now(),
            request_type: Some(RequestType::Vision),
        });
        
        self.input_text.clear();
        self.is_loading = true;
    }
    
    fn generate_response(&mut self) {
        let last_message = self.messages.last().map(|m| m.content.clone()).unwrap_or_default();
        let request_type = RequestType::from_message(&last_message);
        
        // Auto-switch AI based on detected intent
        match request_type {
            RequestType::Code => {
                // Stay with Director - no separate programmer model
            }
            RequestType::Vision => {
                // Stay with Director - vision not supported yet
            }
            RequestType::Install | RequestType::Terminal => {
                // Stay with Director - terminal handled separately
            }
            _ => {
                // Stay with Director for everything
            }
        }
        
        // Get AI-specific model path - always use director for now (vision not supported)
        let ai_type_str = "director";
        
        // Build system prompt based on AI mode
        let system_prompt = match self.current_ai {
            AiMode::Director => "You are Kael, a helpful AI assistant. Be concise and practical. Help with scheduling, email, and general tasks.",
            AiMode::Programmer => "You are Kael, a programming assistant. Provide clear, accurate code and explanations. Focus on Rust, but can help with any language.",
            AiMode::Vision => "You are Kael with vision capabilities. Describe images accurately.",
            AiMode::Terminal => "You are Kael helping with terminal commands. Provide command-line guidance.",
        };
        
        // Get RAG context for this AI
        let rag_context = self.training_manager
            .for_ai(ai_type_str)
            .get_rag_context(20)
            .unwrap_or_default();
        
        // Get SQL context (trivial stuff)
        let sql_context = self.training_manager
            .for_ai(ai_type_str)
            .get_sql_context(10)
            .unwrap_or_default();
        
        // Try to load model if not loaded
        if !self.llama_engine.is_loaded() {
            if ModelDownloader::model_exists(ai_type_str) {
                let model_path = ModelDownloader::get_model_path(ai_type_str);
                if let Err(e) = self.llama_engine.load_model(model_path.to_str().unwrap_or("")) {
                    eprintln!("Failed to load model: {}", e);
                }
            }
        }
        
        // Use local model if loaded
        let response = if self.llama_engine.is_loaded() {
            // Get conversation context
            let context: String = self.messages
                .iter()
                .map(|m| format!("{}: {}", 
                    match m.role {
                        MessageRole::User => "User",
                        MessageRole::Assistant => "Assistant",
                        MessageRole::System => "System",
                    },
                    m.content
                ))
                .collect::<Vec<_>>()
                .join("\n");
            
            // Build prompt with system + RAG + context + user message
            let mut full_prompt = format!("System: {}\n\n", system_prompt);
            if !rag_context.is_empty() {
                full_prompt.push_str(&rag_context);
                full_prompt.push_str("\n\n");
            }
            if !sql_context.is_empty() {
                full_prompt.push_str(&sql_context);
                full_prompt.push_str("\n\n");
            }
            full_prompt.push_str(&context);
            full_prompt.push_str(&format!("\nUser: {}\nAssistant:", last_message));
            
            match self.llama_engine.generate(&full_prompt, Some(256)) {
                Ok(response) => response,
                Err(e) => format!("Error generating: {}", e),
            }
        } else {
            // Demo mode - no model loaded yet
            let demo_responses = vec![
                "Hi! I'm Kael. The AI model is still loading. Please wait or check if the model is downloaded.",
                "Hello! I can help you with coding, scheduling, and more once the AI model is ready.",
                "Hey there! The model is loading - this should only take a moment on powerful hardware.",
            ];
            
            // Simple response based on message content
            let msg_lower = last_message.to_lowercase();
            if msg_lower.contains("hello") || msg_lower.contains("hi") {
                "Hello! I'm Kael. The AI model is loading - please wait a moment.".to_string()
            } else if msg_lower.contains("code") || msg_lower.contains("program") {
                "I can help with coding! The model is loading - try again in a moment.".to_string()
            } else if msg_lower.contains("schedule") || msg_lower.contains("calendar") {
                "I can help with scheduling! The model is loading - please wait.".to_string()
            } else {
                demo_responses[0].to_string()
            }
        };
        
        // Record interaction for learning (after response)
        if self.current_ai != AiMode::Terminal {
            self.training_manager
                .for_ai(ai_type_str)
                .record_interaction(&last_message, &response)
                .ok();
        }
        
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
                    struct Response { message: MessageContent }
                    #[derive(Deserialize)]
                    struct MessageContent { content: String }
                    
                    match resp.json::<Response>() {
                        Ok(r) => r.message.content,
                        Err(e) => format!("Failed to parse: {}", e),
                    }
                } else {
                    format!("Error: {}", resp.status())
                }
            }
            Err(e) => format!("Connection error: {}", e),
        }
    }
    
    fn demo_response(&self, message: &str, request_type: RequestType) -> String {
        match request_type {
            RequestType::Schedule => {
                format!(
                    "📅 **Schedule Request**\n\nI'd help you with:\n• Adding calendar events\n• Setting reminders\n• Checking your schedule\n\nWhat would you like to do? Example: \"Schedule a meeting tomorrow at 3pm\""
                )
            }
            RequestType::Email => {
                "📧 **Email Request**\n\nI'd help you compose and send emails.\n\nWhat would you like to do? Example: \"Send an email to John about the project\"".to_string()
            }
            RequestType::Install => {
                let pm = self.terminal.check_package_manager();
                format!(
                    "📦 **Install Request**\n\nI can help you install applications!\n\nYour system uses: **{}**\n\nWhat would you like to install? Example: \"Install Firefox\"",
                    pm
                )
            }
            RequestType::Code => {
                format!(
                    "💻 **Programming**\n\nRouting to Programmer AI...\n\nYour question: {}\n\n(Demo mode - Ollama not running)",
                    message.chars().take(100).collect::<String>()
                )
            }
            RequestType::Vision => {
                "👁️ **Vision**\n\nI'd analyze your image!\n\n(Demo mode - Ollama not running)".to_string()
            }
            RequestType::Terminal => {
                "📟 **Terminal**\n\nI'll run commands for you!\n\nWhat would you like to do? Example: \"List files in current directory\"".to_string()
            }
            RequestType::Search => {
                format!(
                    "🔍 **Search**\n\nI'd search the web for: \"{}\"\n\n(Demo mode - Ollama not running)",
                    message
                )
            }
            _ => {
                format!(
                    "💬 **Chat**\n\nI understand you're chatting with me!\n\n**Detected intent:** {:?}\n**Routing to:** {}\n\n(Demo mode - Ollama not running)",
                    request_type,
                    request_type.target_ai()
                )
            }
        }
    }
}

impl AiMode {
    fn model_name(&self) -> &str {
        match self {
            AiMode::Director => "dolphin3.0-mistral-7b-q4",
            AiMode::Programmer => "dolphin3.0-coder-7b-q4",
            AiMode::Vision => "llava-7b-q4",
            AiMode::Terminal => "dolphin3.0-mistral-7b-q4",
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
        
        let screen_size = ctx.screen_rect().size();
        let left_width = screen_size.x * 0.20;
        let right_width = screen_size.x * 0.20;
        let _middle_width = screen_size.x * 0.60;
        
        // LEFT PANEL (20%) - Navigation
        egui::SidePanel::left("kael_left_panel")
            .width_range(left_width..=left_width)
            .show(ctx, |ui| {
                ui.heading("🤖 Kael");
                ui.separator();
                
                // Navigation tabs
                ui.label(egui::RichText::new("Navigate").color(egui::Color32::GRAY));
                
                if ui.selectable_label(self.left_panel == LeftPanel::Chat, "💬 Chat").clicked() {
                    self.left_panel = LeftPanel::Chat;
                }
                if ui.selectable_label(self.left_panel == LeftPanel::Calendar, "📅 Calendar").clicked() {
                    self.left_panel = LeftPanel::Calendar;
                }
                if ui.selectable_label(self.left_panel == LeftPanel::Settings, "⚙️ Settings").clicked() {
                    self.left_panel = LeftPanel::Settings;
                }
                
                ui.separator();
                
                // Upload files for AI
                ui.label(egui::RichText::new("Upload").color(egui::Color32::GRAY));
                
                if ui.button("📁 Upload File").clicked() {
                    // For now just show a message - file dialog would need native dialog
                    self.input_text = "I want to analyze a file. (File upload coming soon)".to_string();
                }
                
                // Quick actions
                ui.separator();
                ui.label(egui::RichText::new("Quick Actions").color(egui::Color32::GRAY));
                
                // Image analysis - disabled until vision support added
                // if ui.button("📷 Analyze Image").clicked() {
                //     self.input_text = "Analyze this image: ".to_string();
                // }
                if ui.button("💻 Write Code").clicked() {
                    self.input_text = "Help me write code: ".to_string();
                }
                if ui.button("📅 Schedule").clicked() {
                    self.input_text = "Schedule: ".to_string();
                }
                
                // Status
                ui.separator();
                ui.label(egui::RichText::new("Status").color(egui::Color32::GRAY));
                if self.llama_engine.is_loaded() {
                    let model_name = self.llama_engine.get_model_path()
                        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                        .unwrap_or_else(|| "Model".to_string());
                    ui.label(egui::RichText::new(format!("🟢 {}", model_name)).color(egui::Color32::from_rgb(16, 185, 129)));
                } else {
                    ui.label(egui::RichText::new("🔴 No model").color(egui::Color32::from_rgb(239, 68, 68)));
                }
                
                // Sudo status
                if self.sudo_set {
                    ui.label(egui::RichText::new("🔑 Sudo ready").color(egui::Color32::from_rgb(16, 185, 129)));
                } else {
                    ui.label(egui::RichText::new("⚠️ Sudo not set").color(egui::Color32::from_rgb(251, 191, 36)));
                }
                
                // Download button
                ui.separator();
                if !self.downloading {
                    if ui.button("⬇️ Download Models").clicked() {
                        self.downloading = true;
                        let downloader = self.model_downloader.clone();
                        std::thread::spawn(move || {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            rt.block_on(async {
                                for ai_type in ["director", "programmer", "vision"] {
                                    if !ModelDownloader::model_exists(ai_type) {
                                        match downloader.download_model(ai_type).await {
                                            Ok(path) => println!("Downloaded {} to {:?}", ai_type, path),
                                            Err(e) => println!("Failed: {}: {}", ai_type, e),
                                        }
                                    }
                                }
                            });
                        });
                    }
                } else {
                    ui.label("⬇️ Downloading...");
                }
                
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.separator();
                    ui.label(egui::RichText::new("v0.4.0").small().color(egui::Color32::GRAY));
                });
            });
        
        // RIGHT PANEL (20%) - Tasks/Projects
        egui::SidePanel::right("kael_right_panel")
            .width_range(right_width..=right_width)
            .show(ctx, |ui| {
                ui.heading("📋 Tasks");
                ui.separator();
                
                // Toggle between Tasks and Projects
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.right_panel == RightPanel::Tasks, "📝 Tasks").clicked() {
                        self.right_panel = RightPanel::Tasks;
                    }
                    if ui.selectable_label(self.right_panel == RightPanel::Projects, "📁 Projects").clicked() {
                        self.right_panel = RightPanel::Projects;
                    }
                });
                
                ui.separator();
                
                match self.right_panel {
                    RightPanel::Tasks => {
                        ui.label("Active Tasks:");
                        ui.label("• Task 1 (coming soon)");
                        ui.label("• Task 2 (coming soon)");
                    }
                    RightPanel::Projects => {
                        ui.label("Active Projects:");
                        ui.label("• Project 1 (coming soon)");
                        ui.label("• Project 2 (coming soon)");
                    }
                }
                
                // Training status for current AI
                ui.separator();
                let ai_type = match self.current_ai {
                    AiMode::Director => "director",
                    AiMode::Programmer => "programmer",
                    AiMode::Vision => "vision",
                    _ => "terminal",
                };
                
                if let Ok(stats) = self.training_manager.for_ai(ai_type).get_stats() {
                    ui.label(egui::RichText::new("Training").color(egui::Color32::GRAY));
                    ui.label(format!("💾 SQL: {}", stats.sql_items));
                    ui.label(format!("📚 RAG: {}", stats.rag_items));
                    ui.label(format!("🎯 LoRA: {}", stats.lora_items));
                    ui.label(format!("✅ Baked: {}", stats.baked_items));
                    
                    if stats.should_promote_to_rag || stats.should_create_lora || stats.should_bake {
                        ui.label(egui::RichText::new("💡 Action needed").color(egui::Color32::from_rgb(234, 179, 8)));
                    }
                }
            });
        
        // MIDDLE PANEL (60%) - Working Area
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                let total_height = ui.available_height();
                
                // Reserve bottom 20% for terminal (min 80px)
                let terminal_h = (total_height * 0.20).max(80.0);
                let chat_h = total_height - terminal_h - 2.0;
                
                // Get the available rect
                let avail = ui.available_rect_before_wrap();
                
                // Top: Main content (80%)
                let chat_rect = egui::Rect::from_min_size(
                    egui::pos2(avail.min.x, avail.min.y),
                    egui::vec2(avail.width(), chat_h)
                );
                ui.allocate_ui_at_rect(chat_rect, |ui| {
                    self.render_main_content(ui);
                });
                
                // Bottom: Terminal (20%)
                let term_rect = egui::Rect::from_min_size(
                    egui::pos2(avail.min.x, avail.min.y + chat_h),
                    egui::vec2(avail.width(), terminal_h)
                );
                ui.allocate_ui_at_rect(term_rect, |ui| {
                    self.render_terminal_area(ui);
                });
            });
    }
}

impl KaelApp {
    fn render_main_content(&mut self, ui: &mut egui::Ui) {
        // Based on left panel selection
        match self.left_panel {
            LeftPanel::Chat => {
                self.render_chat_area(ui);
            }
            LeftPanel::Calendar => {
                ui.heading("📅 Calendar");
                ui.separator();
                ui.label("Calendar view coming soon...");
            }
            LeftPanel::Settings => {
                ui.heading("⚙️ Settings");
                ui.separator();
                ui.label("Settings view coming soon...");
            }
        }
    }
}

impl KaelApp {
    fn render_chat_area(&mut self, ui: &mut egui::Ui) {
        // Header based on left panel selection
        match self.left_panel {
            LeftPanel::Chat => {
                ui.heading("💬 Chat");
            }
            LeftPanel::Calendar => {
                ui.heading("📅 Calendar");
                ui.label("Calendar view coming soon...");
                return;
            }
            LeftPanel::Settings => {
                ui.heading("⚙️ Settings");
                ui.label("Settings view coming soon...");
                return;
            }
        }
        
        // Show loading indicator
        ui.horizontal(|ui| {
            if self.is_loading {
                ui.spinner();
                ui.label("Kael is thinking...");
            }
        });
        
        ui.add_space(5.0);
        
        // Chat messages
        egui::ScrollArea::vertical()
            .id_salt("chat_scroll")
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for message in &self.messages {
                    self.render_message(ui, message);
                }
            });
        
        ui.add_space(10.0);
        
        // Input
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.input_text)
                .hint_text("Message Kael...")
                .desired_width(ui.available_width() - 80.0));
            
            if ui.button("Send ➤").clicked() {
                let msg = self.input_text.clone();
                if !msg.is_empty() {
                    self.send_message();
                    self.input_text.clear();
                }
            }
        });
        
        // Handle Enter key
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.is_loading && !self.input_text.is_empty() {
            self.send_message();
            self.input_text.clear();
        }
    }
    
    fn render_terminal_area(&mut self, ui: &mut egui::Ui) {
        ui.heading("📟 Terminal");
        
        // Terminal output
        egui::ScrollArea::vertical()
            .id_salt("terminal_scroll")
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut self.terminal_output.clone())
                    .desired_rows(10)
                    .desired_width(ui.available_width())
                    .interactive(false)
                    .code_editor());
            });
        
        // Terminal input
        ui.horizontal(|ui| {
            ui.label("$ ");
            ui.add(egui::TextEdit::singleline(&mut self.terminal_input)
                .desired_width(ui.available_width() - 30.0));
            
            let input_clone = self.terminal_input.clone();
            if ui.button("Run").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !input_clone.is_empty() {
                    self.handle_terminal_command(&input_clone);
                    self.terminal_input.clear();
                }
            }
        });
    }
}

impl KaelApp {
    fn render_message(&self, ui: &mut egui::Ui, message: &ChatMessage) {
        let is_user = message.role == MessageRole::User;
        
        let (bg_color, text_color, align) = if is_user {
            (egui::Color32::from_rgb(16, 163, 127), egui::Color32::WHITE, egui::Align::Max)
        } else {
            (egui::Color32::from_rgb(40, 40, 40), egui::Color32::from_rgb(220, 220, 220), egui::Align::Min)
        };
        
        ui.with_layout(egui::Layout::right_to_left(align), |ui| {
            ui.add_space(10.0);
            
            // Chat bubble with background
            ui.allocate_ui(egui::vec2(ui.available_width() * 0.7, 0.0), |ui| {
                egui::Frame::none()
                    .fill(bg_color)
                    .rounding(12.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        // Name
                        ui.horizontal(|ui| {
                            let avatar = if is_user { "👤" } else { "🤖" };
                            ui.label(avatar);
                            ui.label(egui::RichText::new(if is_user { "You" } else { "Kael" })
                                .color(text_color)
                                .strong());
                        });
                        
                        ui.add_space(5.0);
                        
                        // Message
                        ui.label(egui::RichText::new(&message.content).color(text_color));
                    });
            });
        });
        
        ui.add_space(8.0);
    }
}
