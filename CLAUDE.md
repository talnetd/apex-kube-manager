# Apex Kube Manager

A native desktop Kubernetes cluster management application built with Tauri 2 + Svelte 5 + TailwindCSS.

## Tech Stack

- **Backend**: Rust with Tauri 2
- **Frontend**: Svelte 5 with TypeScript
- **Styling**: TailwindCSS 4 (CSS-first `@theme` in `src/styles/app.css`, via `@tailwindcss/vite`)
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
│   │   ├── triage/              # Shared health rules (mod.rs Verdict, rules.rs, fixtures.rs runner)
│   │   └── error.rs             # Error types and handling
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri window/build configuration
│
├── src/                          # Frontend (Svelte 5)
│   ├── lib/
│   │   ├── components/
│   │   │   ├── StartupScreen.svelte # Initialization loading screen
│   │   │   ├── Sidebar.svelte   # Collapsible sections (see Sidebar Navigation below)
│   │   │   ├── Header.svelte    # Context/namespace dropdowns, refresh button
│   │   │   ├── MainPanel.svelte # View router for all resource views
│   │   │   ├── Terminal.svelte  # xterm.js pod exec terminal
│   │   │   ├── ui/              # Reusable UI components
│   │   │   ├── views/           # Resource list views + Dashboard (Pulse) and pulse/ parts
│   │   │   ├── shell/           # App shell, icons
│   │   │   ├── tray/            # Menu bar popover (TrayPopover.svelte)
│   │   │   └── detail/          # Resource detail window components
│   │   │       ├── DetailWindow.svelte    # Window shell + router
│   │   │       ├── kit/                   # DetailToolbar, DetailTabs (shared chrome)
│   │   │       ├── ResourceDetail.svelte  # Unified component for 15 resource types
│   │   │       ├── PodDetail.svelte       # Pod detail (logs, exec, containers)
│   │   │       ├── DeploymentDetail.svelte # Deployment detail (scale, restart)
│   │   │       └── StatefulSetDetail.svelte # StatefulSet detail (scale, restart)
│   │   └── stores/
│   │       ├── startup.ts       # App initialization state
│   │       ├── kubernetes.ts    # All K8s state (contexts, all resources, metrics)
│   │       ├── navigation.ts    # UI navigation state
│   │       └── detail.ts        # Detail window access (read-only guard, Switch back)
│   ├── styles/app.css           # Tailwind import, `@theme` tokens, theme palettes, global styles
│   ├── App.svelte               # Root component
│   └── main.ts                  # Svelte mount entry point
│
└── package.json                 # Frontend dependencies
```

## Commands

```bash
# Install frontend dependencies
npm install

# Development mode (hot-reload frontend + Tauri backend)
npm run tauri dev

# Production build (creates native binary)
npm run tauri build
```

### Alternative: Using Cargo directly

```bash
# Install Tauri CLI globally via cargo (one-time setup)
cargo install tauri-cli

# Then run commands directly with cargo
cargo tauri dev     # Development mode
cargo tauri build   # Production build
```

> **Note**: `npm install` is still required for frontend dependencies. The cargo commands call npm/vite under the hood for the frontend build.

## WSL Development Setup

When developing in WSL but building/testing for Windows:

### Prerequisites (Windows side)
1. **Node.js** (LTS): https://nodejs.org
2. **Rust**: https://rustup.rs (will prompt to install Visual Studio Build Tools)
3. **PowerShell execution policy** (run as Administrator):
   ```powershell
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

### Copy kubeconfig from WSL to Windows

```bash
# Run from WSL - copies to Windows user's .kube folder
mkdir -p /mnt/c/Users/$(whoami)/.kube && cp ~/.kube/config /mnt/c/Users/$(whoami)/.kube/config
```

### Build from Windows

```powershell
# Navigate to project (accessible via WSL path)
cd \\wsl$\Ubuntu\home\YOUR_USERNAME\projects\apex-kube-manager

# Install dependencies and build
npm install
npm run tauri build
```

The MSI installer will be generated at `src-tauri/target/release/bundle/msi/`.

## Implemented Resource Views

All resource views are fully functional with listing, namespace filtering (where applicable), and auto-refresh.

