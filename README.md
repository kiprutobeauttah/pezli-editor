# Pezli Editor

A native desktop text editor built with Tauri and plain JavaScript. No web framework, no bundler, no npm build step.

## Layout

```
src-tauri/
├── frontend/              # UI (plain HTML/CSS/JS, loaded directly by Tauri)
│   ├── index.html
│   ├── styles.css
│   ├── app.js
│   └── vendor/feather.min.js   # Feather icons (downloaded from the web, vendored locally)
├── src/                   # Rust application + file I/O commands
├── icons/                 # App icons
├── capabilities/         # Tauri permissions
├── tauri.conf.json
└── Cargo.toml
```

## Run

```bash
cd src-tauri
cargo tauri dev
```

## Build installers

```bash
cd src-tauri
cargo tauri build
```

Bundles are written to `src-tauri/target/release/bundle/`.

## Workspace

The side bar, activity bar and bottom panel can be dragged to any edge, resized with the splitters, or moved through the Layout menu / View menu. Tabs can be dragged to reorder. Layout choices are remembered between launches.
