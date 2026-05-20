# Kryton Desktop

Cross-platform desktop client for [Kryton](https://github.com/azrtydxb/kryton).
A Tauri-based native shell around the Kryton web UI with multi-account support,
system tray, global shortcuts, deep links, and native notifications.

## Development

```sh
pnpm install
pnpm tauri dev
```

You'll need a running Kryton server. The easiest way:

```sh
cd ../kryton
docker compose up -d
```

Then add `http://localhost:8080` as a server in the Kryton Desktop login window.

## Building

```sh
pnpm tauri build
```

## Releases

Tag with `vX.Y.Z-pre.N` for pre-releases or `vX.Y.Z` for stable.
GitHub Actions builds for macOS (Intel + ARM), Windows, and Linux,
signs the artifacts, and uploads them to a GitHub release.

Before cutting the first release tag, generate a Tauri updater key and store it:

```sh
pnpm tauri signer generate -w ~/.tauri/kryton-updater.key
```

Then:
- Paste the printed **public key** into `src-tauri/tauri.conf.json` under
  `plugins.updater.pubkey`, replacing the `REPLACE_WITH_TAURI_SIGNER_PUBLIC_KEY`
  placeholder.
- Add the **private key file contents** to GitHub repo secrets as
  `TAURI_SIGNING_PRIVATE_KEY`, and its password (if set) as
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.

## Required CI secrets

- `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — updater signing
- `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`,
  `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` — macOS Developer ID signing + notarization
- `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD` — optional Authenticode
  (without these, the MSI is produced unsigned and Windows SmartScreen will warn)

## License

Apache-2.0