### Workloads
| Resource | View File | Namespace Filter | Key Columns |
|----------|-----------|------------------|-------------|
| Pods | `PodList.svelte` | Yes | Name, Namespace, Status (verdict pill), Ready, Restarts, Age, Node. Reference view on `PageHeader` + `DataTable`; segments All / Failing / Warning / Pending / Running / Completed |
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
| Endpoints (EndpointSlices) | `more/EndpointSliceList.svelte` | Yes | Name, Namespace, Service, Endpoints (ready/total, health), Type, Ports, Age |
| IngressClasses | `more/IngressClassList.svelte` | No (cluster-scoped) | Name (+default), Controller, Parameters, Age |

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
| StorageClasses | `more/StorageClassList.svelte` | No (cluster-scoped) | Name (+default), Provisioner, Reclaim, Binding mode, Expansion, Age |

### Scaling & Policy
| Resource | View File | Namespace Filter | Key Columns |
|----------|-----------|------------------|-------------|
| PodDisruptionBudgets | `more/PdbList.svelte` | Yes | Name, Namespace, Status (`pdbHealth`), Min available, Max unavailable, Allowed, Age |
| ResourceQuotas | `more/ResourceQuotaList.svelte` | Yes | Name, Namespace, Fullest (`quotaHealth`), Usage, Age |
| LimitRanges | `more/LimitRangeList.svelte` | Yes | Name, Namespace, Applies to, Limits, Age |
| PriorityClasses | `more/PriorityClassList.svelte` | No (cluster-scoped) | Name (+default), Value, Preemption, Description, Age |

### Access Control
| Resource | View File | Namespace Filter | Key Columns |
|----------|-----------|------------------|-------------|
| Roles | `more/RoleList.svelte` | Yes | Name, Namespace, Rules, Age |
| RoleBindings | `more/RoleBindingList.svelte` | Yes | Name, Namespace, Role, Subjects, Age |
| ClusterRoles | `more/ClusterRoleList.svelte` | No (cluster-scoped) | Name, Rules, Aggregated, Age |
| ClusterRoleBindings | `more/ClusterRoleBindingList.svelte` | No (cluster-scoped) | Name, Role, Subjects, Age |

### Custom Resources
| Resource | View File | Namespace Filter | Key Columns |
|----------|-----------|------------------|-------------|
| CRDs ("Definitions") | `more/CrdList.svelte` | No (cluster-scoped) | Name, Kind, Group, Scope, Version, Age; row action Browse |
| Custom resources ("Browse") | `more/CustomResourceBrowser.svelte` | Yes, for namespaced CRDs | Pick a CRD, then Name, Namespace, Age via the dynamic API |

The views in `views/more/` use the same `ListView.svelte` and `listStore` as every other list (see "Lists" under Key Patterns); their stores live in `src/lib/stores/resources.ts`. Pure helpers (quantities, quota usage, PDB health, custom resource type ids) live in `src/lib/utils/resources.ts`, covered by `tests/resources.test.ts`.

### Cluster
| Resource | View File | Namespace Filter | Key Columns |
|----------|-----------|------------------|-------------|
| Namespaces | `NamespaceList.svelte` | No (cluster-scoped) | Name, Status, Age |
| Nodes | `NodeList.svelte` | No (cluster-scoped) | Name, Status, Roles, CPU, Memory, Disk, Version, Internal IP, OS/Runtime, Taints, Age |
| ServiceAccounts | `ServiceAccountList.svelte` | Yes | Name, Secrets, Age |

## Dashboard (Pulse View)

`Dashboard.svelte` (parts in `views/pulse/`), top to bottom:
- **KPI strip**: health %, active workloads, healthy / warning / critical counts, API uptime. Counts come from triage verdicts (see below), so a CrashLoopBackOff pod is critical even though its phase is Running
- **Health grid**: one cell per workload object, coloured by verdict, filterable by kind; click opens its detail
- **CPU / Memory / Disk**: cluster usage bars (disk via kubelet summary; `–` when unavailable)
- **Activity**: recent warning events with sparklines (`pulse/history`)
- **Issues**: worst-first list of failing objects; a row opens `IssueSheet` with reason, rule id and actions

## Triage verdicts (health)

