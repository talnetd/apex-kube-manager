# Architecture

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust + Tauri 2 |
| K8s Client | kube-rs (same lib used in Rust k8s controllers) |
| Frontend | Svelte 5 + TypeScript |
| Styling | TailwindCSS (custom dark theme) |
| Terminal | xterm.js + portable-pty |

## Project Structure

```
apex-kube-manager/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── lib.rs          # App init & command registration
│   │   ├── commands.rs     # Tauri IPC commands (frontend ↔ backend)
│   │   ├── kubernetes.rs   # K8s API operations via kube-rs
│   │   ├── pty.rs          # Terminal PTY handling
│   │   └── error.rs        # Error types
│   ├── capabilities/       # Tauri permission configs
│   └── Cargo.toml
├── src/                    # Svelte frontend
│   ├── lib/
│   │   ├── components/
│   │   │   ├── views/      # Resource list views (17 types)
│   │   │   ├── detail/     # Resource detail views
│   │   │   └── ui/         # Reusable UI components
│   │   └── stores/         # Svelte stores (state management)
│   ├── App.svelte          # Main app component
│   └── main.ts             # Entry point
├── docs/                   # Documentation
└── package.json
```

## How It Works

### Tauri IPC

Frontend communicates with Rust backend via Tauri's IPC (invoke):

```
┌─────────────────┐         ┌─────────────────┐
│  Svelte Frontend │  IPC   │   Rust Backend  │
│                 │ ──────► │                 │
│  invoke('cmd')  │         │  #[tauri::cmd]  │
│                 │ ◄────── │                 │
│  Promise<T>     │         │  -> Result<T>   │
└─────────────────┘         └─────────────────┘
```

### Kubernetes Operations

All K8s operations go through `kube-rs`:

1. Frontend calls `invoke('get_pods', { namespace })`
2. Rust handler in `commands.rs` calls `kubernetes::list_pods()`
3. `kubernetes.rs` uses kube-rs client to query K8s API
4. Results serialized and returned to frontend

### Pod Exec (Terminal)

Pod exec uses a PTY-based approach instead of WebSocket:

```
┌─────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────┐
│ xterm.js │───►│ Tauri IPC   │───►│ portable-pty│───►│ kubectl │
│         │◄───│             │◄───│             │◄───│ exec    │
└─────────┘    └─────────────┘    └─────────────┘    └─────────┘
```

1. App spawns `kubectl exec -it <pod> -- <shell>` via portable-pty
2. PTY stdin/stdout streams over Tauri IPC channels
3. xterm.js renders the terminal in the frontend
4. Result: Native terminal experience, works with any auth method kubectl supports

### Context Switching

- Context changes write to `~/.kube/config` via kube-rs
- App polls kubeconfig for external changes (kubectx, terminal)
- Detail windows store their own context, unaffected by main window switches

### Window Management

- Main window: Single instance, shows resource lists
- Detail windows: Multiple instances, one per resource
- Terminal windows: Separate windows for pod exec sessions

## Key Design Decisions

1. **kubectl for exec**: Instead of implementing K8s exec protocol, we spawn kubectl. This ensures compatibility with all auth methods (exec-based, OIDC, certificates).

2. **kube-rs for everything else**: Direct API access is faster and more reliable for listing/watching resources.

3. **No Electron**: Tauri + WebView keeps the app under 20MB vs 150MB+ for Electron.

4. **Svelte 5**: Runes provide reactive state without boilerplate. Compiles to minimal JS.

5. **Custom window controls**: Native decorations disabled for consistent dark theme across platforms.
