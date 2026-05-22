# Refinery — AI Writing Assistant

> **Draft freely. Polish precisely. Maintain the human in the loop.**

Refinery is an offline-first, local desktop AI writing assistant built with **Tauri**, **Vue 3**, and **Rust**. 

Refinery was born from a simple realization: when writing deep reflections and creative drafts, the human focus should remain entirely on **thinking, concepts, and expression**. Unfortunately, constant dictionary checks, red squiggly lines, and mechanical warnings interrupt the creative flow state, while completely handing a draft over to an AI chat erases the unique human voice. 

Refinery solves this by separating creative drafting from precise, mechanical polishing.

---

## 💡 Core Philosophy

### 1. Thinking vs. Polish Mode
This is the core philosophy that separates Refinery from other writing applications. 
- **Thinking Mode (Drafting)**: All distraction-inducing dictionary and spelling checks are completely disabled. Focus solely on capturing concepts and maintaining flow state. Overlook typos, grammar, and sentence structure without visual interruptions.
- **Polish Mode (AI Review)**: When you are ready, switch to Polish Mode. The assisting AI analyzes your text and presents ultra-minimal inline recommendations in a flat, beautiful side panel.

### 2. Human-in-the-Loop
Refinery never hijacks your writing. Unlike standard chatbots that completely rewrite paragraphs and wash away your style, Refinery uses a precise **approval system**. You see each edit side-by-side (`original ➔ proposed`), selecting only what resonates with you. You are the author; the AI is the mechanical assistant.

---

## ✨ Features

- **Double-Mode Writing**: Switch between a distraction-free thinking canvas and a focused polishing panel.
- **Human-in-the-Loop Approval**: Review edits inline and accept/reject them with a single click.
- **Multi-page Support**: Seamlessly organize and manage multiple reflection pieces and drafts.
- **Page Version History**: Auto-saves Git-like snapshots every time you apply AI changes, allowing you to restore any page version at any time.
- **Side-by-Side Comparison Tool**: Paste any two versions of a draft to see character-level differences and a deep AI analysis of style, flow, and expression.
- **Command Palette**: Speed up your editing workflow with a native, keyboard-first command palette.
- **Vibrant Dark & Light Modes**: Beautiful, premium color systems designed for focused day and night writing.
- **Multiple AI Providers**: Seamless support for **OpenRouter** and **Groq** to select your favorite assisting voices.

---

## 🛠️ Tech Stack

Refinery is built for speed, safety, and a highly polished native desktop experience:
- **Frontend**: Highly performance-optimized **Vue.js 3** + **Pinia** + **Tiptap**.
- **Backend**: Safe, ultra-fast native **Rust** with a local **SQLite** database.
- **Desktop Shell**: **Tauri** for lightweight desktop software support.

---

## 🚀 Upcoming Roadmap

- [ ] **Smart Page-Type Checking**: Page-type-specific checks (e.g., checking if all arguments in a specialized "Royal Inquiry" page type remain closed/complete).
- [ ] **Selected Sentence Rewrite**: Highlight any sentence in the editor to ask the AI for custom vocabulary, clarity, or rewrite variations.

---

## Prerequisites

To run or build this project, you will need:
- [Node.js](https://nodejs.org/) (v18 or newer)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/setup/)

---

## Setup & Running Locally

1. **Install dependencies:**
   ```bash
   npm install
   ```

2. **Run the app in development mode:**
   ```bash
   npm run tauri dev
   ```
   *This command will compile the Rust backend and launch the desktop application.*


