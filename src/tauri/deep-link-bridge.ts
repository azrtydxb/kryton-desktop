/**
 * deep-link-bridge.ts — handles `kryton://` deep-link URLs in the desktop app.
 *
 * Supported URLs:
 *   kryton://note/<path>?account=<id>
 *   kryton://daily?account=<id>
 *   kryton://search?q=<query>&account=<id>
 *   kryton://settings/<section>?account=<id>
 */

import { onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type ParsedDeepLink =
  | { kind: "note"; path: string; account: string }
  | { kind: "daily"; account: string }
  | { kind: "search"; q: string; account: string }
  | { kind: "settings"; section: string; account: string };

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

/**
 * Register the deep-link listener. Call once on app mount.
 *
 * @param onUrl  Callback invoked for each successfully-parsed incoming URL.
 */
export async function setupDeepLinks(
  onUrl: (parsed: ParsedDeepLink) => void,
): Promise<void> {
  await onOpenUrl(async (urls) => {
    for (const url of urls) {
      try {
        const parsed = parseDeepLink(url);
        if (!parsed) {
          console.warn("[deep-link] unrecognised URL, ignoring:", url);
          continue;
        }

        // Open (or focus) the account window for the target account.
        await invoke("open_account_window", { accountId: parsed.account });

        onUrl(parsed);
      } catch (e) {
        console.warn("[deep-link] failed to handle URL", url, e);
      }
    }
  });
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

function parseDeepLink(url: string): ParsedDeepLink | null {
  let u: URL;
  try {
    u = new URL(url);
  } catch {
    return null;
  }

  if (u.protocol !== "kryton:") return null;

  const account = u.searchParams.get("account") ?? "";

  switch (u.host) {
    case "note": {
      const path = decodeURIComponent(u.pathname.replace(/^\//, ""));
      return { kind: "note", path, account };
    }
    case "daily": {
      return { kind: "daily", account };
    }
    case "search": {
      const q = u.searchParams.get("q") ?? "";
      return { kind: "search", q, account };
    }
    case "settings": {
      const section = decodeURIComponent(u.pathname.replace(/^\//, ""));
      return { kind: "settings", section, account };
    }
    default:
      return null;
  }
}
