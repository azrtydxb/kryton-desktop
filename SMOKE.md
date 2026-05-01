# Smoke Testing Checklist

Manual smoke testing for Kryton desktop app. Run these checks after a successful build to verify core functionality.

## Pre-test Setup

- [ ] Build the app: `npm run build`
- [ ] Start the dev server: `npm run vite:dev` (in one terminal)
- [ ] Run Tauri dev: `npm run dev` (in another terminal)

## Core UI

- [ ] App launches without errors
- [ ] Main window appears and is responsive
- [ ] Tailwind CSS styling is applied correctly
- [ ] Responsive layout works at different window sizes

## Authentication & First Run

- [ ] First-run flow displays correctly
- [ ] LoginScreen renders without errors
- [ ] ServerSetup flow is accessible
- [ ] Auth token storage works (verify in app data)

## File Dialogs

- [ ] File picker dialog opens when triggered
- [ ] File selection works and returns correct path
- [ ] Dialog cancel works without errors

## Native Menu Bar

- [ ] Native menu bar appears (platform-specific)
- [ ] Menu items are clickable
- [ ] Menu actions execute correctly

## Tauri Integration

- [ ] Window state persists (maximize/minimize/size)
- [ ] App can be quit gracefully
- [ ] No console errors in dev tools
- [ ] IPC commands execute without errors

## Core Functionality

- [ ] Database adapter initializes correctly
- [ ] Data persistence works
- [ ] No TypeScript compilation errors
- [ ] All unit tests pass: `npm run test`

## Build & Artifacts

- [ ] Debug build completes successfully
- [ ] Build artifacts are created in expected locations
- [ ] No warnings during build process

## Dev Linking

- [ ] `npm run dev:link` completes without errors
- [ ] Dev link verification passes: `npm run dev:verify`
- [ ] `npm run dev:unlink` works correctly

## Tier 3 Native Integrations (Phase B)

- [ ] Native notification appears when `notify(title, body)` is called from the frontend (requires permission grant on first use)
- [ ] Spotlight: after calling `spotlight_index`, note title appears in macOS Spotlight search (⌘Space)
- [ ] Spotlight: `spotlight_remove` removes a previously-indexed note from results
- [ ] Windows Search: `.url` shortcut files appear in `%LocalAppData%\Microsoft\Windows\Start Menu\Programs\Kryton\<account_id>\` after `win_search_upsert_note`
- [ ] Windows taskbar jumplist: right-clicking the Kryton taskbar icon shows "Recent Notes" and tasks (New Note, Open Quick Switcher)

## Release Flow (Operator Checklist)

This section documents what the operator must do to cut a production release.
**Do not push a `v*` tag until all secrets are configured** — the release
workflow will fail without them.

### 1 — Configure GitHub Repository Secrets

Go to **Settings → Secrets and variables → Actions** and add:

| Secret | Where to get it |
|---|---|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` Developer ID Application cert — see README |
| `APPLE_CERT_PASSWORD` | Password for the `.p12` |
| `APPLE_NOTARY_USER` | Apple ID email (must have accepted the latest Apple Developer agreement) |
| `APPLE_NOTARY_PASSWORD` | App-specific password generated at appleid.apple.com |
| `APPLE_TEAM_ID` | 10-character Team ID from developer.apple.com |
| `WINDOWS_CERT_BASE64` | Base64-encoded `.pfx` EV or OV code-signing cert |
| `WINDOWS_CERT_PASSWORD` | Password for the `.pfx` |
| `TAURI_PRIVATE_KEY` | Minisign private key — generate with `npx tauri signer generate` |

The matching public key is already committed to `src-tauri/tauri.conf.json`
under `plugins.updater.pubkey`. If the private key is lost, the updater chain
is broken and users must re-install manually.

### 2 — Bump the Version

Update the version in both files to match (e.g. `1.0.0`):

```sh
# package.json → "version": "1.0.0"
# src-tauri/tauri.conf.json → "version": "1.0.0"
```

Commit and push to `master`.

### 3 — Tag the Release

```sh
git tag -s v1.0.0 -m "release v1.0.0"
git push origin v1.0.0
```

The `release.yml` workflow triggers automatically on `v*` tags.

### 4 — Monitor the Workflow

In the GitHub Actions tab, watch three jobs:

- **build-mac** — signs and notarizes the `.app` bundle + `.dmg`.
- **build-windows** — signs the `.msi` and NSIS installer.
- **release** — downloads both artifact sets, runs
  `scripts/generate-updater-manifest.js` to produce `latest.json`, then
  creates the GitHub Release with all files attached.

If any job fails:
- For signing failures, check the certificate secrets (expiry, format).
- For notarization failures, check `APPLE_NOTARY_USER` / `APPLE_NOTARY_PASSWORD`
  and the Apple Developer account status.
- The workflow can be re-run from the Actions UI once secrets are corrected.

### 5 — Verify the Release

After the workflow completes:

1. Open the [Releases page](https://github.com/azrtydxb/kryton-desktop/releases).
2. Confirm that the release lists `.dmg`, `.msi`/`.exe`, and `latest.json`.
3. Download the `.dmg` on macOS and verify it opens without Gatekeeper warnings.
4. Download the `.msi`/`.exe` on Windows and verify it installs cleanly.
5. On an already-installed older build, the in-app updater should detect the
   new version within a few seconds of launch and prompt to update.

### Auto-Update Architecture

- `latest.json` is served directly from GitHub Releases.
- The Tauri updater on each client checks the endpoint configured in
  `tauri.conf.json → plugins.updater.endpoints` on every launcher window open.
- If a new version is available, the app downloads and installs it silently,
  then relaunches.
- All update bundles are verified against the minisign public key committed to
  `tauri.conf.json` before installation.

## Known Limitations

### DCC-B4: Touch Bar — deferred to v1.1

The `objc2-app-kit` crate (v0.3.2) exposes `NSTouchBar` bindings, but implementing a functional Touch Bar requires defining a custom Objective-C delegate class via `define_class!` and managing `MainThreadOnly` objects across Tauri's async runtime boundary. The integration risk outweighs the benefit for v1 (Touch Bar hardware was discontinued by Apple in 2021 with the 14/16-inch MacBook Pro lineup). Deferred per spec-sanctioned skip policy.

**Re-enable in v1.1:** Implement `touchbar.rs` with a `define_class!`-based `NSTouchBarDelegate`, set it on the `NSWindowController` for each account window, and expose a Tauri event listener to update the current note title and sync status in real time.
