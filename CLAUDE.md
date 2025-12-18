# Apex Kube Manager

A native desktop Kubernetes cluster management application built with Tauri 2 + Svelte 5 + TailwindCSS.

## Tech Stack

- **Backend**: Rust with Tauri 2
- **Frontend**: Svelte 5 with TypeScript
- **Styling**: TailwindCSS 3 with custom dark theme
- **Kubernetes**: `kube-rs` crate for K8s API interactions
- **Terminal**: xterm.js for pod exec

## Project Structure

```
apex-kube-manager/
├── src-tauri/                    # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── lib.rs               # App initialization & command registration
│   │   ├── commands.rs          # Tauri IPC commands exposed to frontend
│   │   ├── kubernetes.rs        # K8s client operations (all resources)
│   │   └── error.rs             # Error types and handling
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri window/build configuration
│
├── src/                          # Frontend (Svelte 5)
│   ├── lib/
│   │   ├── components/
│   │   │   ├── StartupScreen.svelte # Initialization loading screen
│   │   │   ├── Sidebar.svelte   # Categorized navigation (Workloads, Network, Config, Storage, Cluster)
│   │   │   ├── Header.svelte    # Context/namespace dropdowns, refresh button
│   │   │   ├── MainPanel.svelte # View router for all resource views
│   │   │   ├── Terminal.svelte  # xterm.js pod exec terminal
│   │   │   ├── LogViewer.svelte # Pod logs viewer with auto-refresh
│   │   │   ├── ui/              # Reusable UI components
│   │   │   └── views/           # All resource listing views (20+ components)
│   │   └── stores/
│   │       ├── startup.ts       # App initialization state
│   │       ├── kubernetes.ts    # All K8s state (contexts, all resources, metrics)
│   │       └── navigation.ts    # UI navigation state
│   ├── styles/app.css           # TailwindCSS imports + global styles
│   ├── App.svelte               # Root component
│   └── main.ts                  # Svelte mount entry point
│
├── tailwind.config.js           # Dark theme color palette
└── package.json                 # Frontend dependencies
```

## Commands

```bash
# Install dependencies
npm install

# Development mode (hot-reload frontend + Tauri backend)
npm run tauri dev

# Production build (creates native binary)
npm run tauri build
```

## Implemented Resource Views

All resource views are fully functional with listing, namespace filtering (where applicable), and auto-refresh.

### Workloads
| Resource | View File | Namespace Filter | Key Columns |
|----------|-----------|------------------|-------------|
| Pods | `PodList.svelte` | Yes | Name, Status, Ready, Restarts, Age, Node |
| Deployments | `DeploymentList.svelte` | Yes | Name, Ready, Up-to-date, Available, Age |
| StatefulSets | `StatefulSetList.svelte` | Yes | Name, Ready, Replicas, Service, Age |
| DaemonSets | `DaemonSetList.svelte` | Yes | Name, Desired, Current, Ready, Up-to-date, Age |
| ReplicaSets | `ReplicaSetList.svelte` | Yes | Name, Desired, Current, Ready, Owner, Age |
| Jobs | `JobList.svelte` | Yes | Name, Completions, Duration, Status, Age |
| CronJobs | `CronJobList.svelte` | Yes | Name, Schedule, Suspend, Active, Last Schedule, Age |

### Network
| Resource | View File | Namespace Filter | Key Columns |
|----------|-----------|------------------|-------------|
| Services | `ServiceList.svelte` | Yes | Name, Type, ClusterIP, ExternalIP, Ports, Age |
| Ingresses | `IngressList.svelte` | Yes | Name, Class, Hosts, Address, Ports, Age |
| NetworkPolicies | `NetworkPolicyList.svelte` | Yes | Name, Pod Selector, Policy Types, Age |

### Config
| Resource | View File | Namespace Filter | Key Columns |
|----------|-----------|------------------|-------------|
| ConfigMaps | `ConfigMapList.svelte` | Yes | Name, Data (key count), Age |
| Secrets | `SecretList.svelte` | Yes | Name, Type, Data (key count), Age |
| HPAs | `HPAList.svelte` | Yes | Name, Reference, Targets, Min/Max, Replicas, Age |

### Storage
| Resource | View File | Namespace Filter | Key Columns |
|----------|-----------|------------------|-------------|
| PersistentVolumes | `PVList.svelte` | No (cluster-scoped) | Name, Capacity, Access Modes, Reclaim, Status, Claim, StorageClass, Age |
| PersistentVolumeClaims | `PVCList.svelte` | Yes | Name, Status, Volume, Capacity, Access Modes, StorageClass, Age |

