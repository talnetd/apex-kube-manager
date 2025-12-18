# Apex Kube Manager - Design Decisions

This document captures UI/UX design decisions and rationale for the Apex Kube Manager application.

## Design Philosophy

**Target User**: DevOps engineers, SREs, and platform engineers who need efficient Kubernetes cluster management during debugging, monitoring, and daily operations.

**Core Principles**:
1. **Information density over whitespace** - Users need to see data, not decorative UI
2. **Keyboard-first, mouse-friendly** - Power users expect shortcuts, but mouse should work well
3. **Native desktop advantages** - Leverage multi-window, system integration, performance
4. **Reduce context switching** - Keep relevant information accessible without losing place

---

## Navigation Structure

### Decision: Flat sidebar with section headers (not nested accordions)

**Rationale**:
- Single click to any resource type
- All options visible without expanding/collapsing
- Predictable scroll position
- Faster navigation for experienced users

**Categories**:
```
Pulse (Dashboard)
─────────────────
WORKLOADS: Pods, Deployments, StatefulSets, DaemonSets, ReplicaSets, Jobs, CronJobs
NETWORK: Services, Ingresses, Network Policies
CONFIG: ConfigMaps, Secrets, HPAs
STORAGE: Persistent Volumes, Persistent Volume Claims
CLUSTER: Namespaces, Nodes, Service Accounts
```

---

## Resource List Views

### Decision: Compact toolbar with inline filters

**Rationale**:
- Status filters (All/Running/Pending/Failed) as pills in toolbar
- No separate filter panel that takes vertical space
- Count badges show distribution at a glance
- One-click filtering without dropdowns

### Decision: Sortable columns with visual indicators

**Rationale**:
- DevOps often need to find "oldest pods" or "most restarts"
- Age-aware sorting (parses "5d", "2h", "30m" correctly)
- Click column header to sort, click again to reverse
- Subtle arrow indicator shows current sort

### Decision: Remove Node column from Pod list

**Rationale**:
- Node information available in Node detail view
- Reduces horizontal scrolling
- Most debugging doesn't start with "which node"
- Can be added back as optional column later

---

## Refresh Strategy

### Decision: Unified global refresh + auto-refresh

**Implementation**:
- Single refresh button in header (beside search)
- 10-second auto-refresh intervals per view
- Global `refreshTrigger` store for manual refresh all views
- 3-second context polling for external changes (kubectl, kubectx)

**Rationale**:
- Consistent behavior across all views
- No per-view refresh buttons cluttering UI
- External tool compatibility (user can switch context via CLI)
- Auto-refresh keeps data fresh without manual intervention

---

## Window Strategy

### Decision: Multi-window for first-class resource details (UNDER DISCUSSION)

**Context**:
DevOps debugging workflow often requires examining multiple resources simultaneously - comparing env vars between pods, checking events while viewing logs, etc.

**Options Considered**:

#### Option A: Single-window with slide-out panels
```
┌─────────────────────────────────────────────┐
│ Header                                      │
├──────┬──────────────────────────────────────┤
│      │ Pod List                             │
│ Side │ ┌──────────────────────────────────┐ │
│ bar  │ │ Pod 1  Pod 2  Pod 3              │ │
│      │ └──────────────────────────────────┘ │
│      │         ┌────────────────────────────┤
│      │         │ Detail Panel (slides in)   │
│      │         │ - Env vars                 │
│      │         │ - Volumes                  │
│      │         │ - Events                   │
│      │         └────────────────────────────┤
└──────┴──────────────────────────────────────┘
```
- Pros: Simple, no window management
- Cons: Can't compare two pods, loses list context

#### Option B: Multi-window (spawn new windows)
```
┌─────────────────────┐  ┌─────────────────────┐
│ Main Window         │  │ Pod: nginx-abc      │
│ - Pod List          │  │ ─────────────────── │
│ - Sidebar           │  │ Containers  Events  │
│                     │  │ Env Vars    Logs    │
│                     │  │ Volumes     YAML    │
└─────────────────────┘  └─────────────────────┘
                         ┌─────────────────────┐
                         │ Pod: nginx-xyz      │
                         │ (compare side by    │
                         │  side on 2nd monitor│
                         └─────────────────────┘
```
- Pros: Compare resources, multi-monitor, native feel
- Cons: Window management, state sync complexity

#### Option C: Hybrid (default panel, pop-out option)
- Quick peek in slide-out panel
- "Open in new window" button for deep debugging
- Best of both, more implementation work

**Leaning toward**: Option B or C

**Key debugging scenarios to support**:
1. Compare env vars between working and broken pod
2. View logs while checking events
3. Examine pod spec while reading deployment YAML
4. Multi-monitor setup (list on one, details on another)

---

## Pod Detail View (Proposed)

### Information Architecture

**Tabs/Sections**:
1. **Overview** - Status, conditions, QoS class, node, IP addresses
2. **Containers** - Per-container: image, ports, resources, probes, state
3. **Environment** - All env vars (from configmaps, secrets, field refs)
4. **Volumes** - Volume mounts with source details
5. **Events** - Pod events with timestamps
6. **Logs** - Live log streaming (already implemented)
7. **YAML** - Raw manifest with syntax highlighting

