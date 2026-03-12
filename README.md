# Kael - AI Assistant

> **Status: IN DEVELOPMENT** - v0.4.0

## Project Overview

Kael is a fully local AI Assistant built entirely in Rust with continuous learning. Think of Kael as a single AI organism:

- 🧠 **Brain** - Two parts working together:
  - 🎯 **Director** - Decision making, scheduling, conversation flow
  - 💻 **Programmer** - Logic, code, problem solving
- 👁️ **Vision** - Eyes (image analysis)
- 🎤 **Ears** - Voice input (planned)
- 🗣️ **Mouth** - Text-to-speech (planned)

### Key Features
- **100% Local** - No external API calls, runs completely offline
- **Continuous Learning** - Grows smarter over time via SQL → RAG → LoRA → Bake
- **Baking System** - When knowledge gets large, bake into smarter models
- **Auto-download** - Downloads models from HuggingFace

### Tech Stack
- **Language**: Rust (100% - no Node.js)
- **LLM**: llama-gguf (pure Rust GGUF inference)
- **Database**: SQLite for training
- **GUI**: eframe/egui
- **Platform**: Cross-platform (Linux, Windows, macOS)

---

## Folder Structure

```
/home/leroy/Kael/                          # Kael Root
├── .vault/                                # Training Data
│   ├── director/                          # Brain training
│   ├── programmer/                        # Programmer training
│   └── vision/                           # Vision training
├── modals/                                # AI Models (.gguf files)
│   ├── director/director.gguf
│   ├── programmer/programmer.gguf
│   └── vision/vision.gguf
├── apps/kael/                             # Main Application
│   ├── src/
│   │   ├── main.rs
│   │   ├── gui.rs
│   │   └── ai/
│   └── target/debug/kael                 # Built binary
├── .profiles/                             # Encrypted profiles (pending)
├── docs/
└── pkgbuild/
```

---

## What's Done ✅

### Core
- [x] GUI Interface - ChatGPT-style with 3 panels
- [x] Left Panel (20%) - Chat/Calendar/Settings navigation
- [x] Middle Panel (60%) - Chat + Terminal at bottom
- [x] Right Panel (20%) - Tasks/Projects + Training stats
- [x] Fully autonomous chat - just tell Kael what you need

### Chat Interface
- [x] Chat bubbles like ChatGPT/Gemini
- [x] User messages (green bubbles, right)
- [x] Kael responses (dark bubbles, left)
- [x] Kael handles everything through chat

### Local AI (No External Dependencies)
- [x] llama-gguf integration (pure Rust GGUF)
- [x] Auto-download models from HuggingFace
- [x] Models stored in `modals/`

### Training System
- [x] Separate SQLite database per AI part in `.vault/`
- [x] Pipeline: SQL → RAG → LoRA → Bake
- [x] Importance tracking (trivial/important/critical)
- [x] UI shows training progress

### Terminal
- [x] Always visible at bottom of middle panel (20%)
- [x] Run commands directly
- [x] Sudo password support

---

## What's Pending 📋

### High Priority
- [ ] Vision support (needs different backend)
- [ ] Profile encryption (.profiles)
- [ ] Calendar/email integration
- [ ] Bake button in UI
- [ ] Voice input (Ears)
- [ ] Text-to-speech (Mouth)

### Medium Priority
- [ ] Settings panel
- [ ] Auto-save chat history
- [ ] Plugin system

---

## How to Use

### Run Kael
```bash
cd /home/leroy/Kael
./apps/kael/target/debug/kael
```

### First Run
1. Click **"⬇️ Download Models"** in sidebar
2. Wait for download (~1GB per model)
3. Start chatting!

### Chat Examples
```
"Schedule a meeting tomorrow at 3pm"
"Write me a Python function to parse JSON"
"Install Firefox"
"List all files in my home directory"
```

---

## How Learning Works

### Training Pipeline: SQL → RAG → LoRA → Bake

1. **SQL** - All interactions stored here (trivial data stays)
2. **RAG** - Important info promoted (1000+ items)
3. **LoRA** - Create adapter (100+ RAG items)
4. **Bake** - Incorporate into model (5+ LoRAs)

### Learning Process
- Kael learns from every conversation
- Important knowledge gets promoted through pipeline
- Trivial stuff stays in SQL (keeps brain clean)
- Periodic baking = restart cycle with smarter base

---

## GitHub

- Repository: https://github.com/LeeTheOrc/Kael

---

## Version History

- **v0.4.0** - New UI layout, chat bubbles, per-AI training, terminal always visible
- **v0.3.0** - Autonomous chat + built-in terminal
- **v0.2.0** - GUI interface
- **v0.1.0** - Initial project
