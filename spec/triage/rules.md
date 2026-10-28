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

A window is measured from the timestamp named in the rule. **A missing timestamp counts as "longer than the window"**: we observed the problem now and can't tell when it began, so we never hide it.

## Pods

Phase-terminal checks run first and return immediately (`Failed`, `Succeeded`, `Unknown`). Otherwise container rules and the phase rule are all evaluated and the worst wins. "Container" covers init containers too; their labels get kubectl's `Init:` prefix (`Init:CrashLoopBackOff`).

| Id | Condition | Verdict | Reason | since |
|---|---|---|---|---|
| TRI-POD-001 | phase `Failed`, reason ≠ `Evicted`, not owned by a Job | fail / error | pod `status.reason`, else `Failed` | `status.startTime`, else Ready transition |
| TRI-POD-010 | phase `Failed`, reason `Evicted` | warn | `Evicted` | `status.startTime` |
| TRI-POD-012 | phase `Failed`, owned by a Job | ok / neutral | `Failed` | `status.startTime` |
| TRI-POD-002 | phase `Unknown` | fail / error | `Unknown` | Ready transition |
| TRI-POD-000 | phase `Succeeded` | ok / neutral | `Completed` | |
| TRI-POD-003 | any container waiting `CrashLoopBackOff` | fail / error | `CrashLoopBackOff` | earliest `lastState.terminated.finishedAt` among crashlooping containers, else Ready transition |
| TRI-POD-004 | waiting `ImagePullBackOff`, `ErrImagePull`, `InvalidImageName` | fail / error | the waiting reason | Ready transition |
| TRI-POD-005 | waiting `CreateContainerConfigError`, `CreateContainerError`, `RunContainerError` | fail / error | the waiting reason | Ready transition |
| TRI-POD-006 | current or last termination `OOMKilled` with `finishedAt` within 1 h | fail / error **and reported instead of TRI-POD-003** when a container is crashlooping; else warn | `OOMKilled` | earliest such `finishedAt` |
| TRI-POD-009 | total restarts ≥ 3 and some `lastState.terminated.finishedAt` within 1 h | warn | `{n} restarts` (n = total) | earliest such `finishedAt` |
| TRI-POD-011 | any container waiting `ContainerCreating` / `PodInitializing` | ok / progressing | the waiting reason | |
| TRI-POD-011 | `metadata.deletionTimestamp` set | ok / progressing | `Terminating` | |
| TRI-POD-007 | phase `Running`, not all app containers ready, Ready condition transitioned > 5 min ago | warn | `{ready}/{total} ready` | Ready transition |
| TRI-POD-011 | same, ≤ 5 min | ok / progressing | `{ready}/{total} ready` | |
| TRI-POD-000 | phase `Running`, all app containers ready | ok | `Running` | |
| TRI-POD-008 | phase `Pending` (or empty) for > 5 min since `creationTimestamp`, `PodScheduled=False` reason `Unschedulable` | fail / error | `Unschedulable` | `creationTimestamp` |
| TRI-POD-008 | phase `Pending` for > 5 min, otherwise | warn | `Pending` | `creationTimestamp` |
| TRI-POD-011 | phase `Pending` ≤ 5 min | ok / progressing | `Pending` | |

Notes:
- TRI-POD-009 is the **cold-start** form from redesign spec 6.10: Kubernetes only keeps the last termination, so we can't count restarts inside the hour per object. The sampler-delta form ("n restarts in 1h") comes with the Pulse engine (desktop PR2/PR3) and does not change this per-object verdict.
- Total restart count alone never makes a pod failed; desktop only colors the restarts cell (≥ 5 warning, ≥ 20 danger, redesign spec 11.4).
- TRI-POD-012 exists so a Job's leftover failed pods don't stay red after the Job retried and succeeded. The Job's own verdict (TRI-JOB-001) carries the failure.

## Workloads

`desired` = `spec.replicas` (default 1). `available` = `status.availableReplicas` for Deployments, `readyReplicas` for StatefulSets and ReplicaSets.

