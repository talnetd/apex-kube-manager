# macOS release signing and notarization

Tagged releases (`v*.*.*`) build the macOS `.dmg` files in `.github/workflows/release.yml`.
The build is signed with a Developer ID Application certificate and notarized by Apple,
so users can open it on first launch without the right-click > Open workaround.

Nothing in the repo hardcodes the identity or team. Everything comes from repository
secrets, so switching to another Apple team later is a secrets change, not a code change.

## How it works

| Step | Who does it |
|------|-------------|
| Import the `.p12` into a temporary keychain | Tauri bundler (reads `APPLE_CERTIFICATE` + `APPLE_CERTIFICATE_PASSWORD`) |
| Sign the `.app` with hardened runtime | Tauri bundler (`APPLE_SIGNING_IDENTITY`, `bundle.macOS.hardenedRuntime`) |
| Notarize and staple the `.app` | Tauri bundler via `notarytool` (`APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH`) |
| Build and sign the `.dmg` around the stapled `.app` | Tauri bundler |
| Write the `.p8` key to `$RUNNER_TEMP/asc` and delete it afterwards | Workflow steps |
| Verify (`codesign`, `spctl`, `stapler validate`, optional TeamIdentifier) | Workflow step "Verify signature and notarization" |

The bundler notarizes and staples the `.app`, not the `.dmg`. The `.dmg` is signed, and the
verify step mounts it and checks that the app inside carries the stapled ticket.

The minimum supported version is **macOS 11 (Big Sur)** (`bundle.macOS.minimumSystemVersion: "11.0"`).
The UI uses Tailwind 4, which needs Safari 16.4+ WebKit features (`color-mix`, `@property`, cascade layers).
WKWebView ships with Safari, and Safari 16.4+ only exists on macOS 11 and later.

Local builds (`npm run tauri build`) stay unsigned: `signingIdentity` is `null` in
`src-tauri/tauri.conf.json` and the `APPLE_*` variables are only set in CI.

## Secrets