### Cluster
| Resource | View File | Namespace Filter | Key Columns |
|----------|-----------|------------------|-------------|
| Namespaces | `NamespaceList.svelte` | No (cluster-scoped) | Name, Status, Age |
| Nodes | `NodeList.svelte` | No (cluster-scoped) | Name, Status, Roles, Version, Internal IP, OS/Runtime, Age |
| ServiceAccounts | `ServiceAccountList.svelte` | Yes | Name, Secrets, Age |

## Dashboard (Pulse View)

The Dashboard (`Dashboard.svelte`) shows:
- **Cluster Info**: Context, Cluster name, User, K8s version
- **Resource Status Cards**: Shows OK:FAIL counts for workloads (Pods, Deployments, StatefulSets, DaemonSets, ReplicaSets, Jobs)
- **Resource Count Cards**: Simple counts for other resources (CronJobs, Services, ConfigMaps, Secrets, etc.)
- **CPU/Memory Gauges**: Animated ring gauges showing cluster resource capacity (allocatable vs capacity)

## Sidebar Navigation Structure

Flat list with section headers (no nested accordions):

```
Pulse (Dashboard)
─────────────────
WORKLOADS
  Pods
  Deployments
  StatefulSets
  DaemonSets
  ReplicaSets
  Jobs
  CronJobs
─────────────────
NETWORK
  Services
  Ingresses
  Network Policies
─────────────────
CONFIG
  ConfigMaps
  Secrets
  HPAs
─────────────────
STORAGE
  Persistent Volumes
  Persistent Volume Claims
─────────────────
CLUSTER
  Namespaces
  Nodes
  Service Accounts
```

## Tauri Commands (Rust → Frontend IPC)

All commands defined in `src-tauri/src/commands.rs`:

### Startup Commands
| Command | Returns | Description |
|---------|---------|-------------|
| `check_kubeconfig` | `String` | Check kubeconfig exists, returns path |
| `validate_kubeconfig` | `()` | Parse and validate kubeconfig |
| `get_context_names` | `Vec<String>` | Get list of context names |
| `test_cluster_connection` | `()` | Test connection to current cluster |

### Context/Namespace Commands
| Command | Parameters | Returns |
|---------|------------|---------|
| `get_contexts` | - | `Vec<KubeContext>` |
| `get_current_context` | - | `String` |
| `switch_context` | `context_name: String` | `()` |
| `get_namespaces` | - | `Vec<String>` |

### Resource Commands (all support `namespace: Option<String>`)
| Command | Returns |
|---------|---------|
| `get_pods` | `Vec<PodInfo>` |
| `get_deployments` | `Vec<DeploymentInfo>` |
| `get_statefulsets` | `Vec<StatefulSetInfo>` |
| `get_daemonsets` | `Vec<DaemonSetInfo>` |
| `get_replicasets` | `Vec<ReplicaSetInfo>` |
| `get_jobs` | `Vec<JobInfo>` |
| `get_cronjobs` | `Vec<CronJobInfo>` |
| `get_services` | `Vec<ServiceInfo>` |
| `get_ingresses` | `Vec<IngressInfo>` |
| `get_network_policies` | `Vec<NetworkPolicyInfo>` |
| `get_configmaps` | `Vec<ConfigMapInfo>` |
| `get_secrets` | `Vec<SecretInfo>` |
| `get_hpas` | `Vec<HPAInfo>` |
| `get_pvcs` | `Vec<PersistentVolumeClaimInfo>` |
| `get_service_accounts` | `Vec<ServiceAccountInfo>` |

### Cluster-Scoped Commands (no namespace filter)
| Command | Returns |
|---------|---------|
| `get_pvs` | `Vec<PersistentVolumeInfo>` |
| `get_namespaces_info` | `Vec<NamespaceInfo>` |
| `get_nodes` | `Vec<NodeInfo>` |

### Metrics Commands
| Command | Parameters | Returns |
|---------|------------|---------|
| `get_cluster_metrics` | - | `ClusterMetrics` |
| `get_pulse_metrics` | `namespace: Option<String>` | `PulseMetrics` |

### Pod Operations
| Command | Parameters | Returns |
|---------|------------|---------|
| `get_pod_logs` | `namespace, pod_name, container?, tail_lines?` | `String` |
| `delete_pod` | `namespace, pod_name` | `()` |
| `exec_pod` | `namespace, pod_name, container?` | `String` (placeholder) |

## Frontend Stores (src/lib/stores/)

