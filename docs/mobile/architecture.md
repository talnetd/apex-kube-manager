# Apex Kube Manager Mobile: Architecture

Status: Proposed (2026-10-28)
Scope: native iOS and Android apps, v1 "on-call triage companion"
Owner decisions this builds on: fully native (SwiftUI / Jetpack Compose), read-mostly, phone talks directly to the API server, token or client-cert auth only, credentials in Keychain / Keystore, same repo.

---

## 1. TL;DR

- **No shared Rust core in v1.** Each app has a thin native Kubernetes REST client (URLSession / OkHttp). Triage rules stay consistent through a **shared spec plus golden test fixtures** in `spec/triage/`, executed by Rust, Swift and Kotlin tests.
- New code lives under `apps/ios`, `apps/android` and `spec/`. **Desktop files do not move** in this plan. A move to `apps/desktop` is optional and, if wanted, is its own PR after the deps-upgrade, macOS-signing and redesign PRs merge.
- **No background alerting in v1.** iOS and Android give no reliable background polling. v1 is "open it when you're paged"; it accepts deep links from your pager. Real push alerts need a server-side component and are a product decision (section 12).

---

## 2. Decision: shared Rust core vs native thin clients

### Options

| | A. Shared core (`crates/apex-core` + UniFFI, kube-rs + rustls) | B. Native thin clients + shared spec/fixtures | C. Hybrid: native networking + tiny pure-Rust `apex-triage` via UniFFI |
|---|---|---|---|
| Build complexity | High: xcframework (device + 2 sim slices), 4 Android ABIs via cargo-ndk, UniFFI codegen in both builds, tokio runtime bootstrapped per app | Low: Xcode + Gradle only | Medium: same FFI toolchain as A, smaller crate |
| Binary size | +8-15 MB per ABI (tokio, hyper, rustls, kube, k8s-openapi with all API types) | Native libs only, ~0 added | +1-2 MB per ABI |
| Triage logic reuse | Full | Re-implemented twice, verified against fixtures | Full for rules, none for I/O |
| Platform fit | Fights the platform: own TLS stack (no system trust evaluation, proxies, VPN/MDM per-app VPN), own HTTP stack (no background URLSession), FFI async bridging | Native TLS, Keychain/Keystore, background session, OS proxy and VPN all "just work" | Native I/O, FFI only for pure functions |
| Testability | Rust tests good; FFI boundary and mobile runtime issues hard to debug | XCTest / JUnit per platform + shared fixtures | Rust tests + FFI smoke tests |
| CI cost | Rust cross-compiles on macOS runners for iOS + NDK builds for Android on every mobile PR | Plain `xcodebuild test`, `gradle test` | Same cross-compile cost as A, smaller |
| Conflict with in-flight work | Extracting a core means carving up the 6k-line `kubernetes.rs` while the deps-upgrade PR bumps kube-rs and the redesign starts | None | Small: new crate, no extraction |

### Decision: **B** (native thin clients + shared spec and golden fixtures)

Rationale:
1. **There is little triage logic to share today.** Desktop health rules are thin and interleaved with kube-rs I/O (e.g. pod OK/FAIL is mostly `phase == "Running"` inside list functions, `get_pod_phase`, `PulseMetrics` counting). Extracting a core buys reuse of maybe a few hundred lines, at the cost of refactoring the hottest file in the repo during a kube-rs upgrade.
2. **Mobile needs ~15 REST calls**, not the whole kube-rs surface: list pods/deployments/statefulsets/daemonsets/nodes/events, read logs, get metrics, PATCH for restart/scale, DELETE pod, SelfSubjectAccessReview. Codable / kotlinx.serialization with tolerant decoding handles this in a few hundred lines per platform.
3. **Security and networking are native anyway.** Keychain/Keystore, custom CA trust, client identities (SecIdentity / KeyManager), per-app VPN and corporate proxies are platform APIs. A Rust TLS stack would bypass several of them.
4. **The node-metrics degrade logic mostly does not apply to mobile v1.** The kubelet disk sweep (`classify_sweep`, 8 concurrent probes via `nodes/proxy`) is a desktop feature. On a phone over cellular we do CPU/memory from metrics-server only and skip disk. The pattern we keep is "each source degrades independently to `null`, never a confident 0%", which is a spec rule, not shared code.
5. **CI stays cheap and fast.** The repo is public, so macOS minutes are free, but a Rust cross-compile per PR still adds 10+ minutes of wall time.

