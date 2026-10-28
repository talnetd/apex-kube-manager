# Shared mobile/desktop spec

Platform-neutral contracts shared by the iOS, Android and (later) desktop code. See [`docs/mobile/architecture.md`](../docs/mobile/architecture.md), sections 3 and 8.

Planned contents:

| Path | What | Lands in |
|---|---|---|
| `triage/` | `rules.md`, verdict schema, golden fixtures run by Rust (live), Swift and Kotlin tests | redesign PR1b (mobile PR2 adds the Swift/Kotlin runners) |
| `design-tokens/tokens.json` | Colors, type, spacing, status colors | redesign track |
| `rbac/apex-mobile-triage.yaml` | Recommended least-privilege role (no secrets) | PR3 |

Any change under `spec/**` runs the desktop, iOS and Android workflows.
