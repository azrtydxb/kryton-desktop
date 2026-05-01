import type { Database, SqlValue, Statement } from "sql.js";
import type { SqliteAdapter, SqliteRunResult, Row } from "@azrtydxb/core";

export class SqlJsAdapter implements SqliteAdapter {
  private db: Database;
  private closed = false;

  constructor(db: Database) {
    this.db = db;
  }

  exec(sql: string): void {
    this.assertOpen();
    this.db.exec(sql);
  }

  run(sql: string, params: readonly unknown[] = []): SqliteRunResult {
    this.assertOpen();
    const stmt: Statement = this.db.prepare(sql);
    try {
      stmt.run(params as SqlValue[]);
      const changes = this.db.getRowsModified();
      return { changes, lastInsertRowid: 0 };
    } finally {
      stmt.free();
    }
  }

  get<R = Row>(sql: string, params: readonly unknown[] = []): R | undefined {
    this.assertOpen();
    const stmt: Statement = this.db.prepare(sql);
    try {
      stmt.bind(params as SqlValue[]);
      if (!stmt.step()) return undefined;
      return stmt.getAsObject() as R;
    } finally {
      stmt.free();
    }
  }

  all<R = Row>(sql: string, params: readonly unknown[] = []): R[] {
    this.assertOpen();
    const stmt: Statement = this.db.prepare(sql);
    try {
      stmt.bind(params as SqlValue[]);
      const rows: R[] = [];
      while (stmt.step()) {
        rows.push(stmt.getAsObject() as R);
      }
      return rows;
    } finally {
      stmt.free();
    }
  }

  transaction<T>(fn: () => T): T {
    this.assertOpen();
    this.db.exec("BEGIN");
    try {
      const result = fn();
      this.db.exec("COMMIT");
      return result;
    } catch (err) {
      this.db.exec("ROLLBACK");
      throw err;
    }
  }

  close(): void {
    if (!this.closed) {
      this.closed = true;
      this.db.close();
    }
  }

  /** Export the database as a Uint8Array for persistence. */
  serialize(): Uint8Array {
    return this.db.export();
  }

  private assertOpen(): void {
    if (this.closed) throw new Error("SqlJsAdapter: database is closed");
  }
}
