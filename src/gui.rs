use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

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

impl MessageRole {
    fn color(&self) -> egui::Color32 {
        match self {
            MessageRole::User => egui::Color32::from_rgb(16, 163, 127),
            MessageRole::Assistant => egui::Color32::from_rgb(139, 92, 246),
            MessageRole::System => egui::Color32::from_rgb(156, 163, 175),
        }
    }
    
    fn avatar(&self) -> &str {
        match self {
            MessageRole::User => "👤",
            MessageRole::Assistant => "🤖",
            MessageRole::System => "⚙️",
        }
    }
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
}

#[derive(Default)]
pub struct KaelApp {
    messages: Vec<ChatMessage>,
    input_text: String,
    current_ai: AiMode,
    is_loading: bool,
    chat_history: VecDeque<(AiMode, Vec<ChatMessage>)>,
}

impl KaelApp {
    pub fn new() -> Self {
        let mut app = Self::default();
        
        app.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: "Hello! I'm Kael, your AI assistant.\n\nI can help you with:\n• General conversation and tasks (Director)\n• Programming and code help (Programmer)\n• Image analysis (Vision)\n\nSelect an AI from the sidebar and start chatting!".to_string(),
            timestamp: chrono::Utc::now(),
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
                    AiMode::Director => "You are now chatting with Director. I'm here to help with general tasks, questions, and conversation.",
                    AiMode::Programmer => "You are now chatting with Programmer. I'm here to help with coding, debugging, and technical questions.",
                    AiMode::Vision => "You are now chatting with Vision. You can send me images to analyze. Use the paperclip button to image.",
                };
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: welcome.to_string(),
                    timestamp: chrono::Utc::now(),
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
        });
        
        self.input_text.clear();
        self.is_loading = true;
    }
    
    fn generate_response(&mut self) {
        let last_message = self.messages.last().map(|m| m.content.clone()).unwrap_or_default();
        
        let response = match self.current_ai {
            AiMode::Director => format!(
                "I'm running in demo mode. My Director AI would respond to: \"{}\"\n\nOllama integration coming soon!",
                last_message
            ),
            AiMode::Programmer => format!(
                "// I'm running in demo mode.\n// My Programmer AI would respond to your code question.\n// Ollama integration coming soon!\n\n// Regarding: {}\n\nfn example() {{\n    // Code analysis would go here\n}}",
                last_message
            ),
            AiMode::Vision => format!(
                "I'm running in demo mode. My Vision AI would analyze images.\n\nOllama/LLaVA integration coming soon!"
            ),
        };
        
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: response,
            timestamp: chrono::Utc::now(),
        });
        
        self.is_loading = false;
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
                ui.label(egui::RichText::new("Current:").color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(format!("{} {}", self.current_ai.icon(), self.current_ai.display_name()))
                    .color(egui::Color32::from_rgb(139, 92, 246)));
                
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.separator();
                    ui.label(egui::RichText::new("v0.1.0").color(egui::Color32::GRAY));
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
                        .desired_width(ui.available_width() - 80.0);
                    
                    ui.add(text_edit);
                    
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
                        ui.label(egui::RichText::new("Thinking...").color(egui::Color32::from_rgb(251, 191, 36)));
                    } else {
                        ui.label(egui::RichText::new("Ready").color(egui::Color32::from_rgb(16, 185, 129)));
                    }
                    
                    ui.separator();
                    
                    if ui.button("Clear 🗑️").clicked() {
                        self.messages.clear();
                        self.messages.push(ChatMessage {
                            role: MessageRole::Assistant,
                            content: "Chat cleared. How can I help you?".to_string(),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                });
            });
    }
}

impl KaelApp {
    fn render_message(&self, ui: &mut egui::Ui, message: &ChatMessage) {
        let is_user = message.role == MessageRole::User;
        
        let align = if is_user {
            egui::Align::Max
        } else {
            egui::Align::Min
        };
        
        ui.with_layout(egui::Layout::right_to_left(align), |ui| {
            ui.add_space(10.0);
            
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(message.role.avatar());
                    if !is_user {
                        ui.label(egui::RichText::new("Kael").color(egui::Color32::from_rgb(139, 92, 246)));
                    } else {
                        ui.label("You");
                    }
                });
                
                ui.separator();
                
                ui.label(&message.content);
            });
        });
        
        ui.add_space(5.0);
    }
}
