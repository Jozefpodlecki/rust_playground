Rust Playground App Architecture

🛠️ Tech Stack
Frontend: React + TypeScript (with Chakra UI or alternative)

Backend: Rust (Tauri shell)

Markdown Rendering: For exercises and examples

User Flow Overview
Launch App

User sees a Welcome Screen

Options:

Start New Exercise

Continue Last Exercise

Exercise Page

Displays individual coding exercises

Content is Markdown-based

Uses a Markdown renderer (e.g., @chakra-ui/markdown-renderer or react-markdown)

Includes code editor (possibly Monaco or CodeMirror)

Tracks progress (e.g., locally or via SQLite)

Examples Page

Separate section for example code snippets

Also rendered from Markdown

Allows browsing and searching through categorized examples