Trade-off: we accept writing the triage rules twice more (Swift, Kotlin), and we pay for that with a fixture suite that fails CI the moment any platform drifts.

**Revisit trigger (move to C):** if the triage rule set grows past ~30 rules, or we ship two drift bugs that fixtures did not catch. C is the upgrade path because rules are already written as pure functions over decoded objects.

---

## 3. Keeping triage rules consistent

```
spec/triage/
├── rules.md                 # human-readable rules, one ID per rule (TRI-POD-001 ...)
├── schema/verdict.schema.json
└── fixtures/
    ├── meta.json                              # { "now": "2026-10-28T12:00:00Z" }: fixed evaluation time
    ├── pod/crashloop-backoff.input.json       # raw K8s object as the API returns it (apiVersion + kind)
    ├── pod/crashloop-backoff.expected.json    # { "status": "fail", "reason": "CrashLoopBackOff", "ruleId": "TRI-POD-003",
    │                                          #   "since": "2026-10-28T11:58:00Z", "severity": "error" }
    ├── deployment/unavailable-replicas.*.json
    ├── node/not-ready.*.json
    └── ...                                    # pod, deployment, statefulset, daemonset, replicaset, job, node, pvc
```

**Status (2026-10-28):** `spec/triage/` was created by the desktop redesign track (redesign PR1b) with 80 per-object fixtures, and desktop `cargo test` runs them. Mobile PR 2 **adopts** these files instead of creating them: it adds the Swift and Kotlin runners and may add fixtures (with matching `rules.md` edits). Event and issue-grouping fixtures (`events/`, `issues/`) come with desktop PR2; `node/metrics-server-absent` is a capacity contract (usage `null`, never `0`), not a verdict, and is tested by the KubeClient decoding tests (PR 4) instead.

- **Verdict model** (identical names on all three platforms, `schema/verdict.schema.json`): `status: ok | warn | fail | unknown`, `reason: String`, `ruleId: String`, `since: timestamp?` (RFC 3339 UTC), plus the two fields the redesign spec (6.10, 11.1) added: `severity: critical | error | warning` (present exactly when status is `fail` or `warn`) and `tone: progressing | neutral` (only with `ok`; absent = healthy). Display: `fail` → failed, `warn` → warning, `unknown` → unknown, `ok`+progressing → progressing, `ok`+neutral → neutral, `ok` → healthy. Mobile renders these six states with the shapes in redesign spec 11.1.
- **Initial rules** (v1, now specified in full in `rules.md`): pod phase Failed/Unknown, container waiting reasons (CrashLoopBackOff, ImagePullBackOff, ErrImagePull, CreateContainerConfigError, OOMKilled as last termination), not-ready > N minutes while Running, Pending > N minutes, restarts in last hour; deployment/statefulset/daemonset available < desired, Progressing=False, stuck rollout; node Ready != True, MemoryPressure/DiskPressure/PIDPressure, unschedulable (cordoned = warn); events type=Warning within window, deduped by involvedObject + reason.
- **Runners:** `src-tauri` Rust test (`src-tauri/src/triage/fixtures.rs`, live), Swift Package test target, Kotlin JVM unit test. Each reads `spec/triage/fixtures/**`, evaluates at `meta.json`'s `now`, and asserts exact JSON equality, passing the optional `<case>.context.json` (`firstSeenUnsettled`, `storageClassBindingMode`) to the rule. Each also fails on an orphan input/expected/context file and on an implemented rule id with no fixture (`rules.md`, "Fixtures").
- **Mobile must also:** re-evaluate unsettled verdicts on the polling tick even when an object did not change (time windows age), and track "first seen unsettled" per uid for StatefulSets, ReplicaSets and DaemonSets (`rules.md`, "Thresholds"). Without tracking, the rules fall back to `creationTimestamp`, so an old object that starts scaling looks overdue at once.
- **Desktop adoption:** done in redesign PR1b. Desktop computes verdicts in Rust only (`src-tauri/src/triage/`) and attaches `verdict` to every pod, workload, node and PVC list payload; Pulse OK:FAIL counts use them.
- **Change process:** a rule change is a PR that edits `rules.md` + fixtures; path filters run all three platforms' tests.

