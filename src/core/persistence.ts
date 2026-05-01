import { invoke } from "@tauri-apps/api/core";

export interface SerializableAdapter {
  serialize(): Uint8Array;
}

export class PersistenceManager {
  private timer: ReturnType<typeof setTimeout> | null = null;
  private flushing: Promise<void> | null = null;

  constructor(
    private readonly adapter: SerializableAdapter,
    private readonly accountId: string,
    private readonly debounceMs: number = 500,
  ) {}

  scheduleFlush(): void {
    if (this.timer !== null) {
      clearTimeout(this.timer);
    }
    this.timer = setTimeout(() => {
      this.timer = null;
      this._flush();
    }, this.debounceMs);
  }

  async flushNow(): Promise<void> {
    if (this.timer !== null) {
      clearTimeout(this.timer);
      this.timer = null;
    }
    await this._flush();
  }

  private _flush(): Promise<void> {
    this.flushing = (async () => {
      const bytes = Array.from(this.adapter.serialize());
      await invoke("write_db", { accountId: this.accountId, bytes });
    })();
    return this.flushing;
  }
}
