use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub user_message: String,
    pub has_image: bool,
    pub image_path: Option<String>,
    pub chat_history: Vec<ChatMessage>,
    pub current_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestType {
    Chat,           // General conversation
    Schedule,       // Calendar/scheduling related
    Email,          // Email related
    Code,           // Programming/coding related
    Vision,         // Image analysis
    Install,        // Install application
    Search,         // Internet search
    FileOperation,  // File operations
    System,         // System commands
    Unknown,        // Undetermined
}

impl RequestType {
    pub fn from_message(message: &str) -> Self {
        let msg_lower = message.to_lowercase();
        
        // Check for image
        if msg_lower.contains("image") || msg_lower.contains("picture") || msg_lower.contains("photo") || msg_lower.contains("screenshot") {
            return RequestType::Vision;
        }
        
        // Check for install requests
        if msg_lower.contains("install") || msg_lower.contains("download") || msg_lower.contains("apt") || msg_lower.contains("pacman") || msg_lower.contains("brew") || msg_lower.contains("make") {
            return RequestType::Install;
        }
        
        // Check for schedule/calendar
        if msg_lower.contains("schedule") || msg_lower.contains("calendar") || msg_lower.contains("meeting") || msg_lower.contains("appointment") || msg_lower.contains("remind") || msg_lower.contains("event") {
            return RequestType::Schedule;
        }
        
        // Check for email
        if msg_lower.contains("email") || msg_lower.contains("mail") || msg_lower.contains("send") || msg_lower.contains("inbox") || msg_lower.contains("message") {
            return RequestType::Email;
        }
        
        // Check for code/programming
        if msg_lower.contains("code") || msg_lower.contains("program") || msg_lower.contains("function") || msg_lower.contains("debug") || msg_lower.contains("error") || msg_lower.contains("compile") || msg_lower.contains("script") || msg_lower.contains("rust") || msg_lower.contains("python") || msg_lower.contains("javascript") || msg_lower.contains("git") {
            return RequestType::Code;
        }
        
        // Check for file operations
        if msg_lower.contains("file") || msg_lower.contains("folder") || msg_lower.contains("directory") || msg_lower.contains("create") || msg_lower.contains("delete") || msg_lower.contains("move") || msg_lower.contains("copy") {
            return RequestType::FileOperation;
        }
        
        // Check for system commands
        if msg_lower.contains("system") || msg_lower.contains("process") || msg_lower.contains("cpu") || msg_lower.contains("memory") || msg_lower.contains("kill") || msg_lower.contains("restart") {
            return RequestType::System;
        }
        
        // Check for search
        if msg_lower.contains("search") || msg_lower.contains("find") || msg_lower.contains("look up") || msg_lower.contains("what is") || msg_lower.contains("who is") || msg_lower.contains("how to") || msg_lower.contains("?") {
            return RequestType::Search;
        }
        
        RequestType::Chat
    }
    
    pub fn target_ai(&self) -> &str {
        match self {
            RequestType::Chat => "Director",
            RequestType::Schedule => "Director-PA",
            RequestType::Email => "Director-PA", 
            RequestType::Code => "Programmer",
            RequestType::Vision => "Vision",
            RequestType::Install => "Director-Install",
            RequestType::Search => "Director-Search",
            RequestType::FileOperation => "Director",
            RequestType::System => "Director",
            RequestType::Unknown => "Director",
        }
    }
    
    pub fn description(&self) -> &str {
        match self {
            RequestType::Chat => "General conversation",
            RequestType::Schedule => "Calendar/Scheduling",
            RequestType::Email => "Email management",
            RequestType::Code => "Programming help",
            RequestType::Vision => "Image analysis",
            RequestType::Install => "Install application",
            RequestType::Search => "Internet search",
            RequestType::FileOperation => "File operations",
            RequestType::System => "System commands",
            RequestType::Unknown => "Undetermined",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub content: String,
    pub request_type: RequestType,
    pub target_ai: String,
    pub actions: Vec< AIAction>,
    pub needs_image: bool,
    pub needs_user_confirm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIAction {
    SearchInternet { query: String },
    AnalyzeImage { path: String },
    InstallApp { name: String, command: String },
    RunCommand { command: String },
    SendEmail { to: String, subject: String, body: String },
    AddToCalendar { event: String, time: String },
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    None,
}

pub struct AIOrchestrator {
    pub ollama_url: String,
    pub model: String,
}

impl AIOrchestrator {
    pub fn new(ollama_url: String, model: String) -> Self {
        Self { ollama_url, model }
    }
    
    pub fn analyze_request(&self, context: &RequestContext) -> RequestType {
        RequestType::from_message(&context.user_message)
    }
    
    pub fn build_prompt(&self, context: &RequestContext, request_type: RequestType) -> String {
        let system_prompt = match request_type {
            RequestType::Chat => {
                "You are Kael, a helpful AI assistant. Be direct, concise, and practical. Provide useful answers to the user's questions."
            }
            RequestType::Schedule => {
                "You are Kael's PA (Personal Assistant) protocol. Handle calendar and scheduling tasks. Be organized and confirm details before proceeding."
            }
            RequestType::Email => {
                "You are Kael's Email protocol. Help compose, send, and manage emails. Be professional and ask for confirmation before sending."
            }
            RequestType::Code => {
                "You are Kael's Programmer AI. Help with coding, debugging, and technical questions. Provide clear, working code examples when possible."
            }
            RequestType::Vision => {
                "You are Kael's Vision AI. Analyze images carefully. Describe what you see in detail and answer questions about the image."
            }
            RequestType::Install => {
                "You are Kael's Install protocol. When asked to install something, first determine what package manager is needed, then provide the installation command. Ask for confirmation before running."
            }
            RequestType::Search => {
                "You are Kael with internet access. Search the web for current information. Provide accurate, up-to-date answers."
            }
            _ => {
                "You are Kael, a helpful AI assistant. Be direct and practical."
            }
        };
        
        let history_context: String = context.chat_history
            .iter()
            .rev()
            .take(5)
            .map(|m| format!("{}: {}", 
                match m.role {
                    MessageRole::User => "User",
                    MessageRole::Assistant => "Kael",
                    MessageRole::System => "System",
                },
                m.content
            ))
            .collect::<Vec<_>>()
            .join("\n");
        
        format!(
            "{}\n\nRecent conversation:\n{}\n\nUser: {}",
            system_prompt,
            history_context,
            context.user_message
        )
    }
}