Health is computed **in Rust only** (`src-tauri/src/triage/`), never re-derived in TypeScript:
- Rules and ids (`TRI-POD-003` ...) live in `spec/triage/rules.md`, shared with the iOS/Android apps. Schema: `spec/triage/schema/verdict.schema.json`
- `PodInfo`, `DeploymentInfo`, `StatefulSetInfo`, `DaemonSetInfo`, `ReplicaSetInfo`, `JobInfo`, `NodeInfo`, `PersistentVolumeClaimInfo` carry `verdict: { status: ok|warn|fail|unknown, reason, ruleId, since?, severity?: critical|error|warning, tone?: progressing|neutral }`. Old fields (e.g. `PodInfo.status` = phase) stay for compatibility; **display the verdict, not the phase**
- Frontend maps a verdict to display with `toDisplay()` in `src/lib/utils/health.ts`, renders it with `StatusPill`, and buckets with `bucketByHealth()` (`podsByStatus`: `all, failed, warning, progressing, healthy, completed`)
- Golden fixtures: `spec/triage/fixtures/<kind>/<case>.{input,expected}.json`, evaluated at `fixtures/meta.json`'s `now`. `cargo test` runs all of them and fails on a mismatch, an orphan file, or an implemented rule id without a fixture. **A rule change edits `rules.md` and fixtures in the same PR**; `spec/**` changes run desktop CI too
- **Verdicts age without API events** (Pending → Unschedulable after 5 min, OOMKilled expiring after 1h). The pod and deployment watch tasks (`watch.rs`, `Recheck`) re-evaluate unsettled objects every 30s and emit only the ones whose verdict changed. Healthy/neutral objects aren't kept. Any new watch-driven list with time-window rules needs the same tick
- **First seen unsettled**: StatefulSets, ReplicaSets and DaemonSets have no transition times, so their grace windows run from the first time Apex saw them unsettled (`triage::tracked` / `UnsettledTracker`, per uid, process-wide so lists and Pulse agree). It's reset on `switch_to_context`, and a gap of more than 60s between sightings restarts an object's clock. Use the `*_tracked` entry points, never the raw rule fns with `None`. Deployment-owned ReplicaSets are capped at warn: the Deployment owns the failure and Pulse counts it once
- PVC `WaitForFirstConsumer` claims with no consuming pod are neutral (`Waiting for consumer`). `list_pvcs` looks up StorageClass binding modes; without RBAC on StorageClasses it falls back to the Pending window
- **Pod watch invariants** (`startPodWatch` in `kubernetes.ts`): every start/stop bumps `podWatchGen`, and each await in the setup plus the listener checks it. A superseded setup stops the watch it created and never writes `pods`. Events are applied in 100ms batches (`applyPodEvents`), so a burst is one store update. Covered by `tests/podWatch.test.ts`
- Load state: `resourceState(name)` in `kubernetes.ts` (`idle | loading | ready | error | forbidden`, plus `lastSyncedAt`) lets `DataTable` tell connecting / empty / RBAC-denied apart. Pods wire it today; other views move over in redesign PR7

## Sidebar Navigation Structure

Collapsible sentence-case sections (`Sidebar.svelte`); md shows an icon rail, sm an off-canvas drawer:

