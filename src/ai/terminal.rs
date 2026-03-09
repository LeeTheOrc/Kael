use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub struct Terminal {
    pub shell: String,
}

impl Terminal {
    pub fn new() -> Self {
        #[cfg(windows)]
        {
            Self { shell: "cmd.exe".to_string() }
        }
        #[cfg(not(windows))]
        {
            Self { shell: "bash".to_string() }
        }
    }
    
    pub fn execute(&self, command: &str) -> CommandResult {
        let shell_flag = if self.shell == "cmd.exe" { "/C" } else { "-c" };
        
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
            },
            Err(e) => CommandResult {
                success: false,
                stdout: String::new(),
                stderr: format!("Failed to execute: {}", e),
                exit_code: -1,
            },
        }
    }
    
    pub fn execute_interactive(&self, command: &str) -> CommandResult {
        // For interactive commands, run without capturing
        let shell_flag = if self.shell == "cmd.exe" { "/C" } else { "-c" };
        
        let output = Command::new(&self.shell)
            .arg(shell_flag)
            .arg(command)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output();
        
        match output {
            Ok(out) => CommandResult {
                success: out.status.success(),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: out.status.code().unwrap_or(-1),
            },
            Err(e) => CommandResult {
                success: false,
                stdout: String::new(),
                stderr: format!("Failed to execute: {}", e),
                exit_code: -1,
            },
        }
    }
    
    pub fn get_install_command(&self, app_name: &str) -> String {
        #[cfg(windows)]
        {
            // Windows - check for winget, chocolatey, or scoop
            format!("winget install {}", app_name)
        }
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
        #[cfg(not(any(windows, target_os = "arch", target_os = "debian", target_os = "macos")))]
        {
            format!("echo 'Unknown OS, please install {} manually'", app_name)
        }
    }
    
    pub fn check_package_manager(&self) -> String {
        #[cfg(windows)]
        {
            if Command::new("winget").arg("--version").output().is_ok() {
                return "winget".to_string();
            }
            if Command::new("choco").arg("--version").output().is_ok() {
                return "chocolatey".to_string();
            }
            return "unknown".to_string();
        }
        
        #[cfg(target_os = "arch")]
        {
            return "pacman".to_string();
        }
        
        #[cfg(target_os = "debian")]
        {
            return "apt".to_string();
        }
        
        #[cfg(target_os = "macos")]
        {
            if Command::new("brew").arg("--version").output().is_ok() {
                return "brew".to_string();
            }
            return "unknown".to_string();
        }
        
        "unknown".to_string()
    }
}
