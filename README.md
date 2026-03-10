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

## Folder Structure

```
/home/leroy/Kael/                          # Kael Root
├── .vault/                                # Training Data (SQL/RAG/Lora)
│   ├── director/                          # Director AI training
│   │   └── training.db                    # Knowledge, interactions, loras
│   ├── programmer/                        # Programmer AI training
│   │   └── training.db
│   └── vision/                            # Vision AI training
│       └── training.db
├── modals/                                # AI Model Files (.gguf)
│   ├── director/                          # Director model
│   │   └── director.gguf
│   ├── programmer/                        # Programmer model
│   │   └── programmer.gguf
│   └── vision/                            # Vision model
│       └── vision.gguf
├── apps/                                  # Applications
│   └── kael/                              # Main App
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs
│       │   ├── gui.rs                     # Main GUI
│       │   ├── config.rs
│       │   ├── chat.rs
│       │   └── ai/                        # AI Modules
│       │       ├── llama.rs               # llama-gguf
│       │       ├── downloader.rs          # HuggingFace download
│       │       ├── training.rs            # Per-AI training
│       │       ├── database.rs
│       │       ├── vault.rs
│       │       ├── terminal.rs
│       │       ├── orchestrator.rs
│       │       ├── ollama.rs
│       │       └── search.rs
│       └── target/                         # Built binaries
├── .profiles/                             # Encrypted profiles (pending)
├── docs/                                  # Documentation
└── pkgbuild/                              # Package builds
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
- [x] Models stored in `modals/`
- [x] Works completely offline after download

### Per-AI Training System (.vault)
- [x] Separate SQLite database for each AI in `.vault/`
- [x] Training pipeline: SQL → RAG → LoRA → Bake
- [x] Importance tracking (trivial/important/critical)
- [x] Director decides what promotes through the pipeline
- [x] UI shows: SQL, RAG, LoRA, Baked counts
- [x] Suggestions when thresholds reached

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
cd /home/leroy/Kael/apps/kael
cargo run --release
```

Or run the built binary:
```bash
/home/leroy/Kael/apps/kael/target/debug/kael
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
- 💾 SQL items (trivial data)
- 📚 RAG items (important knowledge)
- 🎯 LoRA adapters
- ✅ Baked (incorporated into model)
- ⭐ Important / 🔸 Trivial breakdown
- 💡 Suggestions when thresholds reached

---

## How Learning Works

### Training Pipeline (SQL → RAG → LoRA → Bake)

1. **SQL** - All data starts here (trivial stuff stays here)
2. **RAG** - Important info promoted when SQL gets big (1000+ items)
3. **LoRA** - When RAG gets big (100+ items) → create LoRA adapter
4. **Bake** - When LoRA gets big (5+ adapters) → bake into model
5. **Director Decides** - The Director AI marks what's trivial vs important

### Learning Process
- AI learns from interactions stored in `.vault/` SQL
- Director marks importance (0=trivial, 1=important, 2=critical)
- Trivial data stays in SQL (doesn't clutter brain)
- Important stuff promotes to RAG for context
- Lots of RAG → create LoRA adapter
- Many LoRAs → bake into model and restart cycle

### Auto-Promotion
- 1000+ SQL items → prompt to promote to RAG
- 100+ RAG items → prompt to create LoRA
- 5+ LoRA adapters → prompt to bake

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

- **v0.4.0** - Local GGUF inference + per-AI training (SQL→RAG→LoRA→Bake) + auto-download
- **v0.3.0** - Autonomous chat + built-in terminal
- **v0.2.0** - GUI interface
- **v0.1.0** - Initial project

---

## See You Tomorrow! 🌙
