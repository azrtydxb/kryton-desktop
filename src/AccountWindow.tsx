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
 *   7. (DCC-C1) Wrap the tree in PluginRoot; provide PluginManagerScreen.
 *   8. (DCC-C2) Wire AgentsPanel to CoreAdapter.agents.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  AppShell,
  KrytonDataProvider,
  PluginRoot,
  PluginManagerScreen,
  AgentsPanel,
} from "@azrtydxb/ui";
import type {
  ActivePluginInfo,
  PluginManagerAPI,
  AgentInfo,
  NewTokenResult,
} from "@azrtydxb/ui";
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

// ---------------------------------------------------------------------------
// Plugin screen helper — builds a PluginManagerAPI backed by the server
// ---------------------------------------------------------------------------

function buildPluginManagerAPI(serverUrl: string, authTokenFn: () => Promise<string | null>): PluginManagerAPI {
  const headers = async (): Promise<Record<string, string>> => {
    const tok = await authTokenFn();
    return tok ? { Authorization: `Bearer ${tok}` } : {};
  };

  const apiFetch = async (path: string, opts?: RequestInit): Promise<Response> => {
    const h = await headers();
    return fetch(`${serverUrl}${path}`, { ...opts, headers: { ...h, ...(opts?.headers as Record<string, string> | undefined) } });
  };

  return {
    getRegistry: async () => {
      const res = await apiFetch("/api/plugins/registry");
      if (!res.ok) return [];
      const body = await res.json() as { plugins: unknown[] };
      return (body.plugins ?? []) as ReturnType<PluginManagerAPI["getRegistry"]> extends Promise<infer T> ? T : never;
    },
    getAllPlugins: async () => {
      const res = await apiFetch("/api/plugins/installed");
      if (!res.ok) return [];
      const body = await res.json() as { plugins: unknown[] };
      return (body.plugins ?? []) as ReturnType<PluginManagerAPI["getAllPlugins"]> extends Promise<infer T> ? T : never;
    },
    checkPluginUpdates: async () => {
      const res = await apiFetch("/api/plugins/updates");
      if (!res.ok) return [];
      const body = await res.json() as { updates: unknown[] };
      return (body.updates ?? []) as ReturnType<PluginManagerAPI["checkPluginUpdates"]> extends Promise<infer T> ? T : never;
    },
    getSettings: async () => {
      const res = await apiFetch("/api/plugins/settings");
      if (!res.ok) return {};
      return res.json() as Promise<Record<string, string>>;
    },
    updateSetting: async (key: string, value: string) => {
      await apiFetch("/api/plugins/settings", {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ key, value }),
      });
    },
    installPlugin: async (id: string) => {
      await apiFetch(`/api/plugins/${id}/install`, { method: "POST" });
    },
    uninstallPlugin: async (id: string) => {
      await apiFetch(`/api/plugins/${id}`, { method: "DELETE" });
    },
    enablePlugin: async (id: string) => {
      await apiFetch(`/api/plugins/${id}/enable`, { method: "POST" });
    },
    disablePlugin: async (id: string) => {
      await apiFetch(`/api/plugins/${id}/disable`, { method: "POST" });
    },
    reloadPlugin: async (id: string) => {
      await apiFetch(`/api/plugins/${id}/reload`, { method: "POST" });
    },
    updatePlugin: async (id: string) => {
      await apiFetch(`/api/plugins/${id}/update`, { method: "POST" });
    },
  };
}