```
Pulse
Events
Workloads:        Pods, Deployments, StatefulSets, DaemonSets, ReplicaSets, Jobs, CronJobs
Network:          Services, Endpoints, Ingresses, Ingress Classes, Network Policies
Config:           ConfigMaps, Secrets
Storage:          Persistent Volumes, PV Claims, Storage Classes
Scaling & Policy: HPAs, Pod Disruption Budgets, Resource Quotas, Limit Ranges, Priority Classes
Access Control:   Service Accounts, Roles, Role Bindings, Cluster Roles, Cluster Role Bindings
Cluster:          Nodes, Namespaces
Custom Resources: Definitions, Browse
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
| `get_node_metrics` | `context_name, node_names: Vec<String>` | `Vec<NodeMetricsInfo>` |

**Node metrics sources** (`get_node_metrics`, polled every 10s by `NodeList.svelte`):
- CPU/memory usage from metrics-server (`/apis/metrics.k8s.io/v1beta1/nodes`), as a percentage of **allocatable**, matches `kubectl top nodes`
- Disk usage from each kubelet's summary API (`/api/v1/nodes/{name}/proxy/stats/summary` → `node.fs`), which requires `nodes/proxy` RBAC access
- Each source degrades independently: unavailable usage is `null` and the row renders `–` instead of a bar, never an app-wide error. A quantity that fails to parse is also `null`, never a confident `0%`
- `node_names` is supplied by the caller from its watch stream, so polling never re-LISTs node objects; allocatable denominators ride along on `NodeInfo`
- The two sources are fetched concurrently, and the disk sweep is time-boxed to 3s (8 kubelets at a time, 2s per probe). CPU/memory come from one cheap call and must never queue behind a large or unhealthy fleet, whatever answers in time is cached, the rest fills in on later polls
- Disk results are cached 60s per cluster. A **finished** sweep in which every node refuses backs off for 10 min rather than re-probing each poll; a partial sweep never condemns the endpoint, since it says nothing about nodes it didn't reach
- 401/403 backs off immediately (the cluster won't serve this to anyone), but 404 needs *two* consecutive such sweeps, a node deleted mid-sweep 404s exactly like an absent subresource, and on a single-node cluster that one 404 would otherwise disable disk for 10 min. See `classify_sweep` in `kubernetes.rs`, which is unit-tested

**Polling invariants** (`loadNodeMetrics` in `kubernetes.ts`), worth preserving when editing:
- In-flight guard: a slow poll (unreachable kubelets) can outlast the 10s interval; without it, polls stack and a late response overwrites a newer one
- Context guard: node names collide across clusters, so a poll resolving after a context switch must be dropped, not painted onto the new cluster's rows
- `DataTable.svelte` re-anchors `selectedRowIndex` by `rowKey` (the node name, for `NodeList.svelte`) after each re-sort, the usage columns are sortable and change every poll, so a positional selection would drift onto a different node

### Pod Operations
| Command | Parameters | Returns |
|---------|------------|---------|
| `get_pod_logs` | `namespace, pod_name, container?, tail_lines?, previous?` | `String` |
| `delete_pod` | `namespace, pod_name` | `()` |
| `exec_pod` | `namespace, pod_name, container?` | `String` (placeholder) |

### Resource Detail Commands
All resources have detail/yaml/events commands following this pattern:

| Pattern | Parameters | Returns |
|---------|------------|---------|
| `get_{resource}_detail` | `context_name, namespace?, name` | `{Resource}Detail` |
| `get_{resource}_yaml` | `context_name, namespace?, name` | `String` |
| `get_{resource}_events` | `context_name, namespace?, name` | `Vec<{Resource}Event>` |

Resources with related pods also have:
| Command | Parameters | Returns |
|---------|------------|---------|
| `get_{resource}_pods` | `context_name, namespace?, {resource}Name` | `Vec<PodInfo>` |

**Supported resources**: pod, deployment, statefulset, daemonset, replicaset, job, cronjob, service, ingress, configmap, secret, networkpolicy, hpa, pv, pvc, namespace, node, serviceaccount

**Cluster-scoped resources** (no namespace param): pv, namespace, node

### More resource types (`src-tauri/src/resources.rs`)
List commands (current context, like the other lists):

| Command | Parameters | Returns |
|---------|------------|---------|
| `get_roles` / `get_role_bindings` | `namespace?` | `Vec<RoleInfo>` / `Vec<RoleBindingInfo>` |
| `get_cluster_roles` / `get_cluster_role_bindings` | - | `Vec<ClusterRoleInfo>` / `Vec<ClusterRoleBindingInfo>` |
| `get_storage_classes` / `get_ingress_classes` / `get_priority_classes` | - | `Vec<StorageClassInfo>` / `Vec<IngressClassInfo>` / `Vec<PriorityClassInfo>` |
| `get_endpoint_slices` / `get_pdbs` / `get_resource_quotas` / `get_limit_ranges` | `namespace?` | `Vec<EndpointSliceInfo>` / `Vec<PdbInfo>` / `Vec<ResourceQuotaInfo>` / `Vec<LimitRangeInfo>` |
| `get_crds` | - | `Vec<CrdInfo>` |
| `get_custom_resources` | `target: CustomTarget, namespace?` | `Vec<CustomResourceInfo>` |

Detail, YAML and events for these kinds don't follow the `get_{resource}_*` pattern. One dispatch serves them all and returns a generic `GenericDetail { fields: [{label, value}], tables: [{title, columns, rows}], labels, annotations, ... }`, rendered by one block in `ResourceDetail.svelte` (`generic: true` in `resourceConfig`):

| Command | Parameters | Returns |
|---------|------------|---------|
| `get_resource_detail` / `get_resource_yaml` / `get_resource_events` | `context_name, kind, namespace?, name` | `GenericDetail` / `String` / `Vec<GenericEvent>` |
| `get_custom_resource_detail` / `_yaml` / `_events` | `context_name, target: CustomTarget, namespace?, name` | same |

`kind` ids: role, clusterrole, rolebinding, clusterrolebinding, storageclass, endpointslice, ingressclass, pdb, resourcequota, limitrange, priorityclass, crd. `CustomTarget` is `{ group, version, kind, plural, namespaced }`. A custom resource's detail window gets it packed into its type id: `cr|<group>|<version>|<plural>|<Kind>|<n|c>` (`encodeCrType` / `parseCrType`). The window opens on YAML.

To add a kind here: a list fn + row struct, a detail builder, one arm in each dispatch `match` in `resources.rs`, a `listStore` in `stores/resources.ts`, a small view in `views/more/` and a `resourceConfig` entry with `generic: true`.

**Additional commands**:
| Command | Parameters | Returns |
|---------|------------|---------|
| `scale_deployment` | `namespace, name, replicas` | `()` |
| `restart_deployment` | `namespace, name` | `()` |
| `scale_statefulset` | `namespace, name, replicas` | `()` |
| `restart_statefulset` | `namespace, name` | `()` |
| `add_node_taint` | `context_name, name, key, value?, effect` | `()` |
| `remove_node_taint` | `context_name, name, key, effect` | `()` |
| `cordon_node` | `context_name, name` | `()` |
| `uncordon_node` | `context_name, name` | `()` |
| `check_for_updates` | - | `UpdateInfo` |
| `get_secret_data` | `context_name, namespace, name` | `HashMap<String, String>` |
| `open_resource_detail` | `context_name, resourceType, namespace, name` | `()` |

## Multi-window, shared context, menu bar (spec 13)

- **One shared context** for every window (main, detail, terminal, tray). Rust is the source of truth (`src-tauri/src/shell.rs`): `switch_context` writes the kubeconfig once and broadcasts `context-changed { context, seq }`; one 3s watcher in Rust catches outside kubectx changes. Windows apply it in `startContextPolling()` (`kubernetes.ts`, now an event listener; `acceptContextEvent` drops stale `seq`). The namespace stays per window and resets on a switch
- **New Window**: File > New Window / ⌘N (macOS menu; Ctrl+N handled in `App.svelte` elsewhere) opens `main-<n>` via `open_main_window`. Main windows are min 400 px wide; every view must work at sm/md/lg (13.2). `windows-changed` drives the header's "N windows" marker (`stores/windows.ts`)
- **Detail windows** keep the context they opened on. When the shared context moves away they turn read-only (`utils/detail-access.ts` + `stores/detail.ts`) and show "This resource is on <ctx>" with **Switch back**. Every mutating handler starts with `if (!guard('<action>')) return;` (restart, delete, scale, exec, cordon, taint, edit, apply) and its button gets `disabled={$detailReadOnly}`. New detail actions must do both. Detail windows pass their own `context_name` to every mutating command and `kubectl --context` for exec, so they never act on whatever the shared context is now
- **Menu bar item** (macOS, `src-tauri/src/tray.rs`, Tauri `tray-icon` feature): drawn icon, a template image when healthy and a coloured dot for the worst cluster (critical verdict or unreachable = red, other issues = amber). The current context gets a verdict summary, other contexts only a 5s `/version` probe, every 60s and only while enabled. Popover window `tray` (`tray.html`, `components/tray/TrayPopover.svelte`) gets `tray-status`; switching into a prod context from it goes through `ConfirmDialog`. File > Show in Menu Bar toggles it (persisted in `<app config>/tray.json`); with it on, closing the last window keeps Apex running. With it off, closing the last main window quits, and asks first (`confirm-quit`) when terminal windows are still open. The window must ack (`confirm_quit_shown`) within 4s (`ACK_WITHIN` in `shell.rs`) or the close goes through; a second close within 3s always forces it. Clients are cached per context (the current one too, so its sign-in plugin only runs when its token expires) and dropped when the kubeconfig file changes

| Command | Parameters | Returns |
|---------|------------|---------|
| `switch_context` | `context_name` | `()`, broadcasts `context-changed` |
| `open_main_window` / `focus_main_window` | - | `()` |
| `get_main_window_count` | - | `usize` |
| `get_tray_status` / `refresh_tray_status` | - | `TraySnapshot` / `()` |
| `set_tray_enabled` | `enabled` | `()` |
| `hide_tray_popover` | - | `()` |

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
deletePod(ns, name)
startPodWatch(ns?), listStore(...), listWatch(...)  // see Key Patterns > Lists
```

