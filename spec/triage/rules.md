# Triage rules (shared: desktop, iOS, Android)

The one definition of which Kubernetes object gets which health verdict. Desktop implements it in Rust (`src-tauri/src/triage/rules.rs`, the reference implementation); the iOS and Android apps implement the same rules against the same fixtures (`docs/mobile/architecture.md`, section 3). Display (icons, colors, labels) is defined on top of these verdicts in `docs/design/redesign-spec.md`, section 11.

**Change process:** a rule change edits this file **and** the fixtures in the same PR. A rule change without a fixture change fails review. Any change under `spec/**` runs desktop, iOS and Android CI.

## Verdict

Schema: [`schema/verdict.schema.json`](schema/verdict.schema.json). Names are identical on every platform.

| Field | Values | Notes |
|---|---|---|
| `status` | `ok`, `warn`, `fail`, `unknown` | |
| `reason` | string | The label: the most specific cause (`CrashLoopBackOff`), not the phase. Kubernetes reasons keep source casing; our own words are sentence case (`2/3 available`). |
| `ruleId` | `TRI-<KIND>-<NNN>` | `-000` is the kind's no-finding baseline. |
| `since` | RFC 3339 UTC, `Z` suffix | Optional. When the condition began, if the API says so. |
| `severity` | `critical`, `error`, `warning` | Exactly when `status` is `fail` (critical/error) or `warn` (warning). |
| `tone` | `progressing`, `neutral` | Only with `status: ok`. Absent = healthy. |

Display mapping (redesign spec 11.1): `fail` → failed, `warn` → warning, `unknown` → unknown, `ok`+`progressing` → progressing, `ok`+`neutral` → neutral, `ok` → healthy.

## Picking one verdict

Evaluate every rule for the object and keep the **worst**, in this order: fail/critical, fail/error, warn, unknown, ok+progressing, ok (healthy), ok+neutral. On a tie, the rule listed first in the tables below wins (for pods: the first matching container, init containers before app containers).

## Thresholds

| Window | Value |
|---|---|
| Pod Pending / not-ready grace | 5 min |
| OOMKilled lookback | 1 h |
| Restart lookback, minimum restarts | 1 h, 3 |
| Workload zero-available grace | 1 min |
| Workload degraded grace | 3 min |
| PVC Pending grace | 2 min |
| Pod terminating, past `deletionTimestamp` | 5 min |
| StatefulSet / ReplicaSet / DaemonSet rollout or scale-up deadline | 10 min |

A window is measured from the timestamp named in the rule. **A missing timestamp counts as "longer than the window"**: we observed the problem now and can't tell when it began, so we never hide it.

**Time passes without API events.** A verdict can change with time alone (Pending → Unschedulable after 5 min, an OOMKilled warning expiring after 1h). A client that keeps a live list must re-evaluate unsettled objects on a timer, not only when a watch event arrives. Desktop re-checks every 30s inside the Rust watch task and emits only the objects whose verdict changed.