---

## 4. Monorepo layout

```
apex-kube-manager/
├── src/                         # desktop frontend (unchanged)
├── src-tauri/                   # desktop backend (unchanged; gains triage/ fixture test later)
├── apps/
│   ├── ios/
│   │   ├── project.yml          # XcodeGen spec; .xcodeproj is generated, not committed
│   │   ├── ApexKube/            # app target (SwiftUI)
│   │   │   ├── App/             # entry, navigation, deep links
│   │   │   ├── Features/        # Clusters, Import, Triage, PodDetail, Logs, Workload, Nodes, Events, Settings
│   │   │   └── DesignSystem/    # generated from spec/design-tokens
│   │   ├── Packages/ApexKubeCore/   # SwiftPM, no UIKit/SwiftUI deps
│   │   │   ├── Sources/Kubeconfig/
│   │   │   ├── Sources/Credentials/ # Keychain wrapper
│   │   │   ├── Sources/KubeClient/  # URLSession + TLS delegate
│   │   │   ├── Sources/Triage/      # pure rules
│   │   │   └── Tests/               # incl. fixture conformance
│   │   └── ApexKubeUITests/
│   └── android/
│       ├── settings.gradle.kts
│       ├── gradle/libs.versions.toml
│       ├── app/                 # Compose app, navigation, DI (Hilt)
│       ├── core/kubeconfig/     # pure Kotlin/JVM
│       ├── core/credentials/    # Keystore
│       ├── core/kubeclient/     # OkHttp + kotlinx.serialization
│       ├── core/triage/         # pure Kotlin/JVM, fixture tests
│       ├── core/designsystem/   # generated tokens
│       └── feature/{clusters,import,triage,pod,logs,workload,nodes,events,settings}/
├── spec/
│   ├── triage/                  # rules + golden fixtures (section 3)
│   ├── design-tokens/tokens.json  # shared with desktop redesign (section 8)
│   └── rbac/apex-mobile-triage.yaml  # recommended least-privilege role
├── docs/mobile/architecture.md
└── .github/workflows/
    ├── ci.yml                   # desktop; gains paths-ignore for apps/** (section 9)
    ├── release.yml              # desktop, tag v*.*.* (untouched)
    ├── ios.yml                  # new
    ├── android.yml              # new
    └── mobile-release.yml       # later: tags ios-v* / android-v*
```

Why XcodeGen: `.pbxproj` merge conflicts are the number-one monorepo pain on iOS; a YAML spec avoids them.

**Identifiers:**
| Platform | ID |
|---|---|
| Desktop (existing) | `com.apex.kubemanager` |
| iOS bundle id | `com.apex.kubemanager.mobile` |
| Android applicationId | `com.apex.kubemanager.mobile` |
| iOS extensions later | `com.apex.kubemanager.mobile.widget` |
| Keychain access group | `ZYN774L23Z.com.apex.kubemanager.mobile` |
| URL scheme | `apexkube://` (+ universal links later) |

A distinct `.mobile` suffix keeps the iOS App ID separate from the macOS Developer ID app under the same team, so desktop signing is unaffected.

---

## 5. Module structure and data flow

Both apps use the same layers; names below are iOS / Android.

```
UI (SwiftUI View / Composable)
   │ observes
ViewModel (@Observable / ViewModel + StateFlow)
   │ calls
ClusterRepository (per active cluster; polling, caching, cancellation)
   │                 │
KubeClient          Triage (pure: K8s JSON models -> Verdicts)
   │
URLSession / OkHttp  ── TLS trust from Credentials ──  Keychain / Keystore
```

Rules:
- **KubeClient** decodes only the fields triage and screens need, tolerantly (unknown fields ignored, missing fields optional). No full OpenAPI models.
- **Polling** only in foreground: triage home every 15s, detail screens 10s, paused on background and when the screen is not visible. One in-flight request per resource; a response for a cluster that is no longer active is dropped (same invariant as desktop `loadNodeMetrics`).
- **List cost control:** `limit=500` with `continue`, `resourceVersion=0` (served from apiserver watch cache, not etcd), namespace-scoped when a namespace is selected, `fieldSelector=type=Warning` for events. Show a "large cluster, showing first N" banner rather than paging forever over cellular.
- **Metrics:** `metrics.k8s.io` nodes (and pods on detail). If absent or 403, values are `null` and the UI shows `–`. No kubelet disk probing on mobile.
- **Permissions:** on cluster connect, run `SelfSubjectRulesReview` / `SelfSubjectAccessReview` for the v1 actions and hide actions the token cannot perform.
- **Caching:** in-memory only for cluster data. Nothing about cluster state is written to disk in v1 (keeps the data-at-rest story to "only credentials, encrypted").

