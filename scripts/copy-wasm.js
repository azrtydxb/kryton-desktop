#!/usr/bin/env node
// Copies sql-wasm.wasm from node_modules/sql.js/dist into public/ so Tauri can serve it.
import { mkdirSync, copyFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "..");
const src = resolve(root, "node_modules/sql.js/dist/sql-wasm.wasm");
const destDir = resolve(root, "public");
const dest = resolve(destDir, "sql-wasm.wasm");

if (!existsSync(src)) {
  console.error("copy-wasm: sql-wasm.wasm not found at", src);
  process.exit(1);
}

mkdirSync(destDir, { recursive: true });
copyFileSync(src, dest);
console.log("copy-wasm: copied sql-wasm.wasm to public/");
