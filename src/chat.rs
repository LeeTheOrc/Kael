use std::io::{self, Write};
use anyhow::Result;
use crate::config::Config;

mod config;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

pub struct ChatInterface {
    config: Config,
    messages: Vec<ChatMessage>,
    current_ai: AiMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AiMode {
    Director,
    Programmer,
    Vision,
}

impl ChatInterface {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            messages: Vec::new(),
            current_ai: AiMode::Director,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            print!("\n[{}] > ", self.current_ai_name());
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if input.is_empty() {
                continue;
            }
            
            match self.handle_command(input) {
                Ok(true) => continue,
                Ok(false) => break,
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        
        println!("\nGoodbye!");
        Ok(())
    }

    fn current_ai_name(&self) -> &str {
        match self.current_ai {
            AiMode::Director => "Director",
            AiMode::Programmer => "Programmer",
            AiMode::Vision => "Vision",
        }
    }

    fn handle_command(&mut self, input: &str) -> Result<bool> {
        match input {
            "/help" => {
                self.show_help();
                Ok(true)
            }
            "/quit" | "/exit" => {
                Ok(false)
            }
            "/clear" => {
                self.messages.clear();
                println!("Chat history cleared.");
                Ok(true)
            }
            "/switch" => {
                self.switch_ai();
                Ok(true)
            }
            "/vision" => {
                self.current_ai = AiMode::Vision;
                println!("Switched to Vision AI mode. Use /image <path> to send an image.");
                Ok(true)
            }
            cmd if cmd.starts_with("/image ") => {
                let path = cmd.strip_prefix("/image ").unwrap().trim();
                self.handle_image(path)?;
                Ok(true)
            }
            _ => {
                self.messages.push(ChatMessage {
                    role: MessageRole::User,
                    content: input.to_string(),
                });
                
                let response = self.get_ai_response(input)?;
                println!("\n[Kael - {}]: {}", self.current_ai_name(), response);
                
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: response,
                });
                
                Ok(true)
            }
        }
    }

    fn show_help(&self) {
        println!("
===========================================
           Kael Commands
===========================================
/help     - Show this help message
/clear    - Clear chat history
/switch   - Switch between Director and Programmer AI
/vision   - Switch to Vision AI mode
/image <path> - Send image to Vision AI
/quit     - Exit Kael

Current AI: {}
===========================================",
            self.current_ai_name()
        );
    }

    fn switch_ai(&mut self) {
        self.current_ai = match self.current_ai {
            AiMode::Director => AiMode::Programmer,
            AiMode::Programmer => AiMode::Director,
            AiMode::Vision => AiMode::Director,
        };
        println!("Switched to {} AI", self.current_ai_name());
    }

    fn handle_image(&self, path: &str) -> Result<()> {
        println!("Processing image: {}", path);
        println!("(Vision AI integration coming soon)");
        Ok(())
    }

    fn get_ai_response(&self, input: &str) -> Result<String> {
        let model = match self.current_ai {
            AiMode::Director => &self.config.models.director_model,
            AiMode::Programmer => &self.config.models.programmer_model,
            AiMode::Vision => &self.config.models.vision_model,
        };

        Ok(format!(
            "Using model: {} (Ollama integration coming soon)\nYour message: {}",
            model, input
        ))
    }
}