### API calls in v1

| Purpose | Request |
|---|---|
| Version / reachability | `GET /version` |
| Pods | `GET /api/v1/pods` or `/namespaces/{ns}/pods` |
| Workloads | `GET /apis/apps/v1/{deployments,statefulsets,daemonsets}` |
| Nodes | `GET /api/v1/nodes` |
| Warning events | `GET /api/v1/events?fieldSelector=type=Warning` |
| Logs | `GET /api/v1/namespaces/{ns}/pods/{pod}/log?container=&tailLines=500&previous=` |
| Metrics | `GET /apis/metrics.k8s.io/v1beta1/nodes` |
| Restart | `PATCH` strategic-merge `spec.template.metadata.annotations["kubectl.kubernetes.io/restartedAt"]` (same as desktop) |
| Scale | `PATCH .../{deployments,statefulsets}/{name}/scale` |
| Delete pod | `DELETE /api/v1/namespaces/{ns}/pods/{pod}` |
| Permissions | `POST /apis/authorization.k8s.io/v1/selfsubjectaccessreviews` |

---

## 6. Auth, kubeconfig import, credential storage

### Import paths (v1)
1. **Files picker** (UIDocumentPicker / Storage Access Framework).
2. **Share sheet / "Open in Apex"** (UTI / intent filter for `*.yaml`, `*.kubeconfig`, `config`).
3. **Paste** from clipboard (clipboard cleared after import, with notice).

**No QR in v1.** Kubeconfigs with inline certs often exceed QR capacity (~3 KB), and a QR of a long-lived credential on a screen is a shoulder-surfing and screenshot risk. Revisit for "server URL + short-lived token" only.

### Parsing rules
- Show all contexts found; user picks which to import. Each imported context becomes a "cluster profile".
- Supported: `token`, `client-certificate-data` + `client-key-data`, `certificate-authority-data`, `tls-server-name`, `proxy-url` (http only, warn).
- Rejected with a clear message and a fix:
  - `exec` / `auth-provider` (aws, gcloud, kubelogin, oidc): "Not supported on mobile. Create a ServiceAccount token" plus a copy-able snippet using `spec/rbac/apex-mobile-triage.yaml` and `kubectl create token ... --duration=...`.
  - File path references (`certificate-authority: /Users/...`): "Embed with `kubectl config view --raw --flatten --minify`".
  - `username`/`password` basic auth.
- Token expiry: if the token is a JWT, decode `exp` (no verification needed) and show "expires in N days" plus a warning badge near expiry.

