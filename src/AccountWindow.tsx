/**
 * AccountWindow — the main application window for an authenticated Kryton account.
 *
 * Responsibilities:
 *   1. Resolve account metadata (serverUrl) from the Tauri account store.
 *   2. Initialise the Kryton core via initDesktopCore.
 *   3. Wrap the UI layer in KrytonDataProvider using a CoreAdapter.
 *   4. Render AppShell from @azrtydxb/ui as the top-level shell.
 *   5. Touch the account on mount (updates last_logged_in_at).
 *   6. Listen for Tauri menu-action events and route them.
 */
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { AppShell, KrytonDataProvider } from "@azrtydxb/ui";
import { initDesktopCore } from "./core/desktop-init";
import { authStorage } from "./auth/auth-storage";
import { CoreAdapter } from "./core/CoreAdapter";
import { pickMarkdownFile } from "./tauri/file-dialogs";
import { setupDropHandler } from "./tauri/drop-handler";
import type { Kryton } from "@azrtydxb/core";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface Account {
  id: string;
  label: string;
  server_url: string;
  last_logged_in_at: number;
}

type LoadState =
  | { status: "loading" }
  | { status: "ready"; adapter: CoreAdapter; accountLabel: string; serverUrl: string }
  | { status: "error"; message: string };

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function AccountWindow({ accountId }: { accountId: string }) {
  const [state, setState] = useState<LoadState>({ status: "loading" });
  // Keep a stable ref so the menu-action handler can access the latest adapter
  // without re-subscribing every render.
  const adapterRef = useRef<CoreAdapter | null>(null);

  useEffect(() => {
    let cancelled = false;
    let coreInstance: Kryton | null = null;

    async function init() {
      try {
        // 1. Touch the account (fire-and-forget — don't block init on it).
        invoke("touch_account", { accountId }).catch((err: unknown) => {
          console.warn("[AccountWindow] touch_account failed:", err);
        });

        // 2. Fetch account metadata from the Tauri store.
        const accounts = await invoke<Account[]>("list_accounts");
        const account = accounts.find((a) => a.id === accountId);
        if (!account) {
          setState({ status: "error", message: `Account "${accountId}" not found.` });
          return;
        }

        // 3. Initialise Kryton core.
        const core = await initDesktopCore({
          accountId,
          serverUrl: account.server_url,
          authToken: () => authStorage.getToken(accountId),
        });

        if (cancelled) {
          await core.close();
          return;
        }
        coreInstance = core;

        // 4. Derive userId from available rows (may be empty on a fresh DB).
        let userId = "";
        const firstFolder = core.folders.list()[0];
        if (firstFolder?.userId) {
          userId = firstFolder.userId;
        } else {
          const firstTag = core.tags.list()[0];
          if (firstTag?.userId) userId = firstTag.userId;
        }

        // 5. Build the adapter.
        const adapter = new CoreAdapter(core, userId);
        adapterRef.current = adapter;

        setState({ status: "ready", adapter, accountLabel: account.label, serverUrl: account.server_url });
      } catch (err) {
        if (cancelled) return;
        const message =
          err instanceof Error ? err.message : String(err);
        setState({ status: "error", message });
      }
    }

    void init();

    return () => {
      cancelled = true;
      adapterRef.current = null;
      coreInstance?.close().catch((err: unknown) => {
        console.warn("[AccountWindow] core.close() failed:", err);
      });
    };
  }, [accountId]);

  // ---------------------------------------------------------------------------
  // Global hotkey — open-quick-switcher (DCC-A2)
  // ---------------------------------------------------------------------------

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<void>("open-quick-switcher", () => {
      // TODO: wire to command palette / quick-switcher UI state in a later phase.
      console.info("[AccountWindow] open-quick-switcher received");
    })
      .then((fn) => {
        unlisten = fn;
      })
      .catch((err: unknown) => {
        console.warn("[AccountWindow] failed to subscribe to open-quick-switcher:", err);
      });
    return () => {
      unlisten?.();
    };
  }, []);

  // ---------------------------------------------------------------------------
  // Drag-drop file import (DCC-A4)
  // ---------------------------------------------------------------------------

  useEffect(() => {
    if (state.status !== "ready") return;
    let cleanup: (() => void) | undefined;

    setupDropHandler({
      serverUrl: state.serverUrl,
      authToken: () => authStorage.getToken(accountId),
    })
      .then((unlisten) => {
        cleanup = unlisten;
      })
      .catch((err: unknown) => {
        console.warn("[AccountWindow] setupDropHandler failed:", err);
      });

    return () => {
      cleanup?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [state.status, accountId]);

  // ---------------------------------------------------------------------------
  // Menu-action listener (DC-E1 / DC-E2)
  // ---------------------------------------------------------------------------

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listen<string>("menu-action", (event) => {
      const action = event.payload;
      handleMenuAction(action, adapterRef.current).catch((err: unknown) => {
        console.warn("[AccountWindow] menu-action handler error:", err);
      });
    })
      .then((fn) => {
        unlisten = fn;
      })
      .catch((err: unknown) => {
        console.warn("[AccountWindow] failed to subscribe to menu-action:", err);
      });

    return () => {
      unlisten?.();
    };
  }, []);

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  if (state.status === "loading") {
    return (
      <div className="flex h-screen items-center justify-center bg-white dark:bg-surface-950 text-surface-600 dark:text-surface-400">
        Loading…
      </div>
    );
  }

  if (state.status === "error") {
    return (
      <div className="flex h-screen flex-col items-center justify-center gap-3 bg-white dark:bg-surface-950 p-8">
        <p className="text-red-600 font-semibold">Failed to initialise account</p>
        <p className="text-sm text-surface-500">{state.message}</p>
      </div>
    );
  }

  return (
    <KrytonDataProvider adapter={state.adapter}>
      <AppShell
        header={
          <div className="flex items-center gap-3 px-4 py-2 text-sm font-medium text-surface-700 dark:text-surface-300">
            <span className="text-primary-600 font-semibold">Kryton</span>
            <span className="text-surface-400">/</span>
            <span>{state.accountLabel}</span>
          </div>
        }
      >
        {/* Main content area — feature panels will be mounted here in later phases. */}
        <div className="flex h-full items-center justify-center text-surface-400 text-sm select-none">
          Ready — account window initialised.
        </div>
      </AppShell>
    </KrytonDataProvider>
  );
}

