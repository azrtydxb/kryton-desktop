# Changelog

All notable changes to Kryton Desktop are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial implementation of the Kryton Desktop client (Tauri 2 + Vite + TypeScript).
- Multi-account support: one `WebviewWindow` per connected server, isolated per-account `data_directory`, switch via native menu (`Cmd+1..9`), tray menu, or settings window.
- Auto-login on first launch: credentials stored via OS keychain (release) or AES-256-GCM-encrypted file (dev), injected into the per-server webview to sign into Kryton without showing its login form.
- In-page auth-failure banner inside the server webview when saved credentials are invalid.
- System tray with server list, quick-capture, settings, and quit.
- Global shortcut (`Cmd+Shift+N` by default) opens a quick-capture window that appends to the active server's daily note (`POST /api/daily` then `PUT /api/notes/Daily/<YYYY-MM-DD>.md`).
- Native menu bar (Kryton / File / Edit / View / Servers) with platform-standard accelerators.
- `kryton://` deep-link scheme with prompt-to-add fallback for unknown hosts.
- Single-instance guard: a second launch focuses the running app and forwards deep-link arguments.
- Notification shim: web UI calls `window.__kryton_desktop.notify(title, body)` to fire OS notifications. (Long-term plan: see [docs/superpowers/specs/2026-05-20-desktop-app-design.md](docs/superpowers/specs/2026-05-20-desktop-app-design.md).)
- Auto-update via Tauri's updater pointed at GitHub Releases.
- Settings window: account management (switch / remove / add) plus a system/dark/light theme picker for shell windows.
- 3-platform CI matrix (macOS, Windows, Linux) for builds and tests on every push.
- Tag-triggered release workflow that signs and uploads `.dmg` / `.msi` / `.AppImage` plus `latest.json` to a GitHub release.

[Unreleased]: https://github.com/azrtydxb/kryton-desktop/commits/main
