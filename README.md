# Kael - AI Assistant

> **Status: INITIALIZING** - Project structure being built

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
- **LLM Integration**: llama.cpp bindings (local models)
- **Database**: SQLite for RAG/chat history
- **Platform**: Cross-platform (Linux, Windows, macOS)

---

## Directory Structure

```
Kael/
├── .vault/              # RAG, LoRA files for each AI
│   ├── director/
│   │   ├── rag/         # Director's knowledge base
│   │   └── lora/        # Director's fine-tuned weights
│   ├── programmer/
│   │   ├── rag/
│   │   └── lora/
│   └── vision/
│       ├── rag/
│       └── lora/
├── .profiles/           # Encrypted email/calendar profiles
├── apps/
│   ├── kael/           # Main application source
│   │   └── src/
│   └── src/            # Other apps source
├── pkgbuild/
│   ├── debug/          # Debug builds
│   └── release/        # Release builds
├── modals/             # AI model files
│   ├── director/       # Director AI models
│   ├── programmer/     # Programmer AI models
│   └── vision/         # Vision AI models
├── docs/               # Documentation
└── src/                # Main Kael source
```

---

## Progress Log

### 2026-03-09
- [x] Created directory structure
- [x] Initial Cargo.toml with Rust dependencies
- [x] Basic config module (TOML-based)
- [x] Basic CLI chat interface (no AI yet)
- [x] GPG key created and added to GitHub
- [x] Git repository initialized and pushed to GitHub
- [ ] Ollama/llama.cpp integration (pending)
- [ ] Vision AI integration (pending)
- [ ] RAG system (pending)
- [ ] Profile encryption (pending)

---

## Current Code Status

### Implemented
- `src/main.rs` - Entry point with logging setup
- `src/config.rs` - Configuration management (TOML)
- `src/chat.rs` - CLI chat interface (text-based)

### Pending Implementation
- `src/ai/` - Ollama/llama.cpp integration
- `src/vault/` - RAG system with SQLite
- `src/profiles/` - Encrypted profile management

---

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run
cargo run
```

---

## Configuration

Config is stored in platform-specific directories:
- **Linux**: `~/.local/share/com.kaelos.Kael/config.toml`
- **Windows**: `%APPDATA%\com.kaelos.Kael\config.toml`

### Default Config
```toml
[models]
director_model = "dolphin3.0-mistral-7b-q4"
programmer_model = "dolphin3.0-coder-7b-q4"
vision_model = "llava-7b-q4"

[chat]
max_tokens = 2048
temperature = 0.7

[api]
ollama_url = "http://localhost:11434"
use_local = true
```

---

## Notes for Developers

1. **No Node.js** - Pure Rust project
2. **Cross-platform** - Uses `directories` crate for config paths
3. **Windows support** - Uses `winreg` for registry on Windows
4. **Local AI** - Designed to run with Ollama/llama.cpp locally
5. **Daily commits** - Push to GitHub daily as requested

---

## GitHub

- Repository: https://github.com/LeeTheOrc/Kael
- GPG Key: `E0AA316328B9D877`
