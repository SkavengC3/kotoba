# 言葉 (Kotoba)

A minimalist, high-performance Japanese spaced repetition (SRS) desktop application built with **Tauri v2**, **Rust**, and **SvelteKit**.

Designed for deep focus, keyboard-driven navigation, and JLPT N5/N4 mastery with an authentic Japanese editorial aesthetic (Muji / Washi paper design philosophy). Built to prepare for research internships in Japan (OIST).

---

## ✨ Features

- **Rust-Powered SRS Core:** Spaced-repetition scheduling algorithm with repetition penalties, penalty protections, and thread-safe in-memory state (`tauri::State<Mutex<CardDeck>>`).
- **SQLite Persistence (`rusqlite`):** Automatic local database recording reviews, user streaks, activity rhythm, and custom settings (daily quota, translations).
- **Studio-Quality Offline Audio:** Built-in Tokyo standard pitch-accent audio pronunciation for every vocabulary card. Plays automatically or on hotkey `<R>`, completely offline (zero latency, zero cloud dependency).
- **JLPT N5 Grammar Handbook:** Integrated editorial grammar guide with structural breakdowns, conjugation patterns, and contextual examples for key particles and sentence forms.
- **⚡ 60s Blitz Review Mode:** High-intensity rapid-fire testing mode with keyboard recall and score calculation to cement active recall.
- **📖 Comprehensive Vocabulary Dictionary:** Full lexicon browser with real-time kanji, kana, and romaji search, with instant audio replay buttons.
- **Japanese Editorial Aesthetic:**
  - Traditional warm paper palette (`#F7F5F0`), ink black (`#1A1816`), and vermilion seal red (`#C83226`).
  - Physical split-card design for clear cognitive separation between prompt and translation.
  - Kanji 2×5 today's spread matrix with real-time session progress and rhythm strips.
- **Keyboard-First Workflow:** Full control without touching the mouse (`Enter` to start, `Space`/`Enter` to flip, `1` / `2` / `3` to grade recall, `R` to replay audio, `Esc` to return to dashboard).
- **Ultra-Lightweight Desktop Architecture:** Powered by Tauri v2 with native WebKitGTK bindings, minimal memory footprint (~40 MB RAM), and instant cold boot.

---

## 📦 Downloads (v0.1.0)

Pre-built Linux binaries are available in [GitHub Releases](https://github.com/SkavengC3/kotoba/releases/latest):

- **AppImage (Universal Linux):** Download `Kotoba_0.1.0_amd64.AppImage`, make it executable, and run:
  ```bash
  chmod +x Kotoba_0.1.0_amd64.AppImage
  ./Kotoba_0.1.0_amd64.AppImage
  ```
- **Debian / Ubuntu (.deb):**
  ```bash
  sudo dpkg -i Kotoba_0.1.0_amd64.deb
  ```
- **Fedora / RHEL (.rpm):**
  ```bash
  sudo rpm -i Kotoba-0.1.0-1.x86_64.rpm
  ```

---

## 🛠 Tech Stack

- **Backend:** Rust 2021, Tauri v2 (`@tauri-apps/api`, `rusqlite`, `serde`, `serde_json`)
- **Frontend:** SvelteKit, TypeScript, Vite
- **Styling:** Custom CSS with Japanese editorial typography (`Noto Serif JP`, `Hiragino Mincho ProN`)
- **Audio:** Local 48kHz Tokyo standard pitch-accent voice assets

---

## 🚀 Building from Source

### Prerequisites

- [Rust & Cargo](https://rustup.rs/) (v1.75+)
- [Node.js](https://nodejs.org/) (v18+) & `npm`
- System libraries:
  - **Fedora / RHEL:**
    ```bash
    sudo dnf install webkit2gtk4.1-devel gtk3-devel librsvg2-devel libayatana-appindicator-gtk3-devel
    ```
  - **Debian / Ubuntu:**
    ```bash
    sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libayatana-appindicator3-dev
    ```

### Development

```bash
# Clone the repository
git clone https://github.com/SkavengC3/kotoba.git
cd kotoba

# Install frontend dependencies
npm install

# Run desktop app in development mode
npm run tauri dev
```

### Production Build

```bash
# Build frontend and compile native desktop bundle (AppImage, deb, rpm)
npm run tauri build
```

The output bundles will be located in `src-tauri/target/release/bundle/`.

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| <kbd>Enter</kbd> | Start today's lesson |
| <kbd>Space</kbd> / <kbd>Enter</kbd> | Flip flashcard |
| <kbd>1</kbd> | Grade: Повторить (Again) |
| <kbd>2</kbd> | Grade: Трудно (Hard) |
| <kbd>3</kbd> | Grade: Отлично (Good) |
| <kbd>R</kbd> (or <kbd>К</kbd>) | Replay native audio pronunciation |
| <kbd>Esc</kbd> | Return to Main Dashboard |

---

## 📜 License

MIT License. Designed and developed with focus and craft for OIST Japan preparation.