export function AccountWindow({ accountId }: { accountId: string }) {
  const [state, setState] = useState<LoadState>({ status: "loading" });
  // Keep a stable ref so the menu-action handler can access the latest adapter
  // without re-subscribing every render.
  const adapterRef = useRef<CoreAdapter | null>(null);

  // DCC-C1: active plugins fetched from server for PluginRoot
  const [activePlugins, setActivePlugins] = useState<ActivePluginInfo[]>([]);

  // DCC-C1: which secondary screen is open (null = main content)
  type ActiveScreen = null | "plugins" | "agents";
  const [activeScreen, setActiveScreen] = useState<ActiveScreen>(null);

  // DCC-C2: agents state
  const [agents, setAgents] = useState<AgentInfo[]>([]);
  const [newTokenResult, setNewTokenResult] = useState<NewTokenResult | null>(null);

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

        // 5. Build the adapter (DCC-C2: pass serverUrl + authTokenFn for agent API).
        const adapter = new CoreAdapter(
          core,
          userId,
          /* userEmail */ "",
          account.server_url,
          () => authStorage.getToken(accountId),
        );
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
  // DCC-C1: Fetch active plugins from server when ready
  // ---------------------------------------------------------------------------

  useEffect(() => {
    if (state.status !== "ready") return;
    const { serverUrl } = state;

    async function fetchActivePlugins() {
      try {
        const tok = await authStorage.getToken(accountId);
        const headers: Record<string, string> = tok ? { Authorization: `Bearer ${tok}` } : {};
        const res = await fetch(`${serverUrl}/api/plugins/active`, { headers });
        if (!res.ok) return;
        const body = await res.json() as { plugins: ActivePluginInfo[] };
        setActivePlugins(body.plugins ?? []);
      } catch (err: unknown) {
        console.warn("[AccountWindow] fetchActivePlugins failed:", err);
      }
    }

    void fetchActivePlugins();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [state.status, accountId]);

  // ---------------------------------------------------------------------------
  // DCC-C2: Fetch agents when the agents screen is opened
  // ---------------------------------------------------------------------------

  const refreshAgents = useCallback(async () => {
    if (state.status !== "ready") return;
    try {
      const list = await state.adapter.agents.list();
      setAgents(list);
    } catch (err: unknown) {
      console.warn("[AccountWindow] agents.list failed:", err);
    }
  }, [state]);

  useEffect(() => {
    if (activeScreen === "agents") {
      void refreshAgents();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeScreen]);

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

  // Build stable PluginManagerAPI reference (recreated only when serverUrl changes).
  const pluginManagerAPI: PluginManagerAPI = buildPluginManagerAPI(
    state.serverUrl,
    () => authStorage.getToken(accountId),
  );

  return (
    <KrytonDataProvider adapter={state.adapter}>
      {/* DCC-C1: PluginRoot loads/unloads active client-side plugin bundles */}
      <PluginRoot activePlugins={activePlugins}>
        <AppShell
          header={
            <div className="flex items-center gap-3 px-4 py-2 text-sm font-medium text-surface-700 dark:text-surface-300">
              <span className="text-primary-600 font-semibold">Kryton</span>
              <span className="text-surface-400">/</span>
              <span>{state.accountLabel}</span>
              {/* Secondary-screen nav buttons */}
              <div className="ml-auto flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => setActiveScreen(activeScreen === "plugins" ? null : "plugins")}
                  className="rounded px-2 py-1 text-xs font-medium hover:bg-surface-100 dark:hover:bg-surface-800 data-[active=true]:text-primary-600"
                  data-active={activeScreen === "plugins"}
                  aria-label="Plugin Manager"
                >
                  Plugins
                </button>
                <button
                  type="button"
                  onClick={() => setActiveScreen(activeScreen === "agents" ? null : "agents")}
                  className="rounded px-2 py-1 text-xs font-medium hover:bg-surface-100 dark:hover:bg-surface-800 data-[active=true]:text-primary-600"
                  data-active={activeScreen === "agents"}
                  aria-label="Agents"
                >
                  Agents
                </button>
              </div>
            </div>
          }
        >
          {/* DCC-C1: Plugin Manager screen */}
          {activeScreen === "plugins" && (
            <PluginManagerScreen api={pluginManagerAPI} />
          )}

          {/* DCC-C2: Agents screen */}
          {activeScreen === "agents" && (
            <AgentsPanel
              agents={agents}
              newTokenResult={newTokenResult}
              onDismissNewToken={() => setNewTokenResult(null)}
              onCreateAgent={(name, label, cedarPolicy) => {
                state.adapter.agents
                  .create({ name, label, policyText: cedarPolicy })
                  .then(() => refreshAgents())
                  .catch((err: unknown) =>
                    console.warn("[AccountWindow] agents.create failed:", err),
                  );
              }}
              onDeleteAgent={(id) => {
                state.adapter.agents
                  .delete(id)
                  .then(() => refreshAgents())
                  .catch((err: unknown) =>
                    console.warn("[AccountWindow] agents.delete failed:", err),
                  );
              }}
              onMintToken={(agentId) => {
                state.adapter.agents
                  .mintToken(agentId)
                  .then((result) => setNewTokenResult(result))
                  .catch((err: unknown) =>
                    console.warn("[AccountWindow] agents.mintToken failed:", err),
                  );
              }}
              onRevokeToken={(agentId, tokenId) => {
                state.adapter.agents
                  .revokeToken(agentId, tokenId)
                  .then(() => refreshAgents())
                  .catch((err: unknown) =>
                    console.warn("[AccountWindow] agents.revokeToken failed:", err),
                  );
              }}
            />
          )}

          {/* Main content area */}
          {activeScreen === null && (
            <div className="flex h-full items-center justify-center text-surface-400 text-sm select-none">
              Ready — account window initialised.
            </div>
          )}
        </AppShell>
      </PluginRoot>
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

