import { describe, it, expect, vi, beforeEach } from "vitest";
import { PersistenceManager } from "../persistence";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(async () => undefined) }));
import { invoke } from "@tauri-apps/api/core";

describe("PersistenceManager", () => {
  beforeEach(() => { vi.clearAllMocks(); });

  it("flushes after debounce", async () => {
    const adapter = { serialize: () => new Uint8Array([1, 2, 3]) } as any;
    const pm = new PersistenceManager(adapter, "acc1", 50);
    pm.scheduleFlush();
    await new Promise((r) => setTimeout(r, 100));
    expect(invoke).toHaveBeenCalledWith("write_db", { accountId: "acc1", bytes: [1, 2, 3] });
  });

  it("debounces multiple schedules into one flush", async () => {
    const adapter = { serialize: () => new Uint8Array([1]) } as any;
    const pm = new PersistenceManager(adapter, "acc1", 50);
    pm.scheduleFlush();
    pm.scheduleFlush();
    pm.scheduleFlush();
    await new Promise((r) => setTimeout(r, 100));
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("flushNow flushes immediately", async () => {
    const adapter = { serialize: () => new Uint8Array([5]) } as any;
    const pm = new PersistenceManager(adapter, "acc1", 5000);
    pm.scheduleFlush();
    await pm.flushNow();
    expect(invoke).toHaveBeenCalled();
  });
});
