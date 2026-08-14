# SanitAgent 🛡️

**SanitAgent** is a high-performance macOS desktop native prompt sanitizer HUD built with pure **Rust** and **Tauri v2**. It runs silently in the background and provides a floating, borderless "Whisper-style" pill HUD whenever you hit the global shortcut `Cmd + Shift + S`.

It strips secrets, API keys, memory addresses, timestamps, ANSI colors, and stack trace junk from your clipboard, replacing it with clean, privacy-safe text ready to paste into LLMs or chat tools.

---

## ✨ Key Features

- ⚡ **Global Shortcut (`Cmd + Shift + S`)**: Automatically reads raw clipboard text, sanitizes it, writes clean text back to the clipboard, and triggers the HUD.
- 🚀 **Two-Stage Sanitization Engine**:
  - **Stage 1 (Rule Cleaner <1ms)**: Strips ANSI escape codes, high-entropy secrets (OpenAI `sk-...`, AWS `AKIA...`, GitHub tokens, Bearer tokens, PEM private keys, Slack webhooks, DB URIs), ISO timestamps (`<TIME>`), hex memory pointers (`<ADDR>`), Base64/JWT data, framework stack trace frames (`node_modules`, `site-packages`, etc.), and collapses duplicate log lines (`Repeated Nx`).
  - **Stage 2 (Local LLM Distillation)**: Connects to a local Ollama model to extract core error context. Features a hard **15-second timeout** with automatic fallback to Stage 1 text.
- 📊 **Token Stats & Unified Diff**: BPE token counting (`142 → 48 tokens (-66%)`) and collapsible line-by-line unified diff preview (red for removed secrets/junk, green for kept context).
- 🪟 **Non-Activating Floating Pill HUD**: macOS `NSPanel` floating window positioned top-center that **never steals focus** from your terminal or IDE.

---

## 📋 Prerequisites

Before launching SanitAgent, ensure you have installed:

- **macOS** (macOS 12+ recommended)
- **Rust Toolchain**: `rustc` & `cargo` ([install Rust](https://rustup.rs/))
- **Node.js**: v18+ & `npm` ([install Node.js](https://nodejs.org/))

---

## 🚀 How to Launch

### 1. Clone & Install Dependencies

```bash
cd sanitagent

# Install frontend dependencies
npm install
```

### 2. Launch in Development Mode

Run the following command to start both the Vite dev server and the Tauri native app:

```bash
npm run tauri dev
```

> **Note:** The app will initialize in your system tray and launch the top-center floating HUD.

---

## 🎮 How to Use

1. **Copy text** containing logs, stack traces, or code snippets with secrets to your clipboard.
2. Press **`Cmd + Shift + S`** (or click **Trigger Sanitizer** in the macOS menu bar tray icon).
3. SanitAgent will:
   - Read your clipboard.
   - Run the two-stage sanitization pipeline.
   - Replace your OS clipboard with clean, sanitized text.
   - Pop up the top-center floating pill HUD showing token reduction stats (e.g. `142 → 48 tokens (-66%)`).
4. **Expand Diff Drawer**: Click **Diff** on the pill HUD to view the side-by-side / unified diff of what was removed vs kept.
5. **Dismiss HUD**: Press `Esc` or click `-` on the HUD pill. (The HUD also auto-dismisses after 6 seconds of inactivity).

---

## 📦 Build Release Binary

To compile an optimized production desktop binary:

```bash
npm run tauri build
```

The compiled binary and DMG installer will be created at:
```
src-tauri/target/release/bundle/dmg/SanitAgent_0.1.0_x64.dmg
```

You can also run the built binary directly:
```bash
./src-tauri/target/release/sanitagent
```

---

## 🤖 Optional: Local LLM Distillation (Stage 2)

If you have [Ollama](https://ollama.com) running locally:

```bash
# Run local model for Stage 2 distillation
ollama run qwen2.5:1.5b
```

SanitAgent will automatically detect Ollama at `http://127.0.0.1:11434`. If Ollama has no installed models, is unavailable, or inference takes longer than 15 seconds, SanitAgent falls back to Stage 1 rule-cleaned text.

---

## 🧪 Running Unit Tests

To run the Rust engine unit tests (Stage 1 rules, token stats, diff generator):

```bash
cd src-tauri
cargo test
```

---

## 📁 Project Architecture

```
sanitagent/
├── src/                          # React + TypeScript + Tailwind CSS Frontend HUD
│   ├── components/
│   │   ├── HUD.tsx               # Floating pill container & auto-dismiss controls
│   │   ├── DiffView.tsx          # Collapsible unified diff drawer
│   │   └── StatsBadge.tsx        # Token reduction & latency badge
│   └── App.tsx                   # Tauri event listener & IPC hooks
├── src-tauri/                    # Pure Rust Engine & Tauri v2 App
│   ├── src/
│   │   ├── sanitizer/
│   │   │   ├── stage1.rs         # Rule cleaner (<1ms deterministic regex rules)
│   │   │   ├── stage2.rs         # Local LLM distillation worker with 15s timeout
│   │   │   ├── tokens.rs         # BPE token counter & reduction stats
│   │   │   └── diff.rs           # Unified diff generator (`similar` crate)
│   │   ├── window.rs             # macOS NSPanel non-activating floating window setup
│   │   └── main.rs               # Global shortcut listener & IPC handlers
│   └── Cargo.toml
└── package.json
```

---

## 📄 License

[MIT](LICENSE)