### navigation.ts
```typescript
currentView: View  // 'dashboard' | 'pods' | ... | 'roles' | 'pdbs' | 'crds' | 'customresources' (33 views)
```

## UI Theme (Geist)

Tokens live in `src/styles/app.css`. Each theme (`.dark` default, `.light` on `<html>`) sets `--ui-*` CSS variables; `@theme inline` maps them to utilities (`bg-app`, `bg-surface`, `bg-hover`, `bg-selected`, `border-line`, `text-primary`, `text-secondary`, `text-muted`, `text-accent-text`, `text-danger`, `text-warning`, `text-success`). Dark values:

| Token | Dark | Use |
|-------|------|-----|
| `--ui-app` | `#000000` | Window background |
| `--ui-surface` | `#0a0a0a` | Panels, sidebar |
| `--ui-hover` / `--ui-active` / `--ui-selected` | `#111` / `#1a1a1a` / `#141414` | Row states |
| `--ui-line` / `--ui-line-strong` | `#1f1f1f` / `#2e2e2e` | Borders, dividers |
| `--ui-primary` / `--ui-secondary` / `--ui-muted` | `#ededed` / `#a1a1a1` / `#8f8f8f` | Text |
| `--ui-accent` / `--ui-accent-text` | `#0070f3` / `#52a8ff` | Links, focus, primary action |
| `--ui-danger` / `--ui-warning` / `--ui-success` | `#e5484d` / `#ffb224` / `#45a557` | Status only |