### kubernetes.ts
All resource stores and load functions:
```typescript
// Stores
contexts, currentContext, namespaces, selectedNamespace
pods, deployments, statefulsets, daemonsets, replicasets, jobs, cronjobs
services, ingresses, networkPolicies
configmaps, secrets, hpas
pvs, pvcs
namespacesInfo, nodes, serviceAccounts
clusterMetrics, pulseMetrics
isLoading, error, connectionStatus

// Load functions (all async)
loadContexts(), switchContext(name), loadNamespaces()
loadPods(ns?), loadDeployments(ns?), loadStatefulSets(ns?), ...
loadPulseMetrics(ns?), loadClusterMetrics()
deletePod(ns, name), getPodLogs(ns, name, container?, lines?)
```

### navigation.ts
```typescript
currentView: View  // 'dashboard' | 'pods' | 'deployments' | ... (all 17 views)
sidebarCollapsed: boolean
```

## UI Theme (TailwindCSS)

Custom dark theme in `tailwind.config.js`:
```javascript
colors: {
  bg: {
    primary: '#0a0a0a',    // Main background
    secondary: '#111111',  // Sidebar, cards
    tertiary: '#1a1a1a',   // Hover states
    card: '#141414',       // Card backgrounds
  },
  border: { subtle: '#222222' },
  accent: {
    primary: '#00d4aa',    // Teal
    warning: '#f59e0b',    // Amber
    error: '#ef4444',      // Red
    success: '#22c55e',    // Green
  },
  text: {
    primary: '#ffffff',
    secondary: '#888888',
    muted: '#555555',
  }
}
```

## Key Patterns

### Adding a New Resource Type
1. **Backend** (`src-tauri/src/kubernetes.rs`):
   - Add `ResourceInfo` struct with `#[derive(Debug, Clone, Serialize, Deserialize)]`
   - Add `list_resources(client, namespace)` async function
2. **Commands** (`src-tauri/src/commands.rs`):
   - Add import for new struct
   - Add `#[tauri::command] pub async fn get_resources(...)` function
3. **Register** (`src-tauri/src/lib.rs`):
   - Add `commands::get_resources` to `generate_handler![]`
4. **Frontend Store** (`src/lib/stores/kubernetes.ts`):
   - Add interface matching Rust struct
   - Add `writable<ResourceInfo[]>([])` store
   - Add `loadResources(namespace?)` async function
5. **Component** (`src/lib/components/views/ResourceList.svelte`):
   - Create component with table, refresh button, empty state
   - Use `onMount` for initial load + interval
   - Use `$effect` for namespace reactivity
6. **Router** (`src/lib/components/MainPanel.svelte`):
   - Import component
   - Add `{:else if view === 'resources'}` route

### View Component Pattern
```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { resources, selectedNamespace, loadResources, isLoading } from '../../stores/kubernetes';

  onMount(() => {
    loadResources($selectedNamespace);
    const interval = setInterval(() => loadResources($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    loadResources($selectedNamespace);
  });
</script>
```

## Implementation Status

### Completed
- [x] All 17 resource type listings
- [x] Namespace filtering on header (global)
- [x] Context switching (persists to kubeconfig)
- [x] Pulse dashboard with OK:FAIL metrics
- [x] CPU/Memory animated gauges
- [x] Categorized sidebar navigation
- [x] Pod logs viewer
- [x] Pod deletion
- [x] Status indicators (colored dots, badges)
- [x] Auto-refresh (10s intervals)
- [x] Resource detail panels (Pod, Deployment, StatefulSet, Service, Ingress)
- [x] Shared YamlEditorPanel component (UI ready, editing broken)

### TODO (Next Features)
- [ ] Pod exec WebSocket streaming (xterm.js ready)
- [ ] Resource detail panels for remaining types (ConfigMap, Secret, Job, CronJob, etc.)
- [ ] Deployment scaling
- [ ] Deployment/StatefulSet restart
- [ ] Fix YAML editor - typing/editing not working in CodeMirror (YamlEditorPanel.svelte)
- [ ] Real-time watch streams (K8s watch API)
- [ ] Search/filter within tables
- [ ] Application icons
- [ ] Dark/Light theme toggle

## Architecture Notes

1. **Startup Flow**: `StartupScreen.svelte` runs sequential checks before showing main app
2. **State Management**: Svelte writable stores, data fetched via Tauri IPC
3. **Refresh Strategy**: 10-second auto-refresh intervals + manual refresh buttons
4. **Namespace Filtering**: Global selector in header, all views react to `selectedNamespace`
5. **Context Switching**: Writes to kubeconfig file, reloads all data
6. **Error Handling**: Rust errors via `thiserror`, displayed in frontend

## Key Dependencies

### Rust (Cargo.toml)
- `tauri` v2, `kube` v0.98, `k8s-openapi` v0.24
- `tokio`, `serde`, `chrono`, `thiserror`, `tracing`

### Frontend (package.json)
- `@tauri-apps/api` v2, `svelte` v5, `tailwindcss` v3
- `@xterm/xterm` v5, `@xterm/addon-fit`
