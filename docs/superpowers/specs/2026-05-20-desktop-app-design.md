# Kryton Desktop App — Design

**Date:** 2026-05-20
**Status:** Approved (design phase)
**Repo:** `/Users/pascal/Development/Kryton/kryton-desktop`

## 1. Scope & Goals

A cross-platform desktop client that wraps the existing Kryton web UI in a Tauri shell with multi-account support and native OS integrations. The web UI is loaded unmodified inside a webview; the desktop app provides only the native shell and account-management chrome around it.

### Goals

- Native feel: system tray, global shortcuts, native menu bar, native notifications, deep links, file drag-and-drop, auto-update.
- Multi-account: a single app instance connects to multiple Kryton servers; the user switches between them via menu/tray/shortcut. Each server keeps its own webview, cookies, and storage.
- Credentials stored in the OS keychain; silent re-auth on launch.
- v1 ships on macOS, Windows, and Linux.

### Non-goals (v1)

- No offline mode. No read cache, no offline editing, no sync.
- No in-window chrome (no native sidebar, no tab strip). Switching is via menu bar, tray, and keyboard shortcut only.
- No mobile or web build (separate repos).
- No new server-side features. Authentication uses the existing username/password endpoint; a future desktop-token endpoint can be a drop-in swap.

## 2. Architecture

### Stack

- **Shell:** Tauri 2.x (Rust core + system webview).
- **Shell frontend:** TypeScript + Vite, used only for the small set of native shell pages (login, add-server, settings, quick-capture). It does not render any note-taking UI.
- **Embedded web app:** the live Kryton web UI, served by each connected Kryton server, loaded into a webview per server.

### Process model

```
┌─ WebviewWindow for server A ───┐  ┌─ WebviewWindow for server B ───┐
│ [Native menu bar]              │  │ [Native menu bar]              │
│ → https://server-A.example.com │  │ → https://server-B.example.com │
└────────────────────────────────┘  └────────────────────────────────┘
   one WebviewWindow per connected server (one visible, others hidden)
   + tray icon (separate)
   + quick-capture window (separate, opened on demand)
   + shell windows: login/add-server, settings (opened on demand)
```

- One `WebviewWindow` per connected server, each with its own data directory so cookies/storage do not cross between servers. Tauri 2's stable single-webview-per-window API is used; multi-webview-per-window remains behind an `unstable` flag and is intentionally avoided.
- Switching servers raises/focuses the target window and hides the previously-active one. Hidden windows are kept alive so the embedded web UI's scroll position and open editor tabs are preserved.
- The Rust core owns: account list, keychain access, tray, global shortcuts, deep-link handler, updater, native notifications dispatch, IPC bridge to the shell frontend.
- The shell frontend owns: login/add-server form, settings window, quick-capture window. These never render inside the server webview.
- A small per-account notifier task (in the Rust core) subscribes to the server's notifications stream and emits OS notifications. If Kryton does not yet expose a notifications stream, v1 falls back to having the embedded web UI bridge in-app notifications to the Rust core via an injected IPC shim. This is the only injection into the web UI; it is non-visual.

### Component breakdown

| Component | Owner | Responsibility |
|---|---|---|
| `accounts` | Rust | CRUD on the account list (server URL, label, last-active). Persists to `accounts.json`. Secrets go to keychain, never to JSON. |
| `auth` | Rust | Read/write keychain via `tauri-plugin-stronghold` (or `keyring` fallback). Silent login on launch via the server's password endpoint. |
| `window-mgr` | Rust | Spawn/show/hide one `WebviewWindow` per account, route URL changes, manage per-account data directories. |
| `tray` | Rust | Tray icon and menu (new note, switch server, quit). |
| `shortcuts` | Rust | Register and dispatch global shortcuts. |
| `deep-link` | Rust | Parse `kryton://` URLs, prompt-to-add if host unknown, route to the matching webview. |
| `notifier` | Rust | Subscribe to per-server events, emit OS notifications. |
| `updater` | Rust | Tauri updater against GitHub Releases. |
| `shell-ui` | TS/Vite | Login, settings, quick-capture windows. |