Light mirrors it (`#ffffff` app, `#eaeaea` lines). Rules: no gradients, glows, purple, emoji or shadowed rounded cards; colour means status, everything else stays neutral. Popovers and dialogs use a 1px `--ui-line-strong` ring, not a shadow. Breakpoints are sm < 640, md 640 to 1199, lg ≥ 1200, written as min/max-width queries (older macOS WebKit ignores range syntax).

## Key Patterns

### Lists (every list view except Pods)
One model, in `stores/kubernetes.ts`:
- **`ListSource`**: `items`, `state` (`resourceState`: idle/loading/ready/error/forbidden), `context` (the context the rows came from), `reset()`. Rows clear the moment the context changes, so a row from one cluster is never shown, opened or edited under another
- **`listStore(command, namespaced, { name?, items? })`**: polled loader. `load(ns, extra, { poll, namespaced })`; a `poll: true` tick is skipped while a load is in flight, any other load supersedes it; late answers (older generation or another context) are dropped
- **`listWatch`**: list + watch on a source (deployments, jobs, nodes, events), generation-guarded so double starts can't leak listeners
- **Pods stay on their own watch** (`startPodWatch`, see Triage above) on purpose: it batches events every 100ms, re-checks aging verdicts and has its own tests (`tests/podWatch.test.ts`), and `PodList` has pod-only shortcuts. Moving it onto `listWatch`/`ListView` would mean giving `listWatch` batching first; not worth the risk until then
- `ListView` refuses to open or run shortcuts on rows whose `context` isn't current, and passes the row's context to `open_resource_detail`. Editors (`SecretCreator`, `NetworkPolicyCreator`) take a `context` prop fixed at open and close themselves if the live context moves
- A failed refresh keeps the rows on screen with a banner (DataTable); the full error state is only for an empty table

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

### Resource Detail Architecture

Detail views open in new windows via `open_resource_detail` command.

**Routing** (`DetailWindow.svelte`):
- 3 special cases: Pod, Deployment, StatefulSet (complex features)
- 15 unified cases: Use `ResourceDetail.svelte` component

**ResourceDetail.svelte** - Unified component handling 15 resource types:
```typescript
// Config-driven tabs per resource type
const resourceConfig: Record<ResourceType, ResourceConfig> = {
  daemonset: { displayName: 'DaemonSet', tabs: ['overview', 'pods', 'events', 'yaml'] },
  service: { displayName: 'Service', tabs: ['overview', 'events', 'yaml'] },
  configmap: { displayName: 'ConfigMap', tabs: ['overview', 'data', 'events', 'yaml'] },
  node: { displayName: 'Node', tabs: ['overview', 'pods', 'resources', 'events', 'yaml'], clusterScoped: true },
  // ... 11 more
};

// Dynamic backend calls via template literals
const detail = await invoke(`get_${resourceType}_detail`, getInvokeParams());
const yaml = await invoke(`get_${resourceType}_yaml`, getInvokeParams());
const events = await invoke(`get_${resourceType}_events`, getInvokeParams());
```

