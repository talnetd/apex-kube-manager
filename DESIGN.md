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
