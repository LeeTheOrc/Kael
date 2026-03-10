use once_cell::sync::Lazy;
use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;

static TERMINAL: Lazy<Mutex<Option<TerminalState>>> = Lazy::new(|| Mutex::new(None));

pub struct TerminalState {
    pty_pair: PtyPair,
    writer: Box<dyn Write + Send>,
}

pub struct TerminalManager;

impl TerminalManager {
    pub fn new() -> Self {
        Self
    }

    pub fn start(&self, rows: u16, cols: u16) -> Result<(), String> {
        let mut terminal = TERMINAL.lock();

        if terminal.is_some() {
            return Ok(()); // Already running
        }

        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let mut cmd = CommandBuilder::new("bash");
        cmd.env("TERM", "xterm-256color");

        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to get writer: {}", e))?;

        *terminal = Some(TerminalState {
            pty_pair: pair,
            writer,
        });

        Ok(())
    }

    pub fn write(&self, data: &str) -> Result<(), String> {
        let terminal = TERMINAL.lock();

        if let Some(ref mut term) = *terminal {
            term.writer
                .write_all(data.as_bytes())
                .map_err(|e| format!("Write error: {}", e))?;
            term.writer
                .flush()
                .map_err(|e| format!("Flush error: {}", e))?;
            Ok(())
        } else {
            Err("Terminal not started".to_string())
        }
    }

    pub fn is_running(&self) -> bool {
        TERMINAL.lock().is_some()
    }

    pub fn stop(&self) {
        let mut terminal = TERMINAL.lock();
        *terminal = None;
    }

    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), String> {
        let terminal = TERMINAL.lock();

        if let Some(ref term) = *terminal {
            term.pty_pair
                .master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|e| format!("Resize error: {}", e))?;
            Ok(())
        } else {
            Err("Terminal not started".to_string())
        }
    }
}
