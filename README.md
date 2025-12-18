# Apex Kube Manager

A fast, native desktop application for managing Kubernetes clusters. Built with Tauri 2, Svelte 5, and Rust.

No Electron. No bundled Chromium. Just a ~15MB native app.

## Download

Download from [GitHub Releases](https://github.com/talnetd/apex-kube-manager/releases):

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `Apex.Kube.Manager_x.x.x_aarch64.dmg` |
| macOS (Intel) | `Apex.Kube.Manager_x.x.x_x64.dmg` |
| Linux | `.AppImage` or `.deb` |
| Windows | `.msi` or `.exe` |

### macOS First Launch

macOS blocks unsigned apps. On first launch:

1. **Right-click** the app in Applications
2. Select **Open**
3. Click **Open** in the dialog

Or remove quarantine via Terminal:
```bash
xattr -cr /Applications/Apex\ Kube\ Manager.app
```

## Requirements

- **kubectl** installed and in PATH
- Valid kubeconfig at `~/.kube/config`
- For cloud clusters: respective CLI (aws-cli, gcloud, az)

Verify setup:
```bash
kubectl cluster-info
```

## Features

### Resource Views

| Workloads | Network | Config | Storage | Cluster |
|-----------|---------|--------|---------|---------|
| Pods | Services | ConfigMaps | PVs | Namespaces |
| Deployments | Ingresses | Secrets | PVCs | Nodes |
| StatefulSets | NetworkPolicies | HPAs | | ServiceAccounts |
| DaemonSets | | | | |
| ReplicaSets | | | | |
| Jobs | | | | |
| CronJobs | | | | |

### Operations

- **Pod exec** - Shell into pods (sh, bash, ash, zsh)
- **Pod logs** - Stream with search, download, previous container
- **Scale** - Deployments and StatefulSets
- **Restart** - Rolling restart
- **Delete** - With confirmation

### Search

- **Cmd+K** - Global search across all resources
- **Per-view filter** - Filter current list by name

### Multi-cluster

- Context switching via dropdown
- Detail windows lock to their original context
- Auto-detects external context changes (kubectx)

## Usage

1. Launch the app
2. Select context (top-left dropdown)
3. Select namespace to filter
4. Click any row to open detail view
5. Use action buttons for operations

## Troubleshooting

### App doesn't connect

1. Verify: `kubectl get nodes`
2. Check kubeconfig: `~/.kube/config`
3. For EKS/GKE/AKS: ensure CLI tools in PATH

### Windows: Copy kubeconfig from WSL

To use your WSL kubeconfig with the Windows app:
```bash
# Run from WSL
mkdir -p /mnt/c/Users/$(whoami)/.kube && cp ~/.kube/config /mnt/c/Users/$(whoami)/.kube/config
```

### Exec doesn't work

1. Verify: `kubectl exec -it <pod> -- /bin/sh`
2. Some images only have `/bin/sh`

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - Tech stack, how it works
- [Development](docs/DEVELOPMENT.md) - Build from source, contributing

## License

[MIT](LICENSE)