### Storage
- **iOS:** the parsed profile secret (token or key + cert, CA) stored as one Keychain item, `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`, `kSecAttrSynchronizable = false` (never iCloud Keychain, never in backups). Client identity: cert and key added to Keychain, `SecIdentity` fetched for `URLCredential(identity:)`. Non-secret profile metadata (name, server URL, namespace) in app storage.
- **Android:** AES-256-GCM key in Android Keystore (StrongBox when available, `setUnlockedDeviceRequired(true)`), wrapping an encrypted blob in app-private storage (DataStore). Do not use the deprecated Jetpack `security-crypto`. `android:allowBackup="false"` and data-extraction rules exclude credentials. Client certs: build an in-memory `KeyStore` (PKCS#8; convert PKCS#1/SEC1 keys on import) for OkHttp's `KeyManager`.
- **App lock:** optional Face ID / BiometricPrompt on app open (default on), and **always** required before a mutating action.
- Raw imported kubeconfig is never persisted; only the extracted fields for chosen contexts.

### TLS
- `certificate-authority-data` present: trust **only** that CA for that cluster (iOS: `SecTrustSetAnchorCertificates` + `SecTrustSetAnchorCertificatesOnly(true)` in the session delegate; Android: per-cluster `X509TrustManager` built from the CA). Honour `tls-server-name` for hostname checks.
- No CA: system trust store (works for EKS/GKE/AKS public endpoints).
- **`insecure-skip-tls-verify: true`:** refused by default. The user can opt in per cluster after a full-screen warning; the cluster then shows a persistent red "TLS not verified" banner and **mutating actions are disabled** for it. Bearer tokens are sent to whoever answers, so we never make this silent.
- Plain `http://` servers rejected. ATS stays on (custom trust via the delegate is compatible); Android `cleartextTrafficPermitted=false`.

### Other security notes
- Secrets are out of scope: the app never lists or reads `secrets`, and the recommended RBAC role excludes them.
- Logs may contain sensitive data: "Copy/Share logs" shows a one-time warning; app switcher snapshot is blurred (iOS `scenePhase` overlay, Android `FLAG_SECURE` on logs screen).
- No analytics or crash reporting that could capture server URLs or log text in v1.

---

## 7. v1 screens

1. **Clusters**: imported profiles, last-seen status, add.
2. **Import**: pick/paste, choose contexts, test connection, permissions summary.
3. **Triage (home)**: per cluster + namespace filter; problem counts, then sections.
4. **Pod detail**: status verdict, containers, last termination, events; Logs; Delete.
5. **Logs**: tail 500, previous container toggle, container switch, search, follow (poll), copy/share.
6. **Workload detail** (Deployment / StatefulSet / DaemonSet): replicas, conditions, pods, events; Restart, Scale (not DaemonSet).
7. **Nodes**: list with Ready, pressure, cordon state, CPU/mem bars (`–` when unknown); node detail with conditions and pods.
8. **Events**: Warning feed, grouped by object, time window.
9. **Settings**: app lock, polling interval, remove cluster, RBAC snippet, about.

Brief wireframes:

```
┌ Triage ─────────── prod-eu ▾ ┐   ┌ Pod: api-7f9c ───────────┐
│ all namespaces ▾   ⟳ 12s ago │   │ ● FAIL CrashLoopBackOff   │
│ ┌──────┐┌──────┐┌──────┐     │   │ ns payments · node ip-10… │
│ │ 4    ││ 2    ││ 1    │     │   │ restarts 14 (6 in 1h)     │
│ │ pods ││ wklds││ nodes│     │   │ last exit: OOMKilled 137  │
│ └──────┘└──────┘└──────┘     │   │───────────────────────────│
│ FAILING PODS                 │   │ Containers  ▸ api ● ▸ side│
│ ● api-7f9c  CrashLoop   14↻ │   │ Events (3)                 │
│ ● worker-2  ImagePull        │   │───────────────────────────│
│ UNHEALTHY WORKLOADS          │   │ [ Logs ]     [ Delete… ]  │
│ ◐ api  deploy  1/3 avail     │   └───────────────────────────┘
│ NODES                        │
│ ● ip-10-0-3-7  NotReady 4m   │   ┌ Restart deployment? ─────┐
│ WARNING EVENTS (last 1h)     │   │ api · payments · prod-eu  │
│ ▲ FailedScheduling  ×12      │   │ Type the name to confirm  │
└ Triage  Nodes  Events  ⚙ ────┘   │ [ api            ]        │
                                    │ Face ID required          │
                                    │ [Cancel]      [Restart]   │
                                    └───────────────────────────┘
```

Confirmation rules: every action shows cluster + namespace + object name; Delete pod and Scale-to-0 require typing the name; all require biometric; production-looking context names (`prod`, `prd`) get a red cluster chip throughout.

---

## 8. Visual consistency with the desktop redesign

`docs/design/redesign-spec.md` did not exist when this was written. Proposal to the redesign track:
- Put tokens in `spec/design-tokens/tokens.json` (color, radius, spacing, type scale, status colors) as the single source. Desktop generates Tailwind theme; iOS generates a Swift `Color`/`Font` file; Android generates a Compose `ColorScheme` + `Typography` (small script, run in CI to verify generated files are up to date).
- Status semantics shared with triage verdicts: **superseded by redesign spec section 11** (the shared display contract). Six states derived from `status` + `tone` (failed red octagon ✕, warning amber triangle !, progressing blue half circle, unknown grey ?, healthy green ✓ circle, neutral empty circle), with the SF Symbol / Material equivalents listed in 11.1. Color is never the only signal; `severity` only affects ranking and the CRIT/ERR tag.
- Mobile uses native navigation and controls (tab bar, sheets, swipe actions) and only shares tokens and status language, not layouts. "Compact/comfortable density" maps to Dynamic Type / font scale on mobile, not a separate setting.

---

## 9. CI plan

| Workflow | Trigger paths | Runner | Steps |
|---|---|---|---|
| `ios.yml` | `apps/ios/**`, `spec/**`, `.github/workflows/ios.yml` | `macos-15` | XcodeGen generate, `swift test` (ApexKubeCore incl. fixtures), `xcodebuild test` on iOS Simulator, SwiftLint (warn-only at first) |
| `android.yml` | `apps/android/**`, `spec/**`, `.github/workflows/android.yml` | `ubuntu-24.04` | setup-java 17, `gradle/actions/setup-gradle` cache, `./gradlew lint test assembleDebug` |
| `ci.yml` (desktop) | `paths-ignore: [apps/**, docs/mobile/**]` (`spec/**` is **not** ignored since redesign PR1b: desktop tests the fixtures) | unchanged | unchanged |

Notes:
- The repo is public, so macOS minutes are free; keep `concurrency: cancel-in-progress` like `ci.yml` anyway.
- `master` has no branch protection today, so `paths-ignore` is safe. If required checks are added later, a skipped workflow blocks merges; switch to one always-running gate job using `dorny/paths-filter`.
- The `ci.yml` edit is three lines. It must rebase on the deps-upgrade PR if that PR edits `ci.yml`. `release.yml` is not touched by any mobile PR.

### Signing and distribution (later, PR8)
- **iOS:** Apple team `ZYN774L23Z`. App Store Connect API key (`ASC_KEY_ID`, `ASC_ISSUER_ID`, `ASC_KEY_P8` as Actions secrets), `xcodebuild archive -allowProvisioningUpdates` with automatic signing, upload to TestFlight (internal testers). fastlane optional; start without it.
- **Android:** upload keystore as base64 secret, Play App Signing enabled, Play service-account JSON secret, upload AAB to the **internal** track.
- `mobile-release.yml` triggers on tags `ios-v*` / `android-v*`, which do not match desktop's `v*.*.*`, so desktop releases are never triggered by mobile tags (and vice versa).
- Coordinate with the macOS signing PR (ci-signing): reuse the same team, separate certificates (Apple Distribution vs Developer ID Application).

---

## 10. Background work and alerts

- **iOS:** no long-running background work. `BGAppRefreshTask` is best-effort; the OS decides when (often hours apart, never in Low Power Mode, depends on usage). Background URLSession helps downloads, not polling.
- **Android:** `WorkManager` periodic minimum is 15 minutes, deferred further by Doze and OEM battery managers.
- **Implication:** the phone cannot be the alerting system. v1 has **no background polling and no alerts**. The value proposition is "you got paged, you open Apex, you see why in 5 seconds".
- **v1 helps the paged flow:** deep links `apexkube://cluster/{profile}/ns/{ns}/pod/{name}` (and workload/node) so PagerDuty / Alertmanager / Slack links open the right screen. The link carries no credentials; it matches a locally imported profile by cluster name or server URL.
- **Later options** (product decision): (a) opportunistic BGAppRefresh/WorkManager local notification labeled "best effort, not a pager"; (b) real push via a small in-cluster agent or Alertmanager webhook relay sending APNs/FCM, which is a new server-side component with its own security review.

---

## 11. Phased PR plan

Each phase may be split into `-ios` and `-android` PRs so two engineers can work in parallel. Every PR gets its Claude Code review handled before merge.

| PR | Content | Acceptance criteria |
|---|---|---|
| **1. Scaffold** | `apps/ios` (XcodeGen, SwiftUI hello world, `ApexKubeCore` package with one test), `apps/android` (Compose hello world, `core/triage` module with one test), `ios.yml`, `android.yml`, `ci.yml` `paths-ignore`, this doc | Both workflows green on the PR; a mobile-only commit does not trigger desktop CI and a desktop-only commit does not trigger mobile CI (verified in the PR); no other desktop files changed; app ids as in section 4 |
| **2. Triage spec + fixtures** | Adopt `spec/triage/` (created by redesign PR1b: `rules.md`, schema with `severity`/`tone`, 80 fixtures); Swift and Kotlin rule engines passing them | Fixture tests run in both mobile workflows; changing one expected file fails both |
| **2b. Desktop fixture runner** | **Done in redesign PR1b**: `src-tauri/src/triage/` + fixture test; `ci.yml` includes `spec/**` | Desktop verdicts are live in list payloads and Pulse counts |
| **3. Kubeconfig + credentials (token auth)** | Parser, import (picker, share sheet, paste), Keychain/Keystore store, TLS CA trust, insecure-skip handling, connection test | Unit tests over fixture kubeconfigs (exec, file paths, insecure, multi-context, expired JWT); credentials verified not in backups; connects to a kind cluster in CI-free manual test |
| **4. Triage home (read-only)** | KubeClient lists, repository polling with in-flight + cluster guards, triage screen, namespace filter, cluster list | Mock-server tests (URLProtocol / MockWebServer) for decoding and 403/404 degradation; metrics absent renders `–` |
| **5. Detail screens** | Pod detail, logs, workload detail, nodes, events, deep links | Logs previous/container switch work; deep link opens correct screen or a clear "cluster not imported" |
| **6. Safe actions** | Restart, scale, delete pod; SSAR gating; biometric + typed confirmation; insecure clusters read-only | Actions hidden without RBAC; confirmations match section 7; mock-server tests assert exact PATCH/DELETE bodies |
| **7. Client-cert auth** | PEM key formats, SecIdentity / KeyManager | Works with kind and k3s admin certs; keys never leave secure storage in plaintext |
| **8. Signing + distribution** | `mobile-release.yml`, TestFlight internal, Play internal track | Tag `ios-v0.1.0` produces a TestFlight build; `android-v0.1.0` an internal-track release; desktop release untouched |
| **9. (optional) Best-effort refresh** | BGAppRefresh / WorkManager local notification | Only if the owner opts in (section 12) |
| **Desktop move (optional)** | `src/` + `src-tauri/` to `apps/desktop/` | Own PR, only after deps-upgrade, macOS signing and the redesign merge; pure move + path fixes |

---

## 12. Risks

| Risk | Mitigation |
|---|---|
| Triage rule drift across three implementations | Golden fixtures run on every platform; revisit trigger to option C |
| Most real clusters use exec auth (EKS/GKE/AKS), so import fails for many users | Clear rejection message + RBAC manifest + `kubectl create token` snippet; product question on SSO later |
| Long-lived ServiceAccount tokens on phones | Recommend `--duration`, show expiry, biometric lock, `ThisDeviceOnly`, least-privilege role without secrets |
| Large clusters over cellular | `limit`, `resourceVersion=0`, namespace scoping, foreground-only polling |
| Client-cert key format handling (PKCS#1/SEC1/PKCS#8, EC vs RSA) | Isolated in PR7 with fixture keys |
| Users expect alerts | Product messaging: "companion, not a pager"; deep links from the real pager |

## 13. Open product questions

1. Is v1 OK with **no alerts at all** (pager deep links only), or do you want best-effort local notifications, or a server-side push relay on the roadmap?
2. Exec-plugin clusters (EKS/GKE/AKS with SSO) are most real clusters. Is "create a ServiceAccount token" acceptable for v1, or should cloud SSO be a v2 priority?
3. Should mutating actions be allowed on clusters that look like production, or should v1 be read-only for them unless explicitly enabled per cluster?

## Owner decisions (2026-10-28)

- **Alerts:** none in v1. On-call pagers (PagerDuty/Opsgenie) alert; Apex supports deep links that open a cluster/namespace/pod directly.
- **Auth:** ServiceAccount token (plus client cert later) is acceptable for v1, including for EKS/GKE/AKS. No exec-plugin or OIDC login in v1.
- **Prod safety:** contexts flagged as production (name matches prod/prd, or marked manually) are read-only on mobile until explicitly unlocked per session.
- **Resource scope:** desktop gets the full set (adds RBAC, StorageClasses, Endpoints/EndpointSlices, IngressClasses, PDBs, ResourceQuotas, LimitRanges, PriorityClasses, generic CRD browser). Mobile v1 covers the triage set only: pods, deployments, statefulsets, daemonsets, jobs, nodes, events, namespaces.
