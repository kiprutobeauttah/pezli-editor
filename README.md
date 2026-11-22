# Pezli Editor

A native desktop text editor written entirely in Rust with egui/eframe. No web view, no JavaScript, no bundler.

## Structure

```
.
├── src/                Rust application (UI, editor, themes, settings)
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

- Modern, theme-consistent UI with locally stored Feather icons
- Tabs, line numbers, find and replace, undo/redo
- Open files and folders with a browsable file tree
- Settings in the editor area: themes, fonts, layout, custom themes
- Rearrangeable workspace: side bar and panel can move to either side