**Detail window chrome (Geist)**: `DetailWindow` draws the title bar (kind tag, mono name, namespace · context, PROD, read-only). Each detail view starts with `detail/kit/DetailToolbar.svelte` (the triage `verdict` as a StatusPill, short summary, actions: primary neutral, destructive `variant="danger"`) and `detail/kit/DetailTabs.svelte`. Overviews use `dl.d-props > div.d-prop > dt/dd` rows, sections `h2.d-sec-title`, tables `.d-frame > table.d-table` (all in `app.css`). Every mutating action goes through `ConfirmDialog` (never native `confirm()`/`alert()`); detail structs for pod/deployment/statefulset/daemonset/replicaset/job/node carry `verdict`.

**Key patterns**:
- `clusterScoped: true` flag omits namespace from invoke params
- Conditional `{#if resourceType === 'xxx'}` blocks for resource-specific UI
- Shared tabs: overview, events, yaml
- Resource-specific tabs: pods, data, ingress/egress, metrics, resources

**Shared UI Components** (`src/lib/components/ui/`):
| Component | Purpose |
|-----------|---------|
| `YamlEditorPanel.svelte` | YAML viewer with copy button |
| `EventsTable.svelte` | Card-based events display with refresh |
| `MetadataSection.svelte` | Labels + Annotations grid |
| `ConditionsTable.svelte` | Resource conditions table |
| `CustomSelect.svelte` | Standard dropdown select (use this instead of native `<select>`) |
| `UsageBar.svelte` | Compact table-cell usage bar (`percent: number \| null`, renders `–` when null) |
| `DataTable.svelte` | Shared list table: sorting, keyboard nav, selection anchoring, loading/empty/filtered/error/forbidden states. Columns hide by window width via `hideBelow: BP.md / BP.wide / BP.lg` (`data-table.ts`) |
| `ListView.svelte` | One list page in the Pods style: `PageHeader` (health segments + filter) over `DataTable`. Every list view except Pods is a column config on this. Pass `source={xList}`: a `ListStore` is loaded and polled here; a watched list passes `autoload={false}` and runs its watch in the view |

**CustomSelect Usage** - Use this component for all dropdown selects across the app:
```svelte
<CustomSelect
  bind:value={selectedValue}
  options={[
    { value: 'option1', label: 'Option 1' },
    { value: 'option2', label: 'Option 2' }
  ]}
  placeholder="Select..."
  disabled={false}
  onchange={(value) => handleChange(value)}
/>
```

## Implementation Status

### Completed
- [x] Resource listings (33 views incl. RBAC, storage classes, policy, CRDs)
- [x] Namespace filtering on header (global)
- [x] Context switching (persists to kubeconfig)
- [x] Pulse dashboard with OK:FAIL metrics
- [x] CPU/Memory/Disk usage on Pulse
- [x] Categorized sidebar navigation
- [x] Pod logs viewer (with search, download, previous logs)
- [x] Pod deletion (detail window only; no list-level delete)
- [x] Shared triage verdicts in Rust (`spec/triage/`), Pods list on DataTable + StatusPill
- [x] Status pills from triage verdicts
- [x] Auto-refresh (10s intervals)
- [x] Resource detail panels for all 18 resource types
- [x] Unified ResourceDetail component (handles 15 types)
- [x] Shared UI components (YamlEditorPanel, EventsTable, MetadataSection, ConditionsTable)
- [x] Click-to-open detail from all list views
- [x] Deployment scaling and restart
- [x] StatefulSet scaling and restart
- [x] Secret data reveal/hide toggle
- [x] Node taint management (add/remove taints with effect dropdown)
- [x] Node cordon/uncordon (toggle scheduling with confirmation)
- [x] Update checker (checks GitHub releases, badge in header when update available)
- [x] Search/filter within tables (DataTable filter, ⌘F) and global search (⌘K)
- [x] Multi-window with one shared context, menu bar item (tray), read-only detail windows on a stale context
- [x] Geist redesign (shell, Pulse, detail windows)

