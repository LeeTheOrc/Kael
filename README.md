# Kael - AI Assistant

> **Status: IN DEVELOPMENT** - v0.3.0

## Project Overview

Kael is a 2-part AI Assistant built entirely in Rust:
1. **Director/PA** - General chat, personal assistant, task management
2. **Programmer** - Code assistance and programming help

### AI Models (planned)
- Primary: Dolphin 3.0 (smallest with reasoning)
- Secondary: Dolphin 3.0 Coder variant
- Vision: LLaVA for image analysis

### Tech Stack
- **Language**: Rust (100% - no Node.js)
- **LLM Integration**: Ollama (local models)
- **Database**: SQLite for RAG/chat history
- **GUI**: eframe/egui
- **Platform**: Cross-platform (Linux, Windows, macOS)

---

## Directory Structure

```
Kael/
├── .vault/              # RAG, LoRA files for each AI
│   ├── director/rag/   # Director's knowledge base
│   ├── director/lora/  # Director's fine-tuned weights
│   ├── programmer/rag/
│   ├── programmer/lora/
│   └── vision/rag/,lora/
├── .profiles/           # Encrypted email/calendar profiles (pending)
├── apps/
├── pkgbuild/
├── modals/             # AI model files (.gguf)
├── docs/
└── src/               # Main source
```

---

## What's Done ✅

### Core
- [x] GUI Interface (ChatGPT-style with sidebar)
- [x] Fully autonomous chat - just tell Kael what you want
- [x] Auto-detect intents (schedule, code, install, vision, email)
- [x] Auto-switch to appropriate AI based on context

### Terminal
- [x] Built-in terminal panel (click Terminal in sidebar)
- [x] Type commands directly - no prefix needed
- [x] Sudo password prompt in terminal

### Database
- [x] SQLite for persistent storage
- [x] Chat history
- [x] RAG knowledge base (/learn, /recall)
- [x] LoRA config management
- [x] Database stats

### AI Integration
- [x] Ollama client (connects to local Ollama)
- [x] Demo mode when Ollama not running
- [x] Vision support (/image command)

---

## What's Pending 📋

### High Priority
- [ ] Connect real AI (needs Ollama installed)
- [ ] Profile encryption system (.profiles)
- [ ] Calendar/email integration (PA Protocol)

### Medium Priority
- [ ] llama.cpp direct integration (instead of Ollama)
- [ ] Auto-save chat history
- [ ] Settings panel in GUI

### Nice to Have
- [ ] Voice input
- [ ] Text-to-speech
- [ ] Plugin system
- [ ] Multi-language support

---

## Current Code Status

### Implemented Files
- `src/main.rs` - Entry point
- `src/gui.rs` - Main GUI with autonomous chat + terminal
- `src/config.rs` - Configuration management
- `src/ai/mod.rs` - AI modules
- `src/ai/database.rs` - SQLite database
- `src/ai/vault.rs` - RAG/knowledge system
- `src/ai/terminal.rs` - Terminal execution
- `src/ai/terminal_gui.rs` - PTY terminal (not yet wired)

---

## How to Use

### Running Kael
```bash
cargo run --release
```

### Natural Chat Examples
```
"Schedule a meeting tomorrow at 3pm"
"Write me a Python function to parse JSON"
"Install Firefox"
"What's in this image?" (then use /image path)
"List all files in my home directory"
```

### Sidebar Modes
- 🎯 **Director/PA** - General chat, schedules, emails
- 💻 **Programmer** - Code help
- 👁️ **Vision** - Image analysis
- 📟 **Terminal** - Run shell commands

---

## Setup AI

1. Install Ollama: https://ollama.ai
2. Pull models:
```bash
ollama pull dolphin3.0-mistral
ollama pull dolphin3.0-coder
ollama pull llava
```
3. Restart Kael

---

## Configuration

Config stored in:
- **Linux**: `~/.local/share/com.kaelos.Kael/`
- **Windows**: `%APPDATA%\com.kaelos.Kael\`

Database: `kael.db`

---

## GitHub

- Repository: https://github.com/LeeTheOrc/Kael
- GPG Key: `E0AA316328B9D877`

---

## Version History

- **v0.3.0** - Autonomous chat + built-in terminal
- **v0.2.0** - GUI interface
- **v0.1.0** - Initial project