**Quick Actions**:
- Delete pod
- Copy pod name/namespace
- Open shell (exec)
- Download logs

---

## Color Palette

### Decision: Dark theme with semantic colors

```
Background:   #0a0a0a (primary), #111111 (secondary), #1a1a1a (tertiary)
Borders:      #222222 (subtle)
Text:         #ffffff (primary), #888888 (secondary), #555555 (muted)

Semantic:
- Success/Ready:    #22c55e (green)
- Warning/Pending:  #f59e0b (amber)
- Error/Failed:     #ef4444 (red)
- Accent/Interactive: #00d4aa (teal)
```

**Rationale**:
- High contrast for readability
- Semantic colors match Kubernetes status conventions
- Teal accent differentiates from red/green status colors
- Dark theme reduces eye strain during long debugging sessions

---

## Startup Experience

### Decision: Sequential checks with visible progress

**Checks**:
1. Locating kubeconfig
2. Parsing configuration
3. Loading contexts
4. Connecting to cluster
5. Fetching namespaces

**Timing**: 400ms minimum per step (2+ seconds total)

**Rationale**:
- Shows app is working, not frozen
- Builds confidence in connection status
- Catches errors early with clear messages
- Pleasant visual progression

---

## Future Considerations

### Keyboard Shortcuts (TODO)
- `r` - Refresh
- `f` - Focus search
- `/` - Quick filter
- `j/k` - Navigate list
- `Enter` - Open detail
- `Esc` - Close panel/window

### Search (TODO)
- Global search across all resource types
- Fuzzy matching
- Recent searches

### Themes (TODO)
- Light theme option
- System preference detection
- Custom accent colors

---

## Resolved Decisions

### Resource Detail Windows

**Decision**: Single-click on row opens detail in new window

**Applies to** (phased rollout):
- Phase 1: Pods (priority - debugging focus)
- Phase 2: Deployments, StatefulSets, DaemonSets
- Phase 3: Other resources as needed

