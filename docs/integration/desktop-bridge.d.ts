// Kryton Desktop bridge — type declarations for the embedded Kryton web app.
//
// Vendor this file into the Kryton web client (e.g.
// `packages/client/src/types/desktop-bridge.d.ts`) and reference it from
// `tsconfig.json` so that `window.__kryton_desktop` is well-typed.
//
// See ./desktop-bridge.md for the contract.

export interface KrytonDesktopBridge {
  /** UUID of the connected account this webview belongs to. Stable across navigations within the same window. */
  readonly accountId: string;
  /** Kryton Desktop version (semver, e.g. "0.1.8"). */
  readonly version: string;
}

declare global {
  interface Window {
    /** Present only when the page is running inside Kryton Desktop. Frozen. */
    readonly __kryton_desktop?: KrytonDesktopBridge;
  }
}

export {};
