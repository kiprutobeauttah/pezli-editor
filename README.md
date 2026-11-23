# Pezli Editor

A native desktop text editor written entirely in Rust with egui/eframe. No web view, no JavaScript, no bundler.

## Structure

```
.
├── src/                Rust application (UI, editor, themes, settings, AI, notes)
├── assets/icons/       Feather icons downloaded from the web and stored locally
├── Cargo.toml
└── target/             Build output
```

## Run

```bash
cargo run --release
```

## Build

```bash
cargo build --release
```

The executable is at `target/release/pezli-editor`.

## Features

- Tabs, file/folder opening, recent files, find and replace
- 12 built-in themes plus custom themes, optional auto light/dark following the system
- Animated theme transitions, minimap, status bar with counts and zoom
- VS Code-style layout: drag dividers to resize panels, drag `::` grips to dock panels left/right/bottom or reorder tabs; sizes persist
- Session restore with autosave (open tabs, cursor, view state)
- Command palette, workspace-wide file search, drag-and-drop files
- Toast notifications
- Notes: quick capture, daily Markdown notes, Markdown preview with interactive checklists, timestamps
- AI assistant panel (explain, improve, checklist, summarize) using any OpenAI-compatible chat endpoint

## Shortcuts

| Shortcut | Action |
| --- | --- |
| `Ctrl+P` | Command palette |
| `Ctrl+N` / `Ctrl+O` / `Ctrl+S` | New / open / save |
| `Ctrl+Shift+S` | Save as |
| `Ctrl+F` / `Ctrl+Shift+F` | Find / search workspace files |
| `Ctrl+B` / `Ctrl+T` | Toggle side bar / switch theme |
| `Ctrl+Shift+Q` | Quick capture |
| `Ctrl+Shift+D` | New daily note |
| `Ctrl+Shift+M` | Markdown preview |
| `Ctrl+Alt+A` | AI assistant |
| `Ctrl++` / `Ctrl+-` | Zoom in / out |

Configure the notes folder and AI endpoint/key under Settings → Notes and Settings → AI.
