import { describe, it, expect, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => new Uint8Array()),
}));

vi.mock("sql.js", () => ({
  default: vi.fn(async () => ({
    Database: class {
      exec() {}
      prepare() {
        return {
          run: () => {},
          step: () => false,
          getAsObject: () => ({}),
          free: () => {},
          bind: () => {},
        };
      }
      export() {
        return new Uint8Array();
      }
      close() {}
      getRowsModified() {
        return 0;
      }
    },
  })),
}));

import { initDesktopCore } from "../desktop-init";

// Mock fetch so Kryton.init's version-compatibility check succeeds.
const mockFetch = vi.fn(async (_url: string) =>
  ({
    ok: true,
    json: async () => ({ apiVersion: "4.4.0" }),
  } as Response),
);

describe("initDesktopCore", () => {
  it("returns a Kryton instance", async () => {
    const k = await initDesktopCore({
      accountId: "acc1",
      serverUrl: "https://example.com",
      authToken: () => "T",
      fetch: mockFetch as unknown as typeof fetch,
    });
    expect(k).toBeDefined();
    await k.close();
  });
});
