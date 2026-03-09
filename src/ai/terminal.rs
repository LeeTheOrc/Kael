use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub needs_sudo: bool,
}

pub struct Terminal {
    pub shell: String,
    pub sudo_password: Option<String>,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            shell: "bash".to_string(),
            sudo_password: None,
        }
    }
    
    pub fn set_sudo_password(&mut self, password: String) {
        self.sudo_password = Some(password);
    }
    
    pub fn clear_sudo_password(&mut self) {
        self.sudo_password = None;
    }
    
    pub fn execute(&self, command: &str) -> CommandResult {
        let needs_sudo = command.starts_with("sudo") || command.contains("sudo ");
        
        if needs_sudo {
            if let Some(ref password) = self.sudo_password {
                return self.execute_with_sudo(command, password);
            } else {
                return CommandResult {
                    success: false,
                    stdout: String::new(),
                    stderr: "Sudo password required. Use /setsudo <password> to set your sudo password.".to_string(),
                    exit_code: -1,
                    needs_sudo: true,
                };
            }
        }
        
        self.execute_normal(command)
    }
    
    fn execute_normal(&self, command: &str) -> CommandResult {
        let shell_flag = "-c";
        
        let output = Command::new(&self.shell)
            .arg(shell_flag)
            .arg(command)
            .output();
        
        match output {
            Ok(out) => CommandResult {
                success: out.status.success(),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: out.status.code().unwrap_or(-1),
                needs_sudo: false,
            },
            Err(e) => CommandResult {
                success: false,
                stdout: String::new(),
                stderr: format!("Failed to execute: {}", e),
                exit_code: -1,
                needs_sudo: false,
            },
        }
    }
    
    fn execute_with_sudo(&self, command: &str, password: &str) -> CommandResult {
        let sudo_command = if command.starts_with("sudo ") {
            command.replacen("sudo ", "sudo -S ", 1)
        } else if command.starts_with("sudo") {
            command.replacen("sudo", "sudo -S", 1)
        } else {
            format!("sudo -S {}", command)
        };
        
        let mut child = Command::new(&self.shell)
            .arg("-c")
            .arg(&sudo_command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn command");
        
        if let Some(ref mut stdin) = child.stdin {
            writeln!(stdin, "{}", password).ok();
        }
        
        let output = child.wait_with_output().expect("Failed to wait for command");
        
        CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            needs_sudo: false,
        }
    }
    
    pub fn get_install_command(&self, app_name: &str) -> String {
        #[cfg(target_os = "arch")]
        {
            format!("sudo pacman -S {}", app_name)
        }
        #[cfg(target_os = "debian")]
        {
            format!("sudo apt install {}", app_name)
        }
        #[cfg(target_os = "macos")]
        {
            format!("brew install {}", app_name)
        }
        #[cfg(windows)]
        {
            format!("winget install {}", app_name)
        }
        #[cfg(not(any(target_os = "arch", target_os = "debian", target_os = "macos", windows)))]
        {
            format!("echo 'Unknown OS - please install {} manually'", app_name)
        }
    }
    
    pub fn check_package_manager(&self) -> &'static str {
        #[cfg(target_os = "arch")]
        {
            return "pacman";
        }
        #[cfg(target_os = "debian")]
        {
            return "apt";
        }
        #[cfg(target_os = "macos")]
        {
            return "brew";
        }
        #[cfg(windows)]
        {
            return "winget";
        }
        
        "unknown"
    }
}