### TODO (Next Features)
- [ ] Move Pods onto `listWatch` once it supports batching (see Lists)
- [ ] Command palette with actions (restart deploy, scale, logs, exec - all from keyboard)
- [x] More resource types: RBAC, StorageClasses, EndpointSlices, IngressClasses, PDBs, ResourceQuotas, LimitRanges, PriorityClasses, CRDs + generic custom resource browser
- [ ] Resource deletion (deployments, services, configmaps, etc.)
- [ ] Node drain (evict pods with options)
- [ ] Application icons
- [ ] NetworkPolicy packet monitoring (requires Cilium CNI + Hubble integration)
- [ ] Create UI for Namespaces (simple name input)
- [ ] Create UI for ConfigMaps (key-value editor)
- [x] Create/Edit/Delete UI for Secrets (key-value editor with base64 handling)
- [x] Create/Edit UI for NetworkPolicies (graphical rule builder)
- [x] Dark/Light theme toggle
- [x] Pod exec terminal (via kubectl exec + PTY)
- [x] Keyboard-first navigation (vim-style j/k, Enter to open, shortcuts for actions)

## Architecture Notes

1. **Startup Flow**: `StartupScreen.svelte` runs sequential checks before showing main app
2. **State Management**: Svelte writable stores, data fetched via Tauri IPC
3. **Refresh Strategy**: watch streams where available (pods, deployments, jobs, nodes, events), 10s polling elsewhere, plus manual refresh
4. **Namespace Filtering**: Global selector in header, all views react to `selectedNamespace`
5. **Context Switching**: Rust writes the kubeconfig once and broadcasts `context-changed` to every window (see Multi-window)
6. **Error Handling**: Rust errors via `thiserror`, displayed in frontend
7. **Destructive Actions in Detail Views**: Impactful operations (scale, restart, cordon, taints, delete) are placed in detail views, NOT list views. This creates deliberate "speed bumps":
   - User must click specific resource → opens detail (confirms identity)
   - User must find and click action button (intentional navigation)
   - Confirmation modal for dangerous actions (final check)
   This prevents accidental misclicks on wrong rows in list views. Examples: Deployment scale/restart, Node cordon/taints, Pod delete.

## Keyboard Shortcuts

Press `?` anytime to show the keyboard shortcuts help modal.

### Navigation
| Key | Action |
|-----|--------|
| `j` / `↓` | Move down in list |
| `k` / `↑` | Move up in list |
| `g g` | Go to first row |
| `G` | Go to last row |
| `Enter` | Open detail view |
| `Esc` | Clear selection / Close modal |

### Actions (on selected row)
| Key | Action |
|-----|--------|
| `l` | View logs (Pods) |
| `e` | Exec into pod (Pods) |
| `s` | Scale (Deployments) |
| `d` | Open detail (lists never delete; delete lives in the detail view) |
| `y` | Copy resource name |
| `r` | Refresh view |

### Global
| Key | Action |
|-----|--------|
| `⌘K` / `/` | Open global search |
| `⌘F` | Focus filter input |
| `⌘P` | Toggle port forwards panel |
| `⌘W` | Close detail window |
| `?` | Show keyboard shortcuts help |

### Implementation Details
- Keyboard store: `src/lib/stores/keyboard.ts`
- Help modal: `src/lib/components/ui/KeyboardHelp.svelte`
- Global handlers in `App.svelte`; list views on `DataTable`/`ListView` get Enter/y/r/d, plus per-view `shortcuts` (Pods: l, e)
- Detail windows: tabs are a `DetailTabs` tablist (←/→/Home/End move and activate, ignored with ⌥/⌘/Ctrl held), each tab controls the `role="tabpanel"` below it
- Row selection uses the `selectedRowIndex` store and the neutral `bg-selected` row token (no accent tint)
- Scrolls selected row into view automatically

## Key Dependencies

### Rust (Cargo.toml)
- `tauri` v2, `kube` v4, `k8s-openapi` v0.28 (`v1_36`; `Time` is `jiff::Timestamp`, see `TimestampExt` in `kubernetes.rs`)
- `tokio`, `serde`, `chrono`, `thiserror`, `tracing`

### Frontend (package.json)
- `@tauri-apps/api` v2, `svelte` v5, `vite` v8, `tailwindcss` v4
- `@xterm/xterm` v6, `@xterm/addon-fit`
