/**
 * CoreAdapter — bridges a Kryton instance to the KrytonDataAdapter interface
 * expected by @azrtydxb/ui's KrytonDataProvider.
 *
 * All 14 adapter method groups are implemented here.
 * Agent API methods (DCC-C2) call /api/agents/* directly with a bearer token.
 */
import type { Kryton } from "@azrtydxb/core";
import type {
  KrytonDataAdapter,
  NoteData,
  FolderData,
  TagData,
  NoteShareData,
  TrashItemData,
  CurrentUser,
  SyncStatus,
  NoteFilter,
  AgentInfo,
  NewTokenResult,
} from "@azrtydxb/ui";
import type * as Y from "yjs";
import type { Awareness } from "y-protocols/awareness";

function uuid(): string {
  // Use crypto.randomUUID if available, fallback to a simple timestamp-based ID.
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

export class CoreAdapter implements KrytonDataAdapter {
  /**
   * @param core         - Initialised Kryton instance.
   * @param userId       - The authenticated user's id.
   * @param userEmail    - Optional email for currentUser placeholder.
   * @param serverUrl    - Base URL of the Kryton server (for agent API calls).
   * @param authTokenFn  - Returns the current bearer token (for agent API calls).
   */
  constructor(
    private readonly core: Kryton,
    private readonly userId: string,
    private readonly userEmail: string = "",
    private readonly serverUrl: string = "",
    private readonly authTokenFn: () => string | null | Promise<string | null> = () => null,
  ) {}

  // ---------------------------------------------------------------------------
  // notes
  // ---------------------------------------------------------------------------

  notes: KrytonDataAdapter["notes"] = {
    list: (filter?: NoteFilter): NoteData[] => {
      let rows: ReturnType<Kryton["notes"]["list"]>;
      if (filter?.folderPath) {
        rows = this.core.notes.listByFolder(filter.folderPath);
      } else {
        rows = this.core.notes.list();
      }

      // Client-side tag filter
      if (filter?.tag) {
        const tag = filter.tag;
        rows = rows.filter((n) => {
          try {
            const tags: string[] = JSON.parse(n.tags ?? "[]");
            return tags.includes(tag);
          } catch {
            return false;
          }
        });
      }

      return rows as NoteData[];
    },

    findById: (id: string): NoteData | null => {
      return (this.core.notes.findById(id) ?? null) as NoteData | null;
    },

    findByPath: (path: string): NoteData | null => {
      return (this.core.notes.findByPath(path) ?? null) as NoteData | null;
    },

    create: async (input: {
      path: string;
      title: string;
      content?: string;
      tags?: string[];
    }): Promise<NoteData> => {
      const id = uuid();
      const row = this.core.notes.create({
        id,
        path: input.path,
        title: input.title,
        tags: JSON.stringify(input.tags ?? []),
        modifiedAt: Date.now(),
        version: 0,
      });
      return row as NoteData;
    },

    update: async (
      id: string,
      patch: Partial<NoteData> & { content?: string },
    ): Promise<void> => {
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { content: _content, ...rest } = patch;
      if (Object.keys(rest).length > 0) {
        this.core.notes.update(id, { ...rest, modifiedAt: Date.now() });
      }
      // content is managed via Yjs documents; ignore here
    },

    delete: async (id: string): Promise<void> => {
      this.core.notes.delete(id);
    },
  };

  // ---------------------------------------------------------------------------
  // folders
  // ---------------------------------------------------------------------------

  folders: KrytonDataAdapter["folders"] = {
    list: (): FolderData[] => {
      return this.core.folders.list() as FolderData[];
    },

    create: async (input: {
      path: string;
      parentId: string | null;
    }): Promise<FolderData> => {
      const id = uuid();
      const row = this.core.folders.create({
        id,
        userId: this.userId,
        path: input.path,
        parentId: input.parentId,
        updatedAt: Date.now(),
        version: 0,
        cursor: 0,
      });
      return row as FolderData;
    },

    delete: async (id: string): Promise<void> => {
      this.core.folders.delete(id);
    },
  };

  // ---------------------------------------------------------------------------
  // tags
  // ---------------------------------------------------------------------------

  tags: KrytonDataAdapter["tags"] = {
    list: (): TagData[] => {
      return this.core.tags.list() as TagData[];
    },
  };

  // ---------------------------------------------------------------------------
  // settings
  // ---------------------------------------------------------------------------

  settings: KrytonDataAdapter["settings"] = {
    get: (key: string): string | null => {
      const row = this.core.settings.get(this.userId, key);
      return row?.value ?? null;
    },

    set: async (key: string, value: string): Promise<void> => {
      this.core.settings.set(this.userId, key, value);
    },
  };

  // ---------------------------------------------------------------------------
  // noteShares
  // ---------------------------------------------------------------------------

  noteShares: KrytonDataAdapter["noteShares"] = {
    list: (): NoteShareData[] => {
      return this.core.noteShares.list() as NoteShareData[];
    },
  };

  // ---------------------------------------------------------------------------
  // trashItems
  // ---------------------------------------------------------------------------

  trashItems: KrytonDataAdapter["trashItems"] = {
    list: (): TrashItemData[] => {
      return this.core.trashItems.list() as TrashItemData[];
    },

    restore: async (id: string): Promise<void> => {
      // Restore: look up the trashItem, re-create the note at its original path,
      // then delete the trash_item row.
      const item = this.core.trashItems.findById(id);
      if (!item) return;
      // Re-create the note at the original path using the base title (filename)
      const title = item.originalPath.split("/").pop() ?? item.originalPath;
      const noteId = uuid();
      this.core.notes.create({
        id: noteId,
        path: item.originalPath,
        title,
        tags: "[]",
        modifiedAt: Date.now(),
        version: 0,
      });
      this.core.trashItems.delete(id);
    },

    purge: async (id: string): Promise<void> => {
      this.core.trashItems.delete(id);
    },

    purgeAll: async (): Promise<void> => {
      const items = this.core.trashItems.list();
      for (const item of items) {
        this.core.trashItems.delete(item.id);
      }
    },
  };

  // ---------------------------------------------------------------------------
  // subscribe
  // ---------------------------------------------------------------------------

  subscribe(
    entityType: string,
    _ids: string[] | "*",
    callback: () => void,
  ): () => void {
    // Subscribe to the "change" bus event and call back when the entity type matches.
    const off = this.core.bus.on(
      "change",
      (e: { entityType?: string; source?: string }) => {
        if (!e.entityType || e.entityType === entityType) {
          callback();
        }
      },
    );
    return off;
  }

  // ---------------------------------------------------------------------------
  // openDocument / closeDocument / getAwareness / readNoteContent
  // ---------------------------------------------------------------------------

  async openDocument(noteId: string): Promise<Y.Doc> {
    return this.core.yjs.openDocument(noteId);
  }

  closeDocument(noteId: string): void {
    // YjsManager.closeDocument is async; fire-and-forget is acceptable here
    // since callers expect a synchronous close signal.
    this.core.yjs.closeDocument(noteId).catch((err: unknown) => {
      console.warn("[CoreAdapter] closeDocument error:", err);
    });
  }

  getAwareness(noteId: string): Awareness | null {
    return this.core.yjs.getAwareness(noteId);
  }

  readNoteContent(noteId: string): string | null {
    return this.core.readNoteContent(noteId);
  }

  // ---------------------------------------------------------------------------
  // getSyncStatus / triggerSync
  // ---------------------------------------------------------------------------

  getSyncStatus(): SyncStatus {
    // The SyncOrchestrator does not expose status fields directly.
    // Derive a best-effort status from LocalStorage keys written after each sync cycle.
    const lastPullAtRaw = this.core.storage.get("last_pull_at", "");
    const lastPushAtRaw = this.core.storage.get("last_push_at", "");
    const pendingRaw = this.core.storage.get("pending_count", "0");

    return {
      lastPullAt: lastPullAtRaw ? Number(lastPullAtRaw) : null,
      lastPushAt: lastPushAtRaw ? Number(lastPushAtRaw) : null,
      pending: Number(pendingRaw) || 0,
      // Treat as online if we've received at least one pull response.
      online: !!lastPullAtRaw,
    };
  }

  async triggerSync(): Promise<void> {
    await this.core.sync.full();
    // Record timestamps so getSyncStatus() can reflect them.
    const now = String(Date.now());
    this.core.storage.set("last_pull_at", now);
    this.core.storage.set("last_push_at", now);
  }

  // ---------------------------------------------------------------------------
  // currentUser
  // ---------------------------------------------------------------------------

  currentUser(): CurrentUser | null {
    if (!this.userId) return null;
    return {
      id: this.userId,
      email: this.userEmail,
      displayName: this.userEmail.split("@")[0] || this.userId,
    };
  }

  // ---------------------------------------------------------------------------
  // agents  (DCC-C2) — calls /api/agents/* directly via fetch + bearer token
  // ---------------------------------------------------------------------------

  /** Resolve the auth token (supports both sync and async token accessors). */
  private async resolveToken(): Promise<string | null> {
    const t = this.authTokenFn();
    return t instanceof Promise ? await t : t;
  }

  private authHeader(token: string | null): Record<string, string> {
    return token ? { Authorization: `Bearer ${token}` } : {};
  }

  agents = {
    list: async (): Promise<AgentInfo[]> => {
      const tok = await this.resolveToken();
      const res = await fetch(`${this.serverUrl}/api/agents`, {
        headers: this.authHeader(tok),
      });
      if (!res.ok) throw new Error(`agents.list: ${res.status}`);
      const body = await res.json() as { agents: AgentInfo[] };
      return body.agents ?? [];
    },

    create: async (input: {
      name: string;
      label: string;
      policyText?: string;
    }): Promise<AgentInfo> => {
      const tok = await this.resolveToken();
      const res = await fetch(`${this.serverUrl}/api/agents`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...this.authHeader(tok) },
        body: JSON.stringify(input),
      });
      if (!res.ok) throw new Error(`agents.create: ${res.status}`);
      return res.json() as Promise<AgentInfo>;
    },

    delete: async (id: string): Promise<void> => {
      const tok = await this.resolveToken();
      const res = await fetch(`${this.serverUrl}/api/agents/${id}`, {
        method: "DELETE",
        headers: this.authHeader(tok),
      });
      if (!res.ok) throw new Error(`agents.delete: ${res.status}`);
    },

    mintToken: async (agentId: string): Promise<NewTokenResult> => {
      const tok = await this.resolveToken();
      const res = await fetch(`${this.serverUrl}/api/agents/${agentId}/tokens`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...this.authHeader(tok) },
        body: JSON.stringify({}),
      });
      if (!res.ok) throw new Error(`agents.mintToken: ${res.status}`);
      const body = await res.json() as { tokenId: string; token: string };
      return { agentId, tokenId: body.tokenId, token: body.token };
    },

    revokeToken: async (agentId: string, tokenId: string): Promise<void> => {
      const tok = await this.resolveToken();
      const res = await fetch(`${this.serverUrl}/api/agents/${agentId}/tokens/${tokenId}`, {
        method: "DELETE",
        headers: this.authHeader(tok),
      });
      if (!res.ok) throw new Error(`agents.revokeToken: ${res.status}`);
    },

    setPolicy: async (agentId: string, policyText: string): Promise<void> => {
      const tok = await this.resolveToken();
      const res = await fetch(`${this.serverUrl}/api/agents/${agentId}/policy`, {
        method: "PUT",
        headers: { "Content-Type": "application/json", ...this.authHeader(tok) },
        body: JSON.stringify({ policyText }),
      });
      if (!res.ok) throw new Error(`agents.setPolicy: ${res.status}`);
    },
  };
}
