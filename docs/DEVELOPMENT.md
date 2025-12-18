# Development Guide

## Prerequisites

- **Node.js** 18+
- **Rust** 1.70+
- **Tauri CLI** (`cargo install tauri-cli`)

### Platform-specific

**macOS:**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Windows:**
- Visual Studio Build Tools with C++ workload
- WebView2 (usually pre-installed on Windows 10/11)

## Setup

```bash
# Clone
git clone https://github.com/talnetd/apex-kube-manager.git
cd apex-kube-manager

# Install frontend dependencies
npm install
```

## Development

```bash
# Run in development mode (hot-reload)
npm run tauri dev
```

This will:
1. Start Vite dev server for frontend
2. Compile Rust backend
3. Launch the app with hot-reload

### Frontend only

```bash
npm run dev
```

Runs just the Vite dev server (useful for UI work without full app).

### Type checking

```bash
npm run check
```

## Building

### Development build

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

### Production build

```bash
npm run tauri build
```

Output locations:
- **macOS**: `src-tauri/target/release/bundle/dmg/`
- **Linux**: `src-tauri/target/release/bundle/deb/`, `appimage/`
- **Windows**: `src-tauri/target/release/bundle/msi/`, `nsis/`

## Project Commands

| Command | Description |
|---------|-------------|
| `npm run dev` | Frontend dev server only |
| `npm run build` | Build frontend |
| `npm run check` | TypeScript type check |
| `npm run tauri dev` | Full app dev mode |
| `npm run tauri build` | Production build |

## Code Structure

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed project structure.

### Adding a new resource type

1. **Backend** (`src-tauri/src/kubernetes.rs`):
   - Add `ResourceInfo` struct
   - Add `list_resources()` function

2. **Commands** (`src-tauri/src/commands.rs`):
   - Add `#[tauri::command]` function

3. **Register** (`src-tauri/src/lib.rs`):
   - Add to `generate_handler![]`

4. **Frontend store** (`src/lib/stores/kubernetes.ts`):
   - Add interface, store, and load function

5. **Component** (`src/lib/components/views/`):
   - Create `ResourceList.svelte`

6. **Router** (`src/lib/components/MainPanel.svelte`):
   - Add route

## Debugging

### Frontend

Browser devtools available via `Cmd+Option+I` (macOS) or `Ctrl+Shift+I` (Windows/Linux) in dev mode.

### Backend

```bash
RUST_LOG=debug npm run tauri dev
```

### Tauri IPC

Enable verbose logging:
```bash
RUST_LOG=tauri=debug npm run tauri dev
```