| Secret | Required | Value |
|--------|----------|-------|
| `APPLE_CERTIFICATE` | Yes | Base64 of the exported Developer ID Application `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | Yes | Password you set when exporting the `.p12` |
| `APPLE_SIGNING_IDENTITY` | Yes | Full identity name, e.g. `Developer ID Application: Tin Aung Lin (ZYN774L23Z)` |
| `APPLE_API_ISSUER` | Yes | App Store Connect API Issuer ID (UUID) |
| `APPLE_API_KEY` | Yes | App Store Connect API Key ID (10 characters) |
| `APPLE_API_PRIVATE_KEY` | Yes | Contents of `AuthKey_<KEY_ID>.p8` |
| `APPLE_TEAM_ID` | No | Team ID to assert on the signed app (secret or repo variable). Check is skipped when unset |

The workflow fails early with a clear error if any required secret is missing.

## 1. Create the App Store Connect API key

1. Sign in to [App Store Connect](https://appstoreconnect.apple.com) as the Account Holder or an Admin.
2. Go to **Users and Access > Integrations > Team Keys** (request API access first if prompted).
3. Click **+**, name it (e.g. `apex-kube-manager-notarize`), and pick the **Developer** role.
4. Download `AuthKey_<KEY_ID>.p8`. **You can only download it once**, so keep it safe until it is in the secret.
5. Note the **Key ID** (shown in the key row) and the **Issuer ID** (shown above the keys table).

## 2. Export the certificate as .p12

1. Open **Keychain Access** > **login** keychain > **My Certificates**.
2. Find `Developer ID Application: Tin Aung Lin (ZYN774L23Z)` and expand it to confirm the private key is underneath.
3. Right-click the certificate > **Export...**, choose **Personal Information Exchange (.p12)**, save as `cert.p12`.
4. Set a strong export password. That becomes `APPLE_CERTIFICATE_PASSWORD`.

Check the identity name if unsure:

```bash
security find-identity -v -p codesigning
```

## 3. Set the secrets

Run from the folder holding `cert.p12` and the `.p8`:

```bash
base64 -i cert.p12 | gh secret set APPLE_CERTIFICATE -R talnetd/apex-kube-manager
gh secret set APPLE_CERTIFICATE_PASSWORD -R talnetd/apex-kube-manager   # prompts for the value
gh secret set APPLE_SIGNING_IDENTITY -R talnetd/apex-kube-manager -b "Developer ID Application: Tin Aung Lin (ZYN774L23Z)"
gh secret set APPLE_API_ISSUER -R talnetd/apex-kube-manager -b "<ISSUER_ID>"
gh secret set APPLE_API_KEY -R talnetd/apex-kube-manager -b "<KEY_ID>"
gh secret set APPLE_API_PRIVATE_KEY -R talnetd/apex-kube-manager < AuthKey_<KEY_ID>.p8
# Optional TeamIdentifier check:
gh variable set APPLE_TEAM_ID -R talnetd/apex-kube-manager -b "ZYN774L23Z"
rm cert.p12
```

Store the `.p8` somewhere safe (a password manager) or delete it. A lost key can be revoked and replaced.

Confirm:

```bash
gh secret list -R talnetd/apex-kube-manager
```

## 4. Test with a release candidate tag

Tags containing `-` (e.g. `v0.1.13-rc1`) are published as **prereleases** on every platform.
The in-app update checker uses `/releases/latest`, which ignores prereleases, so users are not prompted.

```bash
git tag v0.1.13-rc1 && git push origin v0.1.13-rc1
gh run watch -R talnetd/apex-kube-manager
```

Then download the `.dmg` from the prerelease and check it on a Mac:

```bash
spctl -a -vv -t open --context context:primary-signature "Apex.Kube.Manager_0.1.13-rc1_aarch64.dmg"
xcrun stapler validate "/Volumes/Apex Kube Manager/Apex Kube Manager.app"
```

Delete the prerelease and tag when done:

```bash
gh release delete v0.1.13-rc1 -R talnetd/apex-kube-manager --cleanup-tag --yes
```

Note: the version inside the app comes from `tauri.conf.json`, not the tag, so the rc build may show the previous version number.

## Switching to an organization team later

When moving to an organization (e.g. Relaytic) Apple Developer team:

1. In the org team, create a new **Developer ID Application** certificate (Account Holder or Admin with cert access), install it, and export it as `.p12` (step 2).
2. Create a new App Store Connect API key in the **org's** App Store Connect (step 1). Keys are per team.
3. Update the secrets:

```bash
base64 -i cert.p12 | gh secret set APPLE_CERTIFICATE -R talnetd/apex-kube-manager
gh secret set APPLE_CERTIFICATE_PASSWORD -R talnetd/apex-kube-manager
gh secret set APPLE_SIGNING_IDENTITY -R talnetd/apex-kube-manager -b "Developer ID Application: <Org Name> (<ORG_TEAM_ID>)"
gh secret set APPLE_API_ISSUER -R talnetd/apex-kube-manager -b "<ORG_ISSUER_ID>"
gh secret set APPLE_API_KEY -R talnetd/apex-kube-manager -b "<ORG_KEY_ID>"
gh secret set APPLE_API_PRIVATE_KEY -R talnetd/apex-kube-manager < AuthKey_<ORG_KEY_ID>.p8
gh variable set APPLE_TEAM_ID -R talnetd/apex-kube-manager -b "<ORG_TEAM_ID>"
rm cert.p12
```

4. Cut an rc tag (section 4) and confirm the verify step reports the new TeamIdentifier.
5. Revoke the old personal API key once the org build is confirmed.

Heads-up: changing the team changes the app's code signature. Users on the old build keep working,
but macOS may ask again for keychain or privacy permissions the app was granted before.
