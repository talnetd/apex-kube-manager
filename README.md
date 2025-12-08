# Apex Kube Manager

A fast, native desktop application for managing Kubernetes clusters. Built with Tauri 2, Svelte 5, and Rust.

<!-- ![Screenshot](./screenshots/dashboard.png) -->

## Features

- **Dashboard (Pulse)** - Real-time cluster overview with resource status and CPU/Memory gauges
- **17 Resource Types** - Full support for Pods, Deployments, Services, ConfigMaps, Secrets, and more
- **Pod Terminal** - Exec into pods with shell selection (/bin/sh, /bin/bash, /bin/ash, /bin/zsh)
- **Pod Logs** - Stream and view container logs
- **YAML Viewer** - View resource definitions with syntax highlighting
- **Context Switching** - Switch between multiple Kubernetes clusters
- **Namespace Filtering** - Global namespace selector across all views
- **Sortable Tables** - Click column headers to sort resources
- **Auto-refresh** - 10-second refresh intervals with manual refresh option
- **Native Performance** - Rust backend with minimal resource usage

## Supported Resources

| Workloads | Network | Config | Storage | Cluster |
|-----------|---------|--------|---------|---------|
| Pods | Services | ConfigMaps | PersistentVolumes | Namespaces |
| Deployments | Ingresses | Secrets | PersistentVolumeClaims | Nodes |
| StatefulSets | NetworkPolicies | HPAs | | ServiceAccounts |
| DaemonSets | | | | |
| ReplicaSets | | | | |
| Jobs | | | | |
| CronJobs | | | | |

## Requirements

- macOS, Linux, or Windows
- [kubectl](https://kubernetes.io/docs/tasks/tools/) installed and configured
- Valid kubeconfig file (`~/.kube/config`)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/apex-kube-manager.git
cd apex-kube-manager

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### Pre-built Binaries

Coming soon.

## Tech Stack

- **Backend**: Rust + Tauri 2 + kube-rs
- **Frontend**: Svelte 5 + TypeScript
- **Styling**: TailwindCSS (dark theme)
- **Terminal**: xterm.js + portable-pty

## Development

```bash
# Install dependencies
npm install

# Start development server (hot-reload)
npm run tauri dev

# Build production binary
npm run tauri build
```

## Project Structure

```
apex-kube-manager/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── lib.rs       # App init & commands
│   │   ├── commands.rs  # Tauri IPC commands
│   │   ├── kubernetes.rs # K8s API operations
│   │   └── pty.rs       # Terminal PTY handling
│   └── Cargo.toml
├── src/                 # Svelte frontend
│   ├── lib/
│   │   ├── components/  # UI components
│   │   └── stores/      # State management
│   └── App.svelte
├── tailwind.config.js
└── package.json
```

## License

[MIT](LICENSE)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