**Window behavior**:
- Single-click opens detail window
- Multiple windows allowed (compare scenarios)
- Windows are independent (closing main doesn't close details)
- Window title: `{resource}: {name}` (e.g., "Pod: nginx-abc123")

---

## Pod Detail Window Specification

### Window Layout
```
┌──────────────────────────────────────────────────────────────┐
│ Pod: nginx-deployment-7d4b8c9f5-abc12              [−][□][x] │
├──────────────────────────────────────────────────────────────┤
│ [Overview] [Containers] [Env] [Volumes] [Events] [Logs] [YAML]│
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Tab content area                                            │
│                                                              │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Tabs

#### 1. Overview
- Pod status, phase, conditions
- QoS class, restart policy
- Node assignment, IP addresses
- Labels and annotations (collapsible)
- Owner references (link to deployment?)

#### 2. Containers
- List all containers (init + regular)
- Per container:
  - Image (with tag)
  - State (running/waiting/terminated + reason)
  - Ports
  - Resource requests/limits
  - Liveness/readiness probes
  - Last restart reason

#### 3. Env (Environment Variables)
- Merged view from all containers
- Source indicator: `(literal)`, `(configmap)`, `(secret)`, `(fieldRef)`
- Expandable secret values (with warning)
- Copy single value / Copy all as .env format
- Search/filter env vars

#### 4. Volumes
- Volume mounts per container
- Volume source details (PVC, ConfigMap, Secret, EmptyDir, etc.)
- Mount paths and read-only status

#### 5. Events
- Pod events with timestamps
- Warning events highlighted
- Auto-refresh (newest first or oldest first toggle)

#### 6. Logs
- **Live tailing enabled by default**
- Container selector (if multi-container)
- Previous container logs toggle
- Timestamps toggle
- Search within logs
- Download logs button
- Clear/pause stream buttons
- Wrap lines toggle

#### 7. YAML
- **Full pod manifest with syntax highlighting**
- **Edit button** → enables editing mode
- **Validation** before apply:
  - YAML syntax validation
  - Kubernetes schema validation (field names, types)
  - Warning for immutable fields (can't change most pod spec fields)
- **Apply button** (creates new pod if spec changed, since pods are mostly immutable)
- **Reset button** (discard changes)
- **Copy button**
- Line numbers
- Collapsible sections (metadata, spec, status)

### YAML Editor Requirements

**Editor features**:
- Syntax highlighting (YAML-aware)
- Line numbers
- Error gutter (red markers for invalid lines)
- Auto-indent
- Bracket matching
- Search & replace (Ctrl+F)
- Minimap (optional)

**Validation layers**:
1. **Syntax**: Valid YAML? (real-time, as you type)
2. **Schema**: Valid K8s Pod spec? (on blur or save attempt)
3. **Semantic**: Warns about immutable fields being changed

**Editor library candidates**:
- Monaco Editor (VS Code's editor) - full featured, larger bundle
- CodeMirror 6 - lighter, modular, good YAML support
- Ace Editor - battle-tested, moderate size

**Recommendation**: CodeMirror 6
- Smaller bundle than Monaco
- Excellent for embedded use
- Good YAML mode with syntax highlighting
- Can add JSON schema validation via addons

### Quick Actions (floating or in header)
```
[Delete Pod] [Copy Name] [Copy YAML] [Refresh]
```

---

## Resolved: Pod Deletion While Window Open

**Scenario**: User has Pod detail window open, pod gets deleted (scaled down, crashed, evicted, etc.)

**Decision**: Graceful degradation with "tombstone" state

**Behavior**:
```
┌──────────────────────────────────────────────────────────────┐
│ Pod: nginx-deployment-7d4b8c9f5-abc12              [−][□][x] │
├──────────────────────────────────────────────────────────────┤
│ ⚠️  THIS POD NO LONGER EXISTS                    [Dismiss]   │
│     Last seen: 2 minutes ago                                 │
├──────────────────────────────────────────────────────────────┤
│ [Overview] [Containers] [Env] [Volumes] [Events] [Logs] [YAML]│
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  (Last known state - read only, actions disabled)            │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

**Rationale**:
- **Post-mortem debugging**: DevOps needs to see what the pod looked like when it crashed
- **Don't auto-close**: Jarring UX, loses context mid-investigation
- **Keep last state**: Env vars, volumes, events still valuable for debugging
- **Disable actions**: Delete button, exec, log streaming disabled (pod gone)
- **Clear visual indicator**: Warning banner so user knows it's stale

**Implementation**:
- Poll pod existence every 5 seconds (or use K8s watch API)
- On 404: Set `isDeleted = true`, show banner, disable actions
- Keep all last-fetched data in memory
- YAML tab shows last known manifest (read-only)
- Logs tab shows "Pod no longer exists, showing cached logs" if any were loaded

---

## Resolved: YAML Editor

**Decision**: Include YAML tab for consistency, but pods rarely need editing

**Behavior**:
- View mode by default (read-only, syntax highlighted)
- Edit button enables editing
- On save attempt for pods: Show warning about immutability
- For mutable resources (Deployments, ConfigMaps): Normal edit flow

---

## Resolved: Pod Exec Terminal Strategy

**Decision**: Hybrid approach - embedded terminal + external terminal option

**Target audience**: DevOps engineers and SREs who need quick access for debugging but also value their customized terminal environments.

### Option 1: Embedded Terminal (xterm.js)

**Use case**: Quick tasks - check a file, run a command, verify env vars

```
┌──────────────────────────────────────────────────────────────┐
│ Pod: nginx-deployment-7d4b8c9f5-abc12              [−][□][x] │
├──────────────────────────────────────────────────────────────┤
│ [Overview] [Containers] [Env] [Volumes] [Events] [Logs] [Terminal]│
├──────────────────────────────────────────────────────────────┤
│ Container: [nginx ▼]  Shell: [/bin/sh ▼]  [Open in Terminal] │
├──────────────────────────────────────────────────────────────┤
│ $ ls -la /etc/nginx/                                         │
│ total 24                                                     │
│ drwxr-xr-x 3 root root 4096 Dec  7 10:00 .                   │
│ -rw-r--r-- 1 root root  643 Dec  7 10:00 nginx.conf          │
│ $ _                                                          │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

**Implementation**:
- xterm.js already integrated
- WebSocket connection to Tauri backend
- Backend uses `kubectl exec` or kube-rs exec API
- Container selector dropdown (for multi-container pods)
- Shell selector: `/bin/sh`, `/bin/bash`, `/bin/zsh` (detect available)
- Session persists while tab is open

### Option 2: Open in External Terminal

**Use case**: Deep debugging, long sessions, need user's tools/aliases

**Button**: "Open in Terminal" in the Terminal tab header

**Behavior**:
1. Generate kubectl command: `kubectl exec -it <pod> -n <ns> -c <container> -- /bin/sh`
2. Platform-specific launch:
   - **macOS**: `open -a Terminal` or detect iTerm2/Warp/Alacritty
   - **Linux**: `x-terminal-emulator` or `$TERMINAL`
   - **Windows**: `wt.exe` (Windows Terminal) or `cmd.exe`
3. Fallback: Copy command to clipboard with toast notification

**User preference** (future): Settings to choose default terminal app

### Why Hybrid?

| Scenario | Best Option |
|----------|-------------|
| Quick check: "is the config file there?" | Embedded |
| Run one command, see output | Embedded |
| Debug with vim/nano | External |
| Need tmux, shell history, aliases | External |
| Multi-hour debugging session | External |
| Compare files across pods | External (multiple terminals) |

**Industry precedent**:
- Lens: Embedded only
- Rancher Desktop: Embedded only
- k9s: Embedded (TUI)
- kubectl: External only

Our hybrid approach gives power users an escape hatch while keeping quick tasks frictionless.

### Terminal Tab Specification

**Header bar**:
```
Container: [nginx ▼]  Shell: [/bin/sh ▼]  [Reconnect] [Open in Terminal] [Copy Command]
```

**Features**:
- Auto-reconnect on disconnect
- Visual indicator: Connected (green) / Disconnected (red) / Connecting (amber)
- Resize handling (xterm.js + fit addon)
- Copy/paste support
- Scrollback buffer (configurable, default 1000 lines)

**Error states**:
- Container not running → "Container is not running. Cannot exec."
- No shell available → "No shell found. Try: /bin/sh, /bin/bash"
- Connection lost → "Connection lost. [Reconnect]"
- Pod deleted → "Pod no longer exists."

---

## Implemented: Terminal Architecture (PTY-based)

**Status**: Implemented

The terminal feature uses a **PTY (Pseudo-Terminal)** architecture for full terminal emulation, supporting interactive shells, line editing, cursor movement, and all terminal features.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Terminal.svelte (Frontend)                       │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │  xterm.js Terminal                                                  ││
│  │  - Renders terminal output                                          ││
│  │  - Captures keyboard input                                          ││
│  │  - Handles resize events                                            ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                              │                                          │
│           terminal.onData() ─┤─ Tauri invoke('pty_write')              │
│           terminal.write() ◄─┤─ Tauri listen('pty-data-{sessionId}')   │
└─────────────────────────────────────────────────────────────────────────┘
                               │
                     Tauri IPC Bridge
                               │
┌─────────────────────────────────────────────────────────────────────────┐
│                         PtyManager (Rust Backend)                        │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │  portable-pty                                                       ││
│  │  - Creates real pseudo-terminal                                     ││
│  │  - Spawns kubectl exec process                                      ││
│  │  - Manages PTY sessions (HashMap<sessionId, PtySession>)            ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                              │                                          │
│     PTY Master ◄─────────────┼─────────────► kubectl exec -it          │
│     (read/write)             │               (PTY slave)               │
└─────────────────────────────────────────────────────────────────────────┘
```

### Why PTY over Shell Plugin?

| Approach | Shell Plugin | PTY (portable-pty) |
|----------|-------------|-------------------|
| TTY support | ❌ No TTY, `-i` only | ✅ Full TTY with `-it` |
| Line editing | ❌ No backspace, arrows | ✅ Full readline support |
| Tab completion | ❌ Not supported | ✅ Works natively |
| Clear screen | ❌ Doesn't work | ✅ `clear`, Ctrl+L work |
| Vi/nano editors | ❌ Broken | ✅ Full support |
| Signal handling | ⚠️ Limited | ✅ Ctrl+C, Ctrl+Z work |
| Color output | ⚠️ Requires workarounds | ✅ Full ANSI support |
| Window resize | ❌ Fixed size | ✅ Dynamic resize |

### Tauri Commands

```rust
// Spawn a new PTY session, returns session_id
pty_spawn(namespace, pod_name, container?) -> String

// Write data to PTY (keyboard input from xterm.js)
pty_write(session_id, data) -> ()

// Resize PTY when terminal window resizes
pty_resize(session_id, rows, cols) -> ()

// Close PTY session and clean up
pty_close(session_id) -> ()
```

### Event Streams

The PTY backend emits events that the frontend listens to:

```typescript
// Output data from PTY → xterm.js
listen<string>(`pty-data-${sessionId}`, (event) => {
  terminal.write(event.payload);
});

// Process exited (user typed 'exit' or connection lost)
listen(`pty-exit-${sessionId}`, () => {
  isConnected = false;
});

// Error occurred
listen<string>(`pty-error-${sessionId}`, (event) => {
  connectionError = event.payload;
});
```

### Session Lifecycle

```
1. User opens terminal → open_terminal_window() creates new window
2. Terminal.svelte mounts → invoke('pty_spawn', {namespace, podName, container})
3. PtyManager spawns PTY → kubectl exec -it <pod> -n <ns> -- /bin/sh
4. Read thread starts → emits pty-data-{sessionId} events
5. User types → terminal.onData() → invoke('pty_write', {sessionId, data})
6. User closes window → cleanup() → invoke('pty_close', {sessionId})
```

### Rust Implementation Details

**PtyManager** (`src-tauri/src/pty.rs`):
- Uses `portable-pty` crate for cross-platform PTY support
- Sessions stored in `Arc<Mutex<HashMap<String, PtySession>>>`
- Each session has: PTY pair, writer handle
- Spawns kubectl with proper args: `exec -it <pod> -n <ns> [-c <container>] -- /bin/sh`
- Reader thread runs in background, emits events via `app.emit()`
- Waiter thread monitors process exit

**Session cleanup**:
- Frontend calls `pty_close` on window close
- Session removed from HashMap
- Reader thread terminates on EOF
- PTY resources automatically cleaned up by Drop

### Frontend Implementation Details

**Terminal.svelte** (`src/lib/components/Terminal.svelte`):
- xterm.js with addons: FitAddon (resize), WebLinksAddon (clickable URLs)
- Dark theme matching app palette
- Connection status indicator (green/red dot)
- Reconnect button when disconnected
- Window resize handling → `pty_resize` call

**Terminal window features**:
- Header shows pod name, namespace, container
- Independent window (can have multiple terminals open)
- Focus terminal on connect
- Scrollback: 10,000 lines

### Dependencies

```toml
# Cargo.toml
portable-pty = "0.8"
uuid = { version = "1", features = ["v4"] }
```

```json
// package.json
"@xterm/xterm": "^5.5.0",
"@xterm/addon-fit": "^0.10.0",
"@xterm/addon-web-links": "^0.11.0"
```

---

## Open Questions

1. **Window positioning**: Remember window positions? Cascade new windows?

2. **Log persistence**: Keep logs when switching tabs? Or re-fetch?

3. **Watch vs Poll**: Use K8s watch API for real-time updates or stick with polling?

4. **Terminal preference**: Should we detect user's preferred terminal app or always use system default?

---

## Session: December 9, 2025 - Pod Log Enhancements

### 1. Terminal Reconnection Fix

**Problem:** When disconnecting from a pod shell and reconnecting, the terminal showed a blank screen.

**Root Cause:** The `reconnect()` function called `cleanup()` which disposed the terminal instance entirely. After disposal, `startPty()` tried to write to a non-existent terminal.

**Solution:** Split cleanup into two functions:
- `cleanupPty()` - Only cleans up PTY session and event listeners, keeps terminal alive
- `cleanup()` - Calls `cleanupPty()` + disposes terminal (used on component unmount)

**Files Changed:**
- `src/lib/components/Terminal.svelte`

---

### 2. Pod Log Enhancements

Added high-value features to the pod logs viewer in both `PodDetail.svelte` and `LogViewer.svelte`.

#### 2.1 Log Search/Filter
- Real-time filtering of log lines as you type
- Shows match count (e.g., "42 matches")
- Clear button to reset search
- Case-insensitive search

#### 2.2 Log Download
- Download button exports logs to a `.txt` file
- Filename format: `{pod}-{container}-{timestamp}.txt`
- If search filter is active, downloads only matching lines
- Uses browser's native download mechanism (Blob + anchor click)

#### 2.3 Previous Container Logs
- "Previous" checkbox to view logs from crashed/restarted containers
- Uses Kubernetes `--previous` flag via `LogParams.previous`
- Useful for debugging crash loops

#### 2.4 Tail Lines Selector
- Dropdown to select number of lines: 100, 500, 1000, 5000
- Allows viewing more history when debugging

#### 2.5 Scroll to Bottom Button
- Floating teal button at bottom-right of log area
- Down arrow icon with hover scale effect
- Uses `absolute` positioning within log container
- Fixes issue where auto-refresh/tailing doesn't properly scroll

**Files Changed:**
- `src-tauri/src/kubernetes.rs` - Added `previous` parameter to `get_logs()`
- `src-tauri/src/commands.rs` - Updated `get_pod_logs` command signature
- `src/lib/stores/kubernetes.ts` - Updated `getPodLogs()` function
- `src/lib/components/detail/PodDetail.svelte` - Enhanced logs tab UI
- `src/lib/components/LogViewer.svelte` - Added same features

**Backend API Change:**
```rust
// Before
pub async fn get_logs(client, namespace, pod_name, container, tail_lines) -> Result<String>

// After
pub async fn get_logs(client, namespace, pod_name, container, tail_lines, previous: Option<bool>) -> Result<String>
```

---

### 3. Title Capitalization Fix

**Problem:** Pod detail window title showed `Pod: Apisix-664967d48d-Lkbbn` with incorrectly capitalized pod name.

**Root Cause:** CSS `text-transform: capitalize` was applied to the entire title, capitalizing the first letter of every word including the pod name.

**Solution:** Only capitalize the resource type, not the resource name:
- Wrap only `{resourceType}` in a `<span class="capitalize">`
- In Rust, manually capitalize first letter for window title

**Files Changed:**
- `src/lib/components/detail/DetailWindow.svelte` - Fixed h1 and h2 titles
- `src-tauri/src/commands.rs` - Fixed window title in `open_resource_detail()`

**Before:** `Pod: Apisix-664967d48d-Lkbbn`
**After:** `Pod: apisix-664967d48d-lkbbn`

---

### Design Decisions Made

#### Log Filtering Strategy
Client-side filtering was chosen over server-side for several reasons:
1. Kubernetes API doesn't support log filtering
2. Immediate feedback as user types
3. No additional API calls needed
4. Logs are already loaded in memory

#### Scroll Button Positioning
Used `absolute` positioning within a `relative` container rather than `fixed`:
- Button stays within the log area bounds
- Doesn't interfere with other UI elements
- Works correctly in windowed detail views

#### Previous Logs Toggle
Implemented as a checkbox rather than a separate button:
- Cleaner UI with existing controls
- State is preserved while switching containers
- Auto-refresh respects the toggle state

---

## Session: December 9, 2025 - Deployment Features

### Deployment Operations Implemented

Full CRUD-like operations for Kubernetes Deployments.

#### Backend Commands (Rust)

| Command | Parameters | Description |
|---------|------------|-------------|
| `scale_deployment` | `namespace, name, replicas` | Scale deployment replicas up/down |
| `restart_deployment` | `namespace, name` | Rollout restart via annotation |
| `get_deployment_detail` | `context_name, namespace, name` | Full deployment details |
| `get_deployment_yaml` | `context_name, namespace, name` | YAML manifest |
| `get_deployment_events` | `context_name, namespace, name` | Deployment events |
| `get_deployment_pods` | `context_name, namespace, name` | Pods owned by deployment |

**Files Changed:**
- `src-tauri/src/kubernetes.rs` - New functions + structs (DeploymentDetail, DeploymentCondition, DeploymentEvent)
- `src-tauri/src/commands.rs` - Tauri command wrappers
- `src-tauri/src/lib.rs` - Command registration

#### Restart Implementation

Rollout restart is implemented by patching the pod template annotation:
```rust
let patch = serde_json::json!({
    "spec": {
        "template": {
            "metadata": {
                "annotations": {
                    "kubectl.kubernetes.io/restartedAt": chrono::Utc::now().to_rfc3339()
                }
            }
        }
    }
});
```
This triggers Kubernetes to create new pods, matching `kubectl rollout restart` behavior.

---

### DeploymentList.svelte Enhancements

#### Status Indicator
- Green dot: All replicas ready (healthy)
- Amber dot: Some replicas ready (degraded)
- Red dot: No replicas ready (failing)

#### Actions Column
- Added "Actions" header to clarify button purpose
- Scale button (resize icon) → Opens scale modal
- Restart button (refresh icon) → Confirms then restarts
- Buttons show hover colors (teal for scale, amber for restart)
- Restart button spins while restarting

#### Click to Open Detail
- Clicking row opens deployment detail in new window
- Uses `open_resource_detail` command with `resourceType: 'deployment'`

---

### DeploymentDetail.svelte Component

New detail window component with tabbed interface.

#### Tabs

| Tab | Content |
|-----|---------|
| Overview | Health status, replicas, strategy, conditions, labels, selector, container images |
| Pods | Table of pods owned by deployment, click to open pod detail |
| Events | Deployment events with type/reason/message |
| YAML | Full manifest with copy button, uses YamlEditor component |

#### Action Bar
- Scale button → Opens modal with +/- controls
- Restart button → Confirms then triggers rollout restart

#### Tombstone State
When deployment is deleted while window is open:
- Warning banner appears
- Actions are disabled
- Last-seen timestamp shown
- Data remains visible for post-mortem analysis

---

### Scale Modal UI

Improved scale modal with better UX:

```
┌─────────────────────────────────────┐
│  Scale Deployment                    │
│                                      │
│  Set replicas for my-deployment      │
│                                      │
│    [ - ]     [ 3 ]     [ + ]        │
│                                      │
│              [Cancel] [Scale]        │
└─────────────────────────────────────┘
```

**Design Decisions:**
- SVG icons for +/- buttons (not text characters)
- Hover colors: red for decrease, green for increase
- Large number input (3xl font) for visibility
- Hidden browser spinners on number input
- Centered layout for visual balance

---

### Files Added/Changed

**New Files:**
- `src/lib/components/detail/DeploymentDetail.svelte`

**Modified Files:**
- `src-tauri/src/kubernetes.rs` - Deployment operations
- `src-tauri/src/commands.rs` - Tauri commands
- `src-tauri/src/lib.rs` - Command registration
- `src/lib/components/detail/DetailWindow.svelte` - Deployment route
- `src/lib/components/views/DeploymentList.svelte` - Working buttons + click handler

---

## Session: December 9, 2025 - List View Consistency

### Problem

Inconsistent layout, font sizes, and spacing between list views. PodList and DeploymentList had different patterns, and other list views varied in implementation.

### Solution: Unified List View Pattern

Applied consistent design pattern across all 16 resource list views.

#### Standard Table Pattern

```svelte
<thead>
  <tr class="text-left border-b border-border-subtle">
    <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-4"></th>
    <SortableHeader label="Name" ... />
    <!-- other columns -->
    <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-24">Actions</th>
  </tr>
</thead>
<tbody>
  {#each sortedData as item}
    <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors cursor-pointer">
      <td class="py-3 pr-2">
        <div class="w-2 h-2 rounded-full {statusColor}"></div>
      </td>
      <td class="py-3 pr-4">
        <span class="text-accent-primary font-medium hover:underline">{item.name}</span>
      </td>
      <!-- other columns -->
    </tr>
  {/each}
</tbody>
```

#### Key Changes Applied

| Element | Before | After |
|---------|--------|-------|
| Status dot | Inside name cell | Separate w-4 column |
| Name color | `text-text-primary` | `text-accent-primary` |
| Name hover | None | `hover:underline` |
| Row cursor | Default | `cursor-pointer` |
| Actions header | Empty `<th>` | "Actions" label |
| Action button hover | `hover:text-text-primary` | Semantic colors |

#### Action Button Hover Colors

- **Scale**: `hover:text-accent-primary` (teal)
- **Restart**: `hover:text-accent-warning` (amber)
- **Delete**: `hover:text-accent-error` (red)
- **Trigger/Play**: `hover:text-accent-success` (green)
- **View Logs**: `hover:text-accent-primary` (teal)

#### Status Dot Logic by Resource Type

| Resource | Green | Amber | Red |
|----------|-------|-------|-----|
| Pods | Running | Pending | Failed/Error |
| Deployments | All ready | Partial ready | None ready |
| StatefulSets | All ready | Partial ready | None ready |
| DaemonSets | All ready | Partial ready | None ready |
| ReplicaSets | Desired=0 or all ready | Partial ready | None ready |
| Jobs | Complete | Running | Failed |
| CronJobs | Active (not suspended) | Suspended | - |
| Services | Always green | - | - |
| Ingresses | Has address | No address | - |
| PVs | Available | Released | Failed |
| PVCs | Bound | Pending | Lost |
| Nodes | Ready | SchedulingDisabled | NotReady |
| Namespaces | Active | Terminating | - |
| ConfigMaps | Primary accent | - | - |
| Secrets | Warning accent | - | - |
| HPAs | Always green | - | - |
| ServiceAccounts | Primary accent | - | - |
| NetworkPolicies | Always green | - | - |

#### PodList Filter Pills

Compacted filter pills to reduce toolbar height:

```svelte
<!-- Before -->
class="px-3 py-1.5 text-sm rounded-lg ..."

<!-- After -->
class="px-2.5 py-1 text-sm rounded-md ..."
```

This makes the PodList toolbar height consistent with other views that don't have filter pills.

---

### Files Modified (16 List Views)

**Workloads:**
- `PodList.svelte` - Filter pills compacted, status dot column
- `DeploymentList.svelte` - Already updated (reference pattern)
- `StatefulSetList.svelte` - Status dot column, Actions header, name styling
- `DaemonSetList.svelte` - Status dot column, Actions header, name styling
- `ReplicaSetList.svelte` - Status dot column, name styling (no actions)
- `JobList.svelte` - Status dot column, Actions header, name styling
- `CronJobList.svelte` - Status dot column, Actions header, name styling

**Network:**
- `ServiceList.svelte` - Status dot column, name styling
- `IngressList.svelte` - Status dot column (address-based), name styling
- `NetworkPolicyList.svelte` - Status dot column, name styling

**Config:**
- `ConfigMapList.svelte` - Status dot column, name styling
- `SecretList.svelte` - Status dot column (removed lock icon), name styling
- `HPAList.svelte` - Status dot column, name styling

**Storage:**
- `PVList.svelte` - Status dot column (status-based), name styling
- `PVCList.svelte` - Status dot column (status-based), name styling

**Cluster:**
- `NodeList.svelte` - Status dot column (separate), name styling
- `NamespaceList.svelte` - Status dot column (separate), name styling
- `ServiceAccountList.svelte` - Status dot column (removed user icon), name styling

---

### Design Rationale

1. **Consistent status indicators**: Users can quickly scan the leftmost column to assess health across any resource type.

2. **Clickable names**: Teal color and underline-on-hover signal interactivity, preparing for detail view navigation.

3. **Action button discoverability**: "Actions" header explicitly labels the column, reducing confusion about icon meanings.

4. **Semantic hover colors**: Action buttons change to colors that match their intent (red for delete, green for trigger, amber for restart).

5. **Compact filter pills**: Reduced padding keeps toolbar height consistent whether or not filters are present.

---

## Session: December 2025 - Unified Resource Detail Views

### Problem

Initial implementation had 18 separate detail view components (one per resource type), resulting in:
- ~18,000 lines of duplicated code
- Inconsistent patterns between components
- Maintenance burden for UI changes
- Large bundle size (~672KB for detail chunk)

### Solution: Unified ResourceDetail Component

Created a single `ResourceDetail.svelte` component (~1100 lines) that handles 15 resource types dynamically.

#### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     DetailWindow.svelte                          │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ Route by resourceType:                                      │ │
│  │  - 'pod' → PodDetail.svelte (logs, exec, containers)       │ │
│  │  - 'deployment' → DeploymentDetail.svelte (scale, restart) │ │
│  │  - 'statefulset' → StatefulSetDetail.svelte (scale, restart)│ │
│  │  - All others → ResourceDetail.svelte                       │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

#### Config-Driven Design

```typescript
const resourceConfig: Record<ResourceType, ResourceConfig> = {
  daemonset:      { displayName: 'DaemonSet',         tabs: ['overview', 'pods', 'events', 'yaml'] },
  replicaset:     { displayName: 'ReplicaSet',        tabs: ['overview', 'pods', 'events', 'yaml'] },
  job:            { displayName: 'Job',               tabs: ['overview', 'pods', 'events', 'yaml'] },
  cronjob:        { displayName: 'CronJob',           tabs: ['overview', 'events', 'yaml'] },
  service:        { displayName: 'Service',           tabs: ['overview', 'events', 'yaml'] },
  ingress:        { displayName: 'Ingress',           tabs: ['overview', 'events', 'yaml'] },
  configmap:      { displayName: 'ConfigMap',         tabs: ['overview', 'data', 'events', 'yaml'] },
  secret:         { displayName: 'Secret',            tabs: ['overview', 'data', 'events', 'yaml'] },
  networkpolicy:  { displayName: 'NetworkPolicy',     tabs: ['overview', 'ingress', 'egress', 'events', 'yaml'] },
  hpa:            { displayName: 'HPA',               tabs: ['overview', 'metrics', 'events', 'yaml'] },
  pv:             { displayName: 'PersistentVolume',  tabs: ['overview', 'events', 'yaml'], clusterScoped: true },
  pvc:            { displayName: 'PersistentVolumeClaim', tabs: ['overview', 'events', 'yaml'] },
  namespace:      { displayName: 'Namespace',         tabs: ['overview', 'events', 'yaml'], clusterScoped: true },
  node:           { displayName: 'Node',              tabs: ['overview', 'pods', 'resources', 'events', 'yaml'], clusterScoped: true },
  serviceaccount: { displayName: 'ServiceAccount',    tabs: ['overview', 'events', 'yaml'] },
};
```

#### Dynamic Backend Calls

Uses template literal pattern for dynamic command invocation:
```typescript
// Instead of: invoke('get_configmap_detail', params)
// Generic:    invoke(`get_${resourceType}_detail`, getInvokeParams())

function getInvokeParams() {
  const config = resourceConfig[resourceType];
  if (config?.clusterScoped) {
    return { contextName: context, name };  // No namespace
  }
  return { contextName: context, namespace, name };
}
```

#### Tab Content Strategy

| Tab | Content Source | Notes |
|-----|---------------|-------|
| Overview | `detail` state | Conditional blocks per resourceType |
| Pods | `pods` state | For workloads + nodes |
| Data | `detail.data` / `secretData` | ConfigMaps direct, Secrets on-demand |
| Events | `events` state | Shared EventsTable component |
| YAML | `yaml` state | Shared YamlEditorPanel component |
| Ingress/Egress | `detail.ingress_rules/egress_rules` | NetworkPolicy only |
| Metrics | `detail.metrics` | HPA only |
| Resources | `detail.capacity/allocatable` | Node only |

### Shared UI Components

Extracted reusable components to reduce duplication:

| Component | Props | Usage |
|-----------|-------|-------|
| `EventsTable.svelte` | `events, resourceType, isLoading, isDeleted, onRefresh` | All detail views |
| `YamlEditorPanel.svelte` | `yaml, resourceType, name, isDeleted` | All detail views |
| `MetadataSection.svelte` | `labels, annotations` | All detail views |
| `ConditionsTable.svelte` | `conditions` | Most detail views |

### EventsTable Redesign

Original EventsTable used tabular layout. PodDetail used card-based layout.

**Unified to card-based design:**
```
┌──────────────────────────────────────────────────────────────┐
│ Service Events                                    [↻ Refresh]│
├──────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────┐  │
│ │ [Normal]  Scheduled  ×3           2m ago                │  │
│ │ Successfully assigned default/nginx to node-1           │  │
│ │ Source: default-scheduler                               │  │
│ └─────────────────────────────────────────────────────────┘  │
│ ┌─────────────────────────────────────────────────────────┐  │
│ │ [Warning] FailedMount  ×1         5m ago                │  │
│ │ Unable to attach or mount volumes...                    │  │
│ │ Source: kubelet                                         │  │
│ └─────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Results

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Detail components | 18 files | 5 files | -13 files |
| Total lines | ~18,000 | ~3,500 | ~80% reduction |
| Bundle size (detail chunk) | 672KB | 561KB | ~17% smaller |
| Patterns to maintain | 18 | 1 | Unified |

### Design Decisions

1. **Keep 3 components separate**: Pod, Deployment, StatefulSet have complex features (logs, exec, scaling, restart) that would bloat the unified component.

2. **Config object over switch statements**: Easier to add new resources, clearer at-a-glance view of capabilities.

3. **Dynamic invoke over explicit mappings**: Template literals `get_${type}_detail` reduce boilerplate while maintaining type safety via TypeScript.

4. **Conditional rendering per resource**: `{#if resourceType === 'xxx'}` blocks within unified component keeps resource-specific UI co-located.

5. **clusterScoped flag**: Simple boolean flag cleaner than separate routing logic for PV/Namespace/Node.

### Files Changed

**Added:**
- `src/lib/components/detail/ResourceDetail.svelte` - Unified component

**Modified:**
- `src/lib/components/detail/DetailWindow.svelte` - Simplified routing
- `src/lib/components/ui/EventsTable.svelte` - Rewritten to card-based style
- All 17 list views - Added click handlers for detail windows

**Deleted:**
- 15 individual detail components (ConfigMapDetail, SecretDetail, JobDetail, etc.)

---

## Resolved: Resource Detail Phased Rollout

**Original plan** (from earlier in this doc):
- Phase 1: Pods
- Phase 2: Deployments, StatefulSets, DaemonSets
- Phase 3: Other resources

**Actual implementation**: All 18 resource types implemented in two sessions.
- Session 1: Backend commands + individual components for all types
- Session 2: Consolidated to unified ResourceDetail component

**Decision**: Skip phased rollout, implement all at once with unified architecture from the start for consistency.
