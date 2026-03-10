# Kael - AI Assistant

> **Status: IN DEVELOPMENT** - v0.4.0

## Project Overview

Kael is a fully local AI Assistant built entirely in Rust with continuous learning:

1. **Director/PA** - General chat, scheduling, email, personal assistance
2. **Programmer** - Code assistance and programming help
3. **Vision** - Image analysis

### Key Features
- **100% Local** - No external API calls, no Ollama needed
- **Continuous Learning** - Each AI grows smarter over time via RAG/Lora
- **Baking System** - When knowledge gets large, bake into smarter models
- **Auto-download** - Downloads smallest Dolphin models from HuggingFace

### Tech Stack
- **Language**: Rust (100% - no Node.js)
- **LLM**: llama-gguf (pure Rust GGUF inference)
- **Database**: SQLite (one per AI for training)
- **GUI**: eframe/egui
- **Platform**: Cross-platform (Linux, Windows, macOS)

---

## Directory Structure

```
Kael/
├── apps/kael/           # Main application
│   ├── src/
│   │   ├── main.rs
│   │   ├── gui.rs
│   │   └── ai/
│   │       ├── llama.rs         # llama-gguf integration
│   │       ├── downloader.rs    # Auto-download models
│   │       └── training.rs     # Per-AI training system
│   └── target/
├── modals/              # AI Models & Training Data
│   ├── director/        # Director AI
│   │   ├── director.gguf
│   │   └── training.db  # Knowledge, interactions, loras
│   ├── programmer/      # Programmer AI
│   │   ├── programmer.gguf
│   │   └── training.db
│   └── vision/         # Vision AI
│       ├── vision.gguf
│       └── training.db
├── .profiles/           # Encrypted profiles (pending)
├── docs/
└── pkgbuild/
```

---

## What's Done ✅

### Core
- [x] GUI Interface (ChatGPT-style with sidebar)
- [x] Fully autonomous chat - just tell Kael what you want
- [x] Auto-detect intents (schedule, code, install, vision, email)
- [x] Auto-switch to appropriate AI based on context
- [x] Built-in terminal panel (click Terminal in sidebar)

### Local AI (No External Dependencies)
- [x] llama-gguf integration (pure Rust GGUF inference)
- [x] Auto-download models from HuggingFace
- [x] Models stored in `../modals/`
- [x] Works completely offline after download

### Per-AI Training System
- [x] Separate SQLite database for each AI
- [x] Knowledge base with confidence scores
- [x] Interaction history with feedback tracking
- [x] LoRA adapter management
- [x] Training sessions log
- [x] "Baking" when knowledge grows large (100+ items)
- [x] UI shows stats: knowledge count, unbaked/baked, sessions

### Terminal
- [x] Built-in terminal panel
- [x] Type commands directly - no prefix needed
- [x] Sudo password prompt in terminal

---

## What's Pending 📋

### High Priority
- [ ] Profile encryption system (.profiles)
- [ ] Calendar/email integration (PA Protocol)
- [ ] Bake button in UI to trigger model update
- [ ] Auto-bake when threshold reached

### Medium Priority
- [ ] Voice input
- [ ] Text-to-speech
- [ ] Settings panel in GUI
- [ ] Auto-save chat history

### Nice to Have
- [ ] Plugin system
- [ ] Multi-language support

---

## How to Use

### Run Kael
```bash
cd apps/kael
cargo run --release
```

### First Run
1. Click **"⬇️ Download Models"** in sidebar
2. Wait for download (Dolphin for Director/Programmer, LLaVA for Vision)
3. Start chatting!

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

### Training Stats (in sidebar)
Each AI shows:
- 📚 Knowledge items learned
- 🔥 Unbaked (ready to bake)
- ✅ Baked (incorporated into model)
- 📖 Sessions trained
- 🏷️ Topics learned

---

## How Learning Works

1. **Start Small** - Each AI starts with base Dolphin model
2. **Learn from Use** - Interactions stored in SQL with confidence scores
3. **RAG Context** - Knowledge injected into prompts automatically
4. **Baking** - When 100+ unbaked items, can "bake" knowledge
5. **Restart Cycle** - Bake = restart with smarter base model
6. **Repeat** - Continuously improves over time

---

## Configuration

Config stored in:
- **Linux**: `~/.local/share/com.kaelos.Kael/`
- **Windows**: `%APPDATA%\com.kaelos.Kael\`

---

## GitHub

- Repository: https://github.com/LeeTheOrc/Kael

---

## Version History

- **v0.4.0** - Local GGUF inference + per-AI training system + auto-download
- **v0.3.0** - Autonomous chat + built-in terminal
- **v0.2.0** - GUI interface
- **v0.1.0** - Initial project
