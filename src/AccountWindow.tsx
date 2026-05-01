/**
 * AccountWindow — the main application window for an authenticated Kryton account.
 *
 * Responsibilities:
 *   1. Resolve account metadata (serverUrl) from the Tauri account store.
 *   2. Initialise the Kryton core via initDesktopCore.
 *   3. Wrap the UI layer in KrytonDataProvider using a CoreAdapter.
 *   4. Render AppShell from @azrtydxb/ui as the top-level shell.
 *   5. Touch the account on mount (updates last_logged_in_at).
 *   6. (Placeholder) Listen for Tauri menu-action events.
 */
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppShell, KrytonDataProvider } from "@azrtydxb/ui";
import { initDesktopCore } from "./core/desktop-init";
import { authStorage } from "./auth/auth-storage";
import { CoreAdapter } from "./core/CoreAdapter";
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
  | { status: "ready"; adapter: CoreAdapter; accountLabel: string }
  | { status: "error"; message: string };

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function AccountWindow({ accountId }: { accountId: string }) {
  const [state, setState] = useState<LoadState>({ status: "loading" });

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

        setState({ status: "ready", adapter, accountLabel: account.label });
      } catch (err) {
        if (cancelled) return;
        const message =
          err instanceof Error ? err.message : String(err);
        setState({ status: "error", message });
      }
    }

    void init();

    // TODO (DC-D2+): wire Tauri menu-action events via listen("menu-action", handler).

    return () => {
      cancelled = true;
      coreInstance?.close().catch((err: unknown) => {
        console.warn("[AccountWindow] core.close() failed:", err);
      });
    };
  }, [accountId]);

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
