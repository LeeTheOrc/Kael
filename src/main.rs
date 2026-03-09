mod ai;
mod chat;
mod vault;
mod profiles;
mod config;

use anyhow::Result;
use chat::{ChatInterface, ChatMessage, MessageRole};
use config::Config;
use std::path::PathBuf;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

fn get_project_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("com", "kaelos", "Kael")
        .expect("Failed to get project directories")
}

fn setup_logging(log_dir: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(log_dir)?;
    
    let file_appender = tracing_appender::rolling::daily(log_dir, "kael.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(non_blocking)
        .with_ansi(false)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)?;
    
    Ok(())
}

fn main() -> Result<()> {
    let proj_dirs = get_project_dirs();
    let data_dir = proj_dirs.data_dir();
    let config_dir = proj_dirs.config_dir();
    let log_dir = data_dir.join("logs");
    
    std::fs::create_dir_all(data_dir)?;
    std::fs::create_dir_all(config_dir)?;
    
    setup_logging(&log_dir)?;
    
    info!("Starting Kael AI Assistant");
    info!("Data directory: {:?}", data_dir);
    info!("Config directory: {:?}", config_dir);
    
    let config = Config::load(config_dir)?;
    
    let mut chat = ChatInterface::new(config.clone());
    
    println!("===========================================");
    println!("         Kael AI Assistant");
    println!("===========================================");
    println!();
    println!("Available commands:");
    println!("  /help     - Show this help message");
    println!("  /clear    - Clear chat history");
    println!("  /switch   - Switch between AI (director/programmer)");
    println!("  /vision   - Send image to vision AI");
    println!("  /quit     - Exit Kael");
    println!();
    println!("Current AI: Director");
    println!("===========================================");
    println!();
    
    chat.run()?;
    
    Ok(())
}