**First seen unsettled.** StatefulSets, ReplicaSets and DaemonSets have no condition transition times. Their windows are measured from `firstSeenUnsettled`: the first time this client saw the object not healthy/neutral. The clock is tracked per `metadata.uid` and cleared when the verdict settles. Without it (a client that doesn't track yet) the clock falls back to `creationTimestamp`, which is the conservative reading: an old object that starts scaling looks overdue at once. **Clients that keep state must track it.** Desktop does (`triage::UnsettledTracker`), shared by lists, watches and Pulse.

## Pods

Phase-terminal checks run first and return immediately (`Failed`, `Succeeded`, `Unknown`). Otherwise container rules and the phase rule are all evaluated and the worst wins. "Container" covers init containers too; their labels get kubectl's `Init:` prefix (`Init:CrashLoopBackOff`).

| Id | Condition | Verdict | Reason | since |
|---|---|---|---|---|
| TRI-POD-001 | phase `Failed`, reason ≠ `Evicted`, not owned by a Job | fail / error | pod `status.reason`, else `Failed` | `status.startTime`, else Ready transition |
| TRI-POD-010 | phase `Failed`, reason `Evicted`, `Terminated`, `NodeShutdown` or `Preempting`; or condition `DisruptionTarget=True` | warn | the pod reason; for DisruptionTarget the condition reason (`TerminationByKubelet`...), else `Disrupted` | DisruptionTarget transition, else `status.startTime` |
| TRI-POD-012 | phase `Failed`, owned by a Job | ok / neutral | `Failed` | `status.startTime` |
| TRI-POD-002 | phase `Unknown` | fail / error | `Unknown` | Ready transition |
| TRI-POD-000 | phase `Succeeded` | ok / neutral | `Completed` | |
| TRI-POD-003 | any container waiting `CrashLoopBackOff` | fail / error | `CrashLoopBackOff` | the `Ready` (non-True) condition transition: when the loop started; else the earliest `lastState.terminated.finishedAt` |
| TRI-POD-004 | waiting `ImagePullBackOff`, `ErrImagePull`, `InvalidImageName` | fail / error | the waiting reason | Ready transition |
| TRI-POD-005 | waiting `CreateContainerConfigError`, `CreateContainerError`, `RunContainerError` | fail / error | the waiting reason | Ready transition |
| TRI-POD-006 | current or last termination `OOMKilled` with `finishedAt` within 1 h | fail / error **and reported instead of TRI-POD-003** when a container is crashlooping; else warn | `OOMKilled` | earliest such `finishedAt` |
| TRI-POD-009 | total restarts ≥ 3 and some `lastState.terminated.finishedAt` within 1 h | warn | `{n} restarts` (n = total) | earliest such `finishedAt` |
| TRI-POD-011 | any container waiting `ContainerCreating` / `PodInitializing` | ok / progressing | the waiting reason | |
| TRI-POD-011 | `metadata.deletionTimestamp` set, ≤ 5 min past it | ok / progressing | `Terminating` | |
| TRI-POD-013 | `metadata.deletionTimestamp` more than 5 min in the past | warn | `Stuck terminating` | `deletionTimestamp` |
| TRI-POD-007 | phase `Running`, not all app containers ready, Ready condition transitioned > 5 min ago | warn | `{ready}/{total} ready` | Ready transition |
| TRI-POD-011 | same, ≤ 5 min | ok / progressing | `{ready}/{total} ready` | |
| TRI-POD-000 | phase `Running`, all app containers ready | ok | `Running` | |
| TRI-POD-008 | phase `Pending` (or empty) for > 5 min since `creationTimestamp`, `PodScheduled=False` reason `Unschedulable` | fail / error | `Unschedulable` | `creationTimestamp` |
| TRI-POD-008 | phase `Pending` for > 5 min, otherwise | warn | `Pending` | `creationTimestamp` |
| TRI-POD-011 | phase `Pending` ≤ 5 min | ok / progressing | `Pending` | |

Notes:
- TRI-POD-009 is the **cold-start** form from redesign spec 6.10: Kubernetes only keeps the last termination, so we can't count restarts inside the hour per object. The sampler-delta form ("n restarts in 1h") comes with the Pulse engine (desktop PR2/PR3) and does not change this per-object verdict.
- Total restart count alone never makes a pod failed; desktop only colors the restarts cell (≥ 5 warning, ≥ 20 danger, redesign spec 11.4).
- `deletionTimestamp` is the deadline (request time + grace period), so TRI-POD-013 measures time past the point the pod should already be gone.
- TRI-POD-012 exists so a Job's leftover failed pods don't stay red after the Job retried and succeeded. The Job's own verdict (TRI-JOB-001) carries the failure.

## Workloads

`desired` = `spec.replicas` (default 1). `available` = `status.availableReplicas` for Deployments, `readyReplicas` for StatefulSets and ReplicaSets.

**Deployment, StatefulSet, ReplicaSet** (first match wins, top to bottom):

| Id | Condition | Verdict | Reason | since |
|---|---|---|---|---|
| TRI-WL-000 | desired == 0 | ok / neutral | `Scaled to 0` | |
| TRI-WL-003 | Deployment: `Progressing=False`, reason `ProgressDeadlineExceeded` | fail / error | `Rollout stuck` | condition transition |
| TRI-WL-001 | available == 0 for > 1 min | fail / **critical** | `Unavailable` | Deployment: `Available` (non-True) transition, else creation; others: first seen unsettled |
| TRI-WL-005 | available == 0 within 1 min | ok / progressing | `Rolling out`, `Scaling` or `Starting` | |
| TRI-WL-000 | Deployment `spec.paused` | ok / neutral | `Paused` | |
| TRI-WL-005 | rolling (below), StatefulSet within 10 min | ok / progressing | `Rolling out` | |
| TRI-WL-006 | StatefulSet rolling for > 10 min | warn | `Rollout stuck` | first seen unsettled |
| TRI-WL-005 | scaling: `status.replicas < desired` (pods not created yet), ≤ 10 min | ok / progressing | `Scaling` | |
| TRI-WL-002 | scaling for > 10 min | warn | `{a}/{d} ready` | first seen unsettled |
| TRI-WL-005 | 0 < available < desired, degraded ≤ 3 min | ok / progressing | `{a}/{d} available` (`ready` for StatefulSet, ReplicaSet) | |
| TRI-WL-002 | 0 < available < desired for > 3 min, available ≤ 50% | fail / error | same label | Deployment: `Progressing.lastUpdateTime`; others: first seen unsettled |
| TRI-WL-002 | same, available > 50% | warn | same label | same |
| TRI-WL-000 | otherwise | ok | `Available` (Deployment), `Ready` (StatefulSet, ReplicaSet) | |

Rolling: Deployment `observedGeneration < generation` or `updatedReplicas < desired` (no 10-min rule: TRI-WL-003's deadline condition covers it); StatefulSet `currentRevision ≠ updateRevision` or `observedGeneration < generation`. Scaling only applies to StatefulSets and ReplicaSets.

The degraded clock is `Progressing.lastUpdateTime` for Deployments (it moves whenever new pods become available) and **first seen unsettled** for StatefulSets and ReplicaSets.

**Deployment-owned ReplicaSets** (an `ownerReferences` entry of kind `Deployment`) are capped at **warn**: a `fail` from the rules above becomes `warn` with the same rule id and reason. The Deployment's verdict owns the failure, so Pulse counts it once.

**DaemonSet** (first match wins; "unsettled" = since first seen unsettled; `since` is that time when tracked):

| Id | Condition | Verdict | Reason |
|---|---|---|---|
| TRI-WL-005 | no status yet | ok / progressing | `Starting` |
| TRI-WL-000 | `desiredNumberScheduled` == 0 | ok / neutral | `No nodes` |
| TRI-WL-004 | `numberAvailable` == 0, unsettled > 1 min | fail / **critical** | `0/{d} available` |
| TRI-WL-004 | `numberMisscheduled` > 0 | warn | `Misscheduled` |
| TRI-WL-005 | `observedGeneration < generation` or `updatedNumberScheduled < desired`, ≤ 10 min | ok / progressing | `Rolling out` |
| TRI-WL-006 | same, > 10 min | warn | `Rollout stuck` |
| TRI-WL-005 | unavailable > 0, unsettled ≤ 3 min | ok / progressing | `{a}/{d} available` |
| TRI-WL-004 | `numberUnavailable` > 0 or available < desired | warn | `{a}/{d} available` |
| TRI-WL-000 | otherwise | ok | `Available` |

## Jobs

| Id | Condition | Verdict | Reason | since |
|---|---|---|---|---|
| TRI-JOB-001 | condition `Failed=True` | fail / error | condition reason (`BackoffLimitExceeded`, `DeadlineExceeded`), else `Failed` | condition transition |
| TRI-JOB-000 | condition `Complete=True` | ok | `Complete` | |
| TRI-JOB-000 | `spec.suspend` | ok / neutral | `Suspended` | |
| TRI-JOB-003 | active > 0 and failed > 0 | warn | `{failed} failed, retrying` | |
| TRI-JOB-000 | active > 0 | ok / progressing | `Running` | |
| TRI-JOB-000 | succeeded ≥ completions | ok | `Complete` | |
| TRI-JOB-000 | otherwise | ok / progressing | `Pending` | |

Redesign spec 6.10 only raises a TRI-JOB-001 **issue** when the Job failed within 1 h or is the latest Job of its CronJob. That's an issue-level filter (Pulse, desktop PR2) applied with `since`; the per-object verdict stays `fail` so the Jobs list tells the truth. TRI-JOB-002 (CronJob: last ≥ 2 runs failed) needs the Job history and lands with the issues engine.

## Nodes

All rules are evaluated; the worst wins.

| Id | Condition | Verdict | Reason | since |
|---|---|---|---|---|
| TRI-NODE-001 | `Ready` is `False` | fail / **critical** | `NotReady` | transition |
| TRI-NODE-001 | `Ready` is `Unknown` or missing | fail / **critical** | `Unknown` | transition |
| TRI-NODE-003 | `NetworkUnavailable=True` | fail / **critical** | `NetworkUnavailable` | transition |
| TRI-NODE-002 | `MemoryPressure`, `DiskPressure` or `PIDPressure` True | fail / error | the condition type | transition |
| TRI-NODE-004 | `spec.unschedulable` (cordoned) | warn | `Cordoned` | (none in the API) |
| TRI-NODE-000 | otherwise | ok | `Ready` | |

TRI-NODE-005 (CPU/memory ≥ 90% for 5 min) and TRI-NODE-006 (root fs ≥ 90/95%) need metric samples and land with the Pulse sampler (desktop PR4). Missing metrics are `null`, never `0%`; that's a capacity contract, not a verdict.

## PersistentVolumeClaims

| Id | Condition | Verdict | Reason | since |
|---|---|---|---|---|
| TRI-PVC-000 | `Bound` | ok | `Bound` | |
| TRI-PVC-002 | `Lost` | fail / error | `Lost` | |
| TRI-PVC-003 | `Pending`, StorageClass `volumeBindingMode: WaitForFirstConsumer`, no `volume.kubernetes.io/selected-node` annotation | ok / neutral | `Waiting for consumer` | |
| TRI-PVC-001 | `Pending` (or no phase) > 2 min since creation | warn | `Pending` | `creationTimestamp` |
| TRI-PVC-001 | `Pending` ≤ 2 min | ok / progressing | `Pending` | |
| TRI-PVC-000 | any other phase | unknown | the phase | |

A WaitForFirstConsumer claim is Pending by design until a pod uses it. The scheduler sets `volume.kubernetes.io/selected-node` once a consuming pod is placed: from then on a pod is waiting, and TRI-PVC-001's window applies. The binding mode comes from the claim's StorageClass, which the caller looks up (desktop lists StorageClasses with the PVCs). When it can't (no RBAC on StorageClasses), the mode is unknown and TRI-PVC-001 applies as before.

Escalation to fail/error "when a pod is waiting on it" (redesign spec 6.10) needs the pod list and happens in issue grouping (storage absorption, desktop PR2).

## Not in this file yet

Event rules (TRI-EVT-001), issue grouping, ranking and fingerprints go into `issues.md` with `fixtures/issues/**` (desktop PR2). PDB and ResourceQuota rules come with those resource types (desktop PR8).

## Fixtures

```
fixtures/
├── meta.json                         # { "now": "2026-10-28T12:00:00Z" }: every fixture is evaluated at this instant
└── <kind>/<case>.input.json          # one raw object, exactly as the API returns it (apiVersion + kind required)
    <kind>/<case>.expected.json       # the exact verdict, compared as JSON (absent optional fields must be absent)
    <kind>/<case>.context.json        # optional: what the caller observed besides the object
```

`context.json` keys (unknown keys are an error): `firstSeenUnsettled` (RFC 3339, the tracker's clock for StatefulSet, ReplicaSet and DaemonSet) and `storageClassBindingMode` (the PVC's StorageClass `volumeBindingMode`). No context file means no tracking and an unknown binding mode.

Kinds: `pod`, `deployment`, `statefulset`, `daemonset`, `replicaset`, `job`, `node`, `pvc`. Runners dispatch on the input's `kind`.

Every runner must:
1. Evaluate every `*.input.json` at `meta.json`'s `now` and compare to its `*.expected.json` exactly.
2. Fail on an input without an expected file, or an expected or context file without an input.
3. Fail if a rule id it implements has no fixture.

Runners: desktop `cargo test` (`src-tauri/src/triage/fixtures.rs`); iOS Swift Package test; Android `core/triage` JVM test.
