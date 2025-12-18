# Apex Kube Manager

A fast, native desktop application for managing Kubernetes clusters. Built with Tauri 2, Svelte 5, and Rust.

No Electron. No bundled Chromium. Just a ~15MB native app.

## Download

### macOS

Download the latest release from [GitHub Releases](https://github.com/tinaunglin/apex-kube-manager/releases):

| Chip | File |
|------|------|
| Apple Silicon (M1/M2/M3/M4) | `Apex.Kube.Manager_x.x.x_aarch64.dmg` |
| Intel | `Apex.Kube.Manager_x.x.x_x64.dmg` |

#### macOS Installation

1. Download the `.dmg` file for your chip
2. Open the DMG and drag **Apex Kube Manager** to your Applications folder
3. **First launch** (unsigned app):
   - Right-click the app and select **Open**
   - Click **Open** in the security dialog
   - You only need to do this once

Alternatively, bypass Gatekeeper via Terminal:
```bash
xattr -cr /Applications/Apex\ Kube\ Manager.app
```

### Linux

Download `.AppImage` or `.deb` from [GitHub Releases](https://github.com/tinaunglin/apex-kube-manager/releases).

### Windows

Download `.msi` or `.exe` from [GitHub Releases](https://github.com/tinaunglin/apex-kube-manager/releases).

## Requirements

- **kubectl** installed and in your PATH
- Valid kubeconfig file at `~/.kube/config`
- For exec-based auth (EKS, GKE, AKS): respective CLI tools installed (aws-cli, gcloud, az)

### Verify kubectl is working

```bash
kubectl cluster-info
kubectl get nodes
```

If these commands work, Apex Kube Manager will work.

## Features

### Cluster Management
- **One-click context switching** - Simple dropdown, app-wide refresh
- **Locked detail windows** - Open a pod detail, switch context, the window stays on the original cluster
- **External kubectx detection** - Switch context via kubectx/terminal, app picks it up automatically
- **Namespace filtering** - Global selector applies to all views

### Resource Views (17 types)
| Workloads | Network | Config | Storage | Cluster |
|-----------|---------|--------|---------|---------|
| Pods | Services | ConfigMaps | PersistentVolumes | Namespaces |
| Deployments | Ingresses | Secrets | PersistentVolumeClaims | Nodes |
| StatefulSets | NetworkPolicies | HPAs | | ServiceAccounts |
| DaemonSets | | | | |
| ReplicaSets | | | | |
| Jobs | | | | |
| CronJobs | | | | |

### Pod Operations
- **Exec terminal** - Shell into pods with selection (/bin/sh, /bin/bash, /bin/ash, /bin/zsh)
- **Logs viewer** - Stream logs with search, download, previous container logs
- **Delete pods** - With confirmation

### Deployment & StatefulSet Operations
- **Scale** - Adjust replica count with +/- controls
- **Restart** - Rolling restart via annotation patch
- **Detail view** - Overview, managed pods, events, YAML export

### Dashboard (Pulse)
- Cluster info (context, user, K8s version)
- Resource status cards (OK/FAIL counts)
- CPU/Memory capacity gauges

## Usage

1. Launch the app
2. It will automatically detect your kubeconfig and connect to the current context
3. Use the **context dropdown** (top-left) to switch clusters
4. Use the **namespace dropdown** to filter resources
5. Click any resource row to open its detail view
6. Use action buttons (Scale, Restart, Logs, Exec) for operations

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust + Tauri 2 |
| K8s Client | kube-rs (same lib used in Rust k8s controllers) |
| Frontend | Svelte 5 + TypeScript |
| Styling | TailwindCSS (custom dark theme) |
| Terminal | xterm.js + portable-pty |

### How Pod Exec Works

1. App spawns `kubectl exec -it <pod> -- <shell>` via portable-pty
2. PTY stdin/stdout streams over Tauri IPC channels
3. xterm.js renders the terminal in the frontend
4. Result: Native terminal experience, works with any auth method kubectl supports

## Build from Source

```bash
# Prerequisites
# - Node.js 18+
# - Rust 1.70+
# - Tauri CLI

# Clone
git clone https://github.com/tinaunglin/apex-kube-manager.git
cd apex-kube-manager

# Install dependencies
npm install

# Development (hot-reload)
npm run tauri dev

# Production build
npm run tauri build
# Output: src-tauri/target/release/bundle/
```

## Project Structure

```
apex-kube-manager/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── lib.rs       # App init & command registration
│   │   ├── commands.rs  # Tauri IPC commands
│   │   ├── kubernetes.rs # K8s API operations (kube-rs)
│   │   ├── pty.rs       # Terminal PTY handling
│   │   └── error.rs     # Error types
│   └── Cargo.toml
├── src/                 # Svelte frontend
│   ├── lib/
│   │   ├── components/  # UI components
│   │   │   ├── views/   # Resource list views
│   │   │   └── detail/  # Resource detail views
│   │   └── stores/      # Svelte stores (state)
│   └── App.svelte
└── package.json
```

## Troubleshooting

### "App is damaged" on macOS
Run this to remove quarantine flag:
```bash
xattr -cr /Applications/Apex\ Kube\ Manager.app
```

### App doesn't connect to cluster
1. Verify kubectl works: `kubectl get nodes`
2. Check kubeconfig path: `echo $KUBECONFIG` or `~/.kube/config`
3. For EKS/GKE/AKS: ensure CLI tools are in PATH

### Exec terminal doesn't work
1. Verify kubectl exec works: `kubectl exec -it <pod> -- /bin/sh`
2. Some minimal images only have `/bin/sh`, not `/bin/bash`

## Looking for Testers

If you use Kubernetes daily and have opinions about how these tools should work, I'd love your feedback.

Open an issue or reach out.

## License

[MIT](LICENSE)

## Contributing

Contributions welcome. Please open an issue first to discuss what you'd like to change.