// ---------------------------------------------------------------------------
// Menu action router
// ---------------------------------------------------------------------------

/**
 * Route a `menu-action` event payload to the appropriate handler.
 * The adapter may be null if the window hasn't finished initialising yet.
 */
async function handleMenuAction(
  action: string,
  adapter: CoreAdapter | null,
): Promise<void> {
  console.info("[AccountWindow] menu-action:", action);

  switch (action) {
    case "new-note": {
      if (!adapter) {
        console.warn("[AccountWindow] new-note: adapter not ready");
        return;
      }
      // Create a new untitled note via the public adapter API.
      await adapter.notes.create({
        path: `/Untitled-${Date.now()}.md`,
        title: "Untitled",
        content: "",
      });
      break;
    }

    case "open": {
      // Open a Markdown file from disk and import it as a new note.
      const file = await pickMarkdownFile();
      if (!file) return; // user cancelled

      if (!adapter) {
        console.warn("[AccountWindow] open: adapter not ready — file picked but cannot import");
        return;
      }
      // Derive title from the filename (strip path and extension).
      const filename = file.path.split("/").pop() ?? "imported.md";
      const title = filename.replace(/\.(md|markdown)$/i, "");
      const path = `/${filename}`;
      await adapter.notes.create({ path, title, content: file.content });
      break;
    }

    case "find":
      // Handled by the editor component directly (browser find or custom).
      break;

    case "toggle-sidebar":
    case "toggle-graph":
    case "toggle-edit-preview":
    case "show-daily":
      // View actions — will be wired to UI state in later phases.
      console.info("[AccountWindow] view action (not yet wired):", action);
      break;

    case "switch-account":
      // Open the launcher so the user can pick another account.
      invoke("open_launcher_window").catch((err: unknown) => {
        console.warn("[AccountWindow] open_launcher_window failed:", err);
      });
      break;

    case "docs":
      // Open documentation in the default browser.
      window.open("https://docs.kryton.app", "_blank");
      break;

    case "show-logs":
      // Logs are accessible via the Tauri developer console in dev mode.
      console.info("[AccountWindow] show-logs triggered");
      break;

    case "report-issue":
      window.open("https://github.com/azrtydxb/kryton-desktop/issues/new", "_blank");
      break;

    default:
      // Predefined menu items (cut, copy, paste, etc.) are handled natively.
      break;
  }
}