**Deployment, StatefulSet, ReplicaSet** (first match wins, top to bottom):

| Id | Condition | Verdict | Reason | since |
|---|---|---|---|---|
| TRI-WL-000 | desired == 0 | ok / neutral | `Scaled to 0` | |
| TRI-WL-003 | Deployment: `Progressing=False`, reason `ProgressDeadlineExceeded` | fail / error | `Rollout stuck` | condition transition |
| TRI-WL-001 | available == 0 for > 1 min | fail / **critical** | `Unavailable` | Deployment: `Available` (non-True) transition, else creation |
| TRI-WL-005 | available == 0 within 1 min | ok / progressing | `Rolling out` if rolling, else `Starting` | |
| TRI-WL-005 | rolling: Deployment `observedGeneration < generation` or `updatedReplicas < desired`; StatefulSet `currentRevision ≠ updateRevision` or `observedGeneration < generation` | ok / progressing | `Rolling out` | |
| TRI-WL-005 | 0 < available < desired, degraded ≤ 3 min | ok / progressing | `{a}/{d} available` (`ready` for StatefulSet, ReplicaSet) | |
| TRI-WL-002 | 0 < available < desired for > 3 min, available ≤ 50% | fail / error | same label | Deployment: `Progressing.lastUpdateTime` |
| TRI-WL-002 | same, available > 50% | warn | same label | same |
| TRI-WL-000 | otherwise | ok | `Available` (Deployment), `Ready` (StatefulSet, ReplicaSet) | |

The "degraded" clock is `Progressing.lastUpdateTime` for Deployments (it moves whenever new pods become available) and `creationTimestamp` for StatefulSets and ReplicaSets, which carry no transition times.

**DaemonSet** (first match wins):

| Id | Condition | Verdict | Reason |
|---|---|---|---|
| TRI-WL-005 | no status yet | ok / progressing | `Starting` |
| TRI-WL-000 | `desiredNumberScheduled` == 0 | ok / neutral | `No nodes` |
| TRI-WL-004 | `numberAvailable` == 0, created > 1 min ago | fail / **critical** | `0/{d} available` |
| TRI-WL-004 | `numberMisscheduled` > 0 | warn | `Misscheduled` |
| TRI-WL-005 | `observedGeneration < generation` or `updatedNumberScheduled < desired` | ok / progressing | `Rolling out` |
| TRI-WL-005 | unavailable > 0, created ≤ 3 min ago | ok / progressing | `{a}/{d} available` |
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
| TRI-PVC-001 | `Pending` (or no phase) > 2 min since creation | warn | `Pending` | `creationTimestamp` |
| TRI-PVC-001 | `Pending` ≤ 2 min | ok / progressing | `Pending` | |
| TRI-PVC-000 | any other phase | unknown | the phase | |

Escalation to fail/error "when a pod is waiting on it" (redesign spec 6.10) needs the pod list and happens in issue grouping (storage absorption, desktop PR2).

## Not in this file yet

Event rules (TRI-EVT-001), issue grouping, ranking and fingerprints go into `issues.md` with `fixtures/issues/**` (desktop PR2). PDB and ResourceQuota rules come with those resource types (desktop PR8).

## Fixtures

```
fixtures/
├── meta.json                         # { "now": "2026-10-28T12:00:00Z" }: every fixture is evaluated at this instant
└── <kind>/<case>.input.json          # one raw object, exactly as the API returns it (apiVersion + kind required)
    <kind>/<case>.expected.json       # the exact verdict, compared as JSON (absent optional fields must be absent)
```

Kinds: `pod`, `deployment`, `statefulset`, `daemonset`, `replicaset`, `job`, `node`, `pvc`. Runners dispatch on the input's `kind`.

Every runner must:
1. Evaluate every `*.input.json` at `meta.json`'s `now` and compare to its `*.expected.json` exactly.
2. Fail on an input without an expected file, or an expected file without an input.
3. Fail if a rule id it implements has no fixture.

Runners: desktop `cargo test` (`src-tauri/src/triage/fixtures.rs`); iOS Swift Package test; Android `core/triage` JVM test.