Each component has one clear purpose and a typed IPC interface to the others. Files stay small enough that any one can be held in context while editing.

## 3. Data & Storage

### Account record

Stored at `~/.config/kryton-desktop/accounts.json` (platform-appropriate path; the directory is created if missing):

```json
{
  "accounts": [
    {
      "id": "uuid",
      "label": "Home",
      "server_url": "https://kryton.home.example.com",
      "username": "pascal",
      "last_active": "2026-05-20T12:00:00Z"
    }
  ],
  "default_active": "uuid",
  "settings": {
    "shortcut_quick_capture": "CmdOrCtrl+Shift+N",
    "launch_at_login": false,
    "auto_update_channel": "stable"
  }
}
```

### Secrets

Passwords live only in the OS keychain via `tauri-plugin-stronghold`, with `keyring` crate as fallback on platforms where stronghold is unavailable. The keychain entry is keyed by account `id`. The JSON file never holds secrets.

### Webview storage

Each per-server `WebviewWindow` is configured with its own `data_directory` (Tauri `WebviewWindowBuilder::data_directory`) under `<app_data_dir>/webview-data/<account-id>/`, so cookies, localStorage, and IndexedDB do not cross between servers.

### Update artifacts

- `latest.json` plus signed `.dmg` / `.msi` / `.AppImage` on GitHub Releases (`github.com/azrtydxb/kryton-desktop`).
- Tauri verifies an Ed25519 signature against a public key compiled into the binary. The private key lives in repo CI secrets only.

## 4. Key Flows

### First launch

1. Shell window opens with "Add your first server" (URL + username + password).
2. POST to `<server>/api/auth/login`. On success, store password in keychain, write the account, swap to a webview pointed at `<server>/`.

### Subsequent launches

1. Read `accounts.json`. For each account, retrieve the password from keychain and re-auth in the background.
2. Show the webview for `default_active` (or the most recent if unset).

### Switch server

- Triggered by `View → Servers → <name>`, tray menu, or `Cmd+1..9` / `Ctrl+1..9`.
- Rust core calls `window-mgr.show(account_id)`: shows and focuses the target server's `WebviewWindow`, hides the previously-active one. Hidden windows stay alive so scroll position and open editor tabs are preserved.

### Quick-capture

- Global shortcut (default `Cmd/Ctrl+Shift+N`) opens a small always-on-top window with a body field and Save button.
- On save, POST to the **active** server's daily-note append endpoint, then close. No server picker.
- If no server is currently active (e.g. all are unreachable), the capture window shows an error and the note is not lost: it remains in the field until the user retries or copies it out.

### Deep link

- `kryton://<host>[:port]/<path>` is registered as a URL scheme on all three platforms.
- If `<host>` matches a connected account, focus that webview and navigate to `<path>`.
- Otherwise, show a modal: "Add `<host>` to your accounts? [Add] [Cancel]". If Add, run the add-server flow then navigate.

### File drag-and-drop

- Implemented inside the embedded web UI's existing attachments flow.
- The Tauri shell does not intercept drag events; it simply does not block them at the window level.

### Auto-update

- On launch and once daily thereafter, check the GitHub Releases `latest.json`.
- If a newer version is available, prompt the user. On accept, download, verify signature, and restart.

## 5. Error Handling & Edge Cases

- **Server unreachable on launch** — the webview shows the OS-level "failed to load" page; we overlay a small native banner: "Can't reach `<server>`. [Retry] [Edit account]". Other connected servers still work; the menu lets the user switch away.
- **Auth fails on silent re-login** — the webview shows the server's own login page. If we detect a 401 from the silent re-auth path, we prompt: "Password for `<server>` changed. [Update] [Remove account]".
- **Keychain unavailable or locked** — fall back to an in-memory password for the session; on next launch the user re-enters. Logged, never silently dropped.
- **Malformed deep link** — toast: "Invalid Kryton link." No crash.
- **Updater signature mismatch** — abort the update, log it, surface to the user. Never auto-restart on a bad payload.
- **Two app instances** — single-instance guard via `tauri-plugin-single-instance`. A second launch focuses the existing window and forwards any deep-link argv.
- **OS notification permission denied** — degrade silently; surface a one-line warning in settings.
- **Account removal** — close the per-server window, wipe the account's `data_directory`, delete the keychain entry, update `accounts.json` atomically.

