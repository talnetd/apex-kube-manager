# Dev builds

Every push to `master` (except changes only under `apps/`, `docs/`, `spec/` or `*.md`) runs
`.github/workflows/dev-build.yml`, which builds an Apple Silicon `.dmg` and uploads it as a
workflow artifact. No release or tag is created. Artifacts are kept for 14 days.

- **Artifact name**: `apex-dev-<short-sha>-aarch64` (`-x86_64` for Intel)
- **App version**: `<tauri.conf.json version>-dev.<run number>`, set in the runner only
- **Signing**: signed and notarized when the `APPLE_*` secrets from [release-signing.md](release-signing.md) are set, otherwise ad-hoc signed (unsigned). The job summary says which

## Download the latest one

```bash
gh run list -R talnetd/apex-kube-manager --workflow dev-build.yml -L 3
gh run download <run-id> -R talnetd/apex-kube-manager --name apex-dev-<sha>-aarch64
```

For an unsigned build, clear quarantine once after dragging the app to `/Applications`:

```bash
xattr -dr com.apple.quarantine "/Applications/Apex Kube Manager.app"
```

## Build any branch on demand

```bash
gh workflow run dev-build.yml -R talnetd/apex-kube-manager --ref <branch>
gh workflow run dev-build.yml -R talnetd/apex-kube-manager -f ref=<branch-or-sha> -f x86_64=true
gh run watch -R talnetd/apex-kube-manager
```

A PR that edits `dev-build.yml` also builds once, so workflow changes are tested before merge.

## Update checker

The in-app update checker compares against `CARGO_PKG_VERSION` (from `src-tauri/Cargo.toml`),
which dev builds leave untouched. So a dev build behaves like the release it is based on:
it prompts only when a real, newer release is published.
