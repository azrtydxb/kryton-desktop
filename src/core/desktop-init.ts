import { Kryton, type KrytonInitOpts } from "@azrtydxb/core";
import initSqlJs from "sql.js";
import { invoke } from "@tauri-apps/api/core";
import { SqlJsAdapter } from "./SqlJsAdapter";
import { PersistenceManager } from "./persistence";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
let SQL: any | null = null;

export interface DesktopInitOpts {
  accountId: string;
  serverUrl: string;
  authToken: () => string | null | Promise<string | null>;
  /** Override fetch for testing. Defaults to global fetch. */
  fetch?: typeof fetch;
}

export async function initDesktopCore(opts: DesktopInitOpts): Promise<Kryton> {
  if (!SQL) {
    SQL = await initSqlJs({
      locateFile: (file: string) => `/${file}`, // sql-wasm.wasm served from public/
    });
  }

  // 1. Read existing DB blob
  const bytesResp = await invoke<number[] | Uint8Array>("read_db", {
    accountId: opts.accountId,
  });
  const bytes =
    bytesResp instanceof Uint8Array ? bytesResp : new Uint8Array(bytesResp);

  // 2. Create SqlJsAdapter
  const db = bytes.length > 0 ? new SQL.Database(bytes) : new SQL.Database();
  const adapter = new SqlJsAdapter(db);

  // 3. Init Kryton
  const initOpts: KrytonInitOpts = {
    adapter,
    serverUrl: opts.serverUrl,
    authToken: opts.authToken,
  };
  if (opts.fetch) initOpts.fetch = opts.fetch;
  const k = await Kryton.init(initOpts);

  // 4. Wire persistence: schedule flush on every local change
  const pm = new PersistenceManager(adapter, opts.accountId, 500);
  k.bus.on("change", (e: { source?: string }) => {
    if (e.source === "local") pm.scheduleFlush();
  });
  k.bus.on("yjs:update", () => pm.scheduleFlush());

  // 5. Final flush on close
  const origClose = k.close.bind(k);
  (k as unknown as { close: () => Promise<void> }).close = async () => {
    await pm.flushNow();
    return origClose();
  };

  return k;
}
