# Kryton Desktop Bridge

The Kryton web app (`kryton/packages/client`) runs inside a Tauri 2 webview when the user opens it through Kryton Desktop. This document specifies the **only** contract between the embedded web app and the desktop host. Anything not documented here is an internal of Kryton Desktop and may change without notice.

## Design principles

- The web app is the source of truth for the user-facing UI. Kryton Desktop is a thin shell around it.
- The web app must run unchanged in any browser. Anything desktop-specific is opt-in behind feature detection.
- The web app does **not** talk to Tauri directly. `window.__TAURI__` is not exposed inside the per-server webviews, and the `server-*` capability scope grants zero plugin permissions.
- All host-side capabilities the web app may need are surfaced through a single namespaced global: `window.__kryton_desktop`.

## Detecting the desktop host

```ts
if (typeof window !== "undefined" && (window as any).__kryton_desktop) {
  // running inside Kryton Desktop
}
```

`__kryton_desktop` is `undefined` in plain browsers, an object inside Kryton Desktop. The object is frozen — its shape cannot be tampered with at runtime.

## Surface (v1)

```ts
interface KrytonDesktopBridge {
  /** UUID of the connected account this webview belongs to. Stable across navigations within the same window. */
  readonly accountId: string;
  /** Kryton Desktop version (semver, e.g. "0.1.8"). */
  readonly version: string;
}
```

That's the entire current surface. New members are additive only.

## Notifications

Notifications are delivered **from the Kryton server to Kryton Desktop**, not from the web app to the host. The desktop subscribes to:

```
GET /api/notifications/stream
```

(authenticated by the same session cookie as the rest of the Kryton API; one long-lived SSE connection per connected account).

Each event is a single JSON object:

```json
{
  "id": "string",
  "kind": "share-invite|mention|deadline|custom",
  "title": "string",
  "body": "string",
  "createdAt": "RFC3339 timestamp"
}
```

The desktop emits an OS notification per event via the host's native notification system. The web app does not need to know whether the user is running inside the desktop — it renders in-app indicators normally, and the desktop adds the OS-level layer in parallel.

When `/api/notifications/stream` is absent (e.g. against an older Kryton server) the subscriber 404s and exits silently; no retries, no user-visible errors.

## Proposed additions (not yet implemented)

| Member | Purpose |
|---|---|
| `setBadge(count: number)` | Set the dock/taskbar icon's unread badge. |
| `openExternal(url: string)` | Open a URL in the user's default browser (instead of inside the webview). |
| `requestQuickCapture(seed?: string)` | Programmatically open the quick-capture window with optional pre-filled text. |
| `app.preferences` | Expose user-set keyboard shortcut for quick-capture, theme preference, etc. |

These will land as additive members on `window.__kryton_desktop`. The web app should feature-detect each one and fall back gracefully on older desktop versions.

## How to add to the Kryton web app

```ts
// In a hook, e.g. useDesktopHost.ts
type Bridge = Readonly<{ accountId: string; version: string }>;

export function useDesktopHost(): Bridge | null {
  if (typeof window === "undefined") return null;
  return ((window as unknown) as { __kryton_desktop?: Bridge }).__kryton_desktop ?? null;
}
```

A `.d.ts` declaration is published at [`./desktop-bridge.d.ts`](./desktop-bridge.d.ts) and can be vendored into `packages/client/src/types/`.

## Why a custom bridge instead of `window.__TAURI__`

- Exposing Tauri's own API to the embedded web app would couple Kryton's UI to a specific desktop runtime. Switching shells (e.g. Electron, a custom Rust webview, or running Kryton in a regular browser) would break the web app.
- Tauri's permission model is per-window: granting any plugin to the `server-*` window scope opens an attack surface from any HTTPS origin the user has added. The minimal `__kryton_desktop` namespace exposes only what we deliberately offer.
- Tauri's plugin webview-side helpers (e.g. `tauri-plugin-notification`) probe permission state on load, which produces spurious console errors when the embedded page hasn't requested anything.
