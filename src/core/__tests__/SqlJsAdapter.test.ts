import initSqlJs from "sql.js";
// @azrtydxb/core publishes the conformance suite in dist/__tests__ but does not
// expose it via the exports map, so we use a local copy (see conformance.ts).
import { runConformanceSuite } from "./conformance";
import { SqlJsAdapter } from "../SqlJsAdapter";

const SQL = await initSqlJs();

runConformanceSuite("SqlJsAdapter", () => {
  return new SqlJsAdapter(new SQL.Database());
});