## 6. Testing

- **Rust unit tests** for `accounts`, `auth`, `deep-link` parsing, and `window-mgr` state transitions (show/hide/close, focus tracking).
- **Integration tests** (Tauri's test harness) for first-launch, switch-server, deep-link routing, and single-instance behavior. Run against a local `docker-compose` Kryton from the `kryton` repo.
- **Manual smoke matrix** before each release on macOS, Windows, and Linux: add server, switch, quick-capture, global shortcut, deep link, tray, drag-drop, auto-update from the prior version.
- **No E2E browser tests against the embedded web app** — that is the web project's responsibility.

## 7. Build, Release, Repo Layout

### Repo layout

```
kryton-desktop/
├── src-tauri/                # Rust core
│   ├── src/
│   │   ├── accounts.rs
│   │   ├── auth.rs
│   │   ├── window_mgr.rs
│   │   ├── tray.rs
│   │   ├── shortcuts.rs
│   │   ├── deep_link.rs
│   │   ├── notifier.rs
│   │   ├── updater.rs
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                      # TS shell UI (Vite)
│   ├── login/
│   ├── settings/
│   ├── quick-capture/
│   └── main.ts
├── .github/workflows/
│   ├── ci.yml                # build + test on PRs
│   └── release.yml           # tag v* → build for 3 OSes, sign, publish to Releases
├── docs/
│   └── superpowers/specs/    # this document
├── package.json
└── README.md
```

### Release flow

- Versions follow the project convention: iterate with `vX.Y.Z-pre.N`; clean `vX.Y.Z` tags only with explicit user instruction.
- On a `v*` tag, GH Actions runs three matrix jobs (`macos-latest`, `windows-latest`, `ubuntu-latest`), builds, signs, and uploads `.dmg` / `.msi` / `.AppImage` plus `latest.json` to the release.

### Signing secrets (repo CI)

- macOS: Apple Developer ID certificate + notarization API key.
- Windows: Authenticode certificate (or accept SmartScreen warnings until one is available).
- Linux: no signing required; AppImage and `.deb` produced.
- Tauri updater: Ed25519 signing key.

## 8. Notification Architecture

**Decided 2026-05-20** — long-term, the desktop subscribes to a server-side notifications stream and emits OS notifications from the Rust core. The injected `window.__kryton_desktop.notify(title, body)` shim is kept as a transitional path for ad-hoc UI events that don't have a server-side equivalent.

### Server-side contract (proposed; not yet implemented in `kryton`)

`GET /api/notifications/stream` — Server-Sent Events. Authenticated by the same session cookie as the rest of the API. Each event is a single JSON object:

```json
{
  "id": "string",
  "kind": "share-invite|mention|deadline|custom",
  "title": "string",
  "body": "string",
  "createdAt": "RFC3339 timestamp"
}
```

The desktop subscribes once per connected account after `silent_relogin` succeeds. On `200 OK` it parses events and calls `notifier::notify(...)`. On `404` (endpoint not yet shipped on the user's server) it logs once and exits silently — no retries, no errors surfaced to the user.

### Why a stream rather than polling

Polling adds 1 request per account per poll interval; many users will run with 1–3 accounts forever. SSE is "1 long-lived connection per account" and lets the server push at most one event per real notification. better-auth's session cookie already authenticates the stream; no new auth code required.

### Why we still keep the IPC shim

Some notifications are purely client-side (e.g. "your draft was saved offline"). The shim lets the web UI fire those without a server round-trip. Long-term we may remove it; for v1 both paths coexist.

## 9. Open Questions Deferred to Implementation

- The notifications-stream endpoint described in §8 must be added to the Kryton server before the SSE subscriber does anything useful. The desktop ships the subscriber as a stub today.
- Exact path of the daily-note append endpoint used by quick-capture: confirmed against the server's current API at implementation time.
- Whether to ship a `.deb` in addition to `.AppImage` for Linux at v1, or defer `.deb` to v1.x.
