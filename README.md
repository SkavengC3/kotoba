# 言葉 (Kotoba)

A minimalist, high-performance Japanese spaced repetition (SRS) desktop application built with **Tauri v2**, **Rust**, and **SvelteKit**.

Designed for deep focus, keyboard-driven navigation, and JLPT N5/N4 mastery with an authentic Japanese editorial aesthetic (Muji / Washi paper design philosophy).

---

## ✨ Features

- **Rust-Powered SRS Core:** Spaced-repetition scheduling algorithm with repetition penalties, penalty protections, and thread-safe in-memory state (`tauri::State<Mutex<CardDeck>>`).
- **Japanese Editorial Aesthetic:**
  - Traditional warm paper palette (`#F7F5F0`), ink black (`#1A1816`), and vermilion seal red (`#C83226`).
  - Physical split-card design for clear cognitive separation between prompt and translation.
  - Kanji 2×5 today's spread matrix with real-time session progress and rhythm strips.
- **Keyboard-First Workflow:** Full control without touching the mouse (`Enter` to start, `Space`/`Enter` to flip, `1` / `2` / `3` to grade recall, `Esc` to return to dashboard).
- **Lightweight Desktop Architecture:** Powered by Tauri v2 with native WebKitGTK bindings, minimal memory footprint, and instant cold boot.

---

## 🛠 Tech Stack

- **Backend:** Rust 2021, Tauri v2 (`@tauri-apps/api`, `serde`, `serde_json`)
- **Frontend:** SvelteKit, TypeScript, Vite
- **Styling:** Custom CSS with Japanese editorial typography (`Noto Serif JP`, `Hiragino Mincho ProN`)

---

## 🚀 Getting Started

### Prerequisites

- [Rust & Cargo](https://rustup.rs/) (v1.75+)
- [Node.js](https://nodejs.org/) (v18+) & `npm`
- System libraries (Linux / Fedora):
  ```bash
  sudo dnf install webkit2gtk4.1-devel gtk3-devel librsvg2-devel libayatana-appindicator-gtk3-devel
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

---

## 📜 License

MIT License. Designed and developed with focus and craft.
