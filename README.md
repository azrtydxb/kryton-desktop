# Kryton Desktop

Kryton desktop application built with Tauri v2 + React.

## Development

```sh
npm install
npm run dev        # tauri dev (hot-reload)
npm run typecheck  # TypeScript check
npm run test       # vitest
```

## CI

The CI pipeline runs on both `macos-latest` and `windows-latest` via a GitHub Actions matrix. Each target runs:

1. `npm ci`
2. `npm run vite:build`
3. `npm run typecheck`
4. `npm run test`
5. `npm run tauri build -- --debug` (unsigned debug build)

The `dev:verify` step runs on macOS only (it checks dev-link symlinks that are macOS-specific).

## Code Signing & Notarization — Operator Setup

Signed release builds require the following GitHub repository secrets. These must be set by the operator (never commit any of these values).

### Apple (macOS)

| Secret | Description |
|---|---|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` Developer ID Application certificate |
| `APPLE_CERT_PASSWORD` | Password for the `.p12` certificate |
| `APPLE_NOTARY_USER` | Apple ID email used for notarization |
| `APPLE_NOTARY_PASSWORD` | App-specific password for the Apple ID |
| `APPLE_TEAM_ID` | Apple Developer Team ID (10-character string) |

To export your certificate as base64:

```sh
base64 -i DeveloperIDApplication.p12 | pbcopy
```

### Windows

| Secret | Description |
|---|---|
| `WINDOWS_CERT_BASE64` | Base64-encoded `.pfx` code-signing certificate |
| `WINDOWS_CERT_PASSWORD` | Password for the `.pfx` certificate |

To export your certificate as base64:

```sh
certutil -encode cert.pfx cert_b64.txt
# or on macOS/Linux:
base64 -i cert.pfx
```

### Tauri Updater

| Secret | Description |
|---|---|
| `TAURI_PRIVATE_KEY` | Minisign private key for signing update bundles (see below) |

The updater public key is already committed to `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`. The matching private key must be stored as the `TAURI_PRIVATE_KEY` secret and used at release build time via the `TAURI_SIGNING_PRIVATE_KEY` environment variable.

**DO NOT commit the private key.** If the private key is lost, existing installs will no longer be able to verify updates — a new keypair must be generated and distributed via a forced re-install.

To regenerate the keypair:

```sh
npx tauri signer generate -w /tmp/tauri_key --password ""
# Commit the contents of /tmp/tauri_key.pub into tauri.conf.json plugins.updater.pubkey
# Add the contents of /tmp/tauri_key as the TAURI_PRIVATE_KEY secret
# Delete both temp files
```
