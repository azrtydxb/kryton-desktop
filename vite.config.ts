import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));

// Inline the SQL schema so it can be used in the browser/WebView.
// The @azrtydxb/core package uses node:fs to read schema.sql at runtime,
// which doesn't work in a browser environment. We provide stub modules
// via alias so the build succeeds.
const schemaSqlPath = resolve(
  __dirname,
  "node_modules/@azrtydxb/core/dist/generated/schema.sql"
);
const schemaSql = readFileSync(schemaSqlPath, "utf8");

// Write stub modules that Vite will alias into
import { mkdirSync, writeFileSync } from "node:fs";
const stubsDir = resolve(__dirname, "node_modules/.vite-stubs");
mkdirSync(stubsDir, { recursive: true });

writeFileSync(
  resolve(stubsDir, "node-fs.js"),
  `
export const readFileSync = (path, enc) => {
  if (String(path).endsWith("schema.sql")) {
    return ${JSON.stringify(schemaSql)};
  }
  throw new Error("readFileSync not available in browser: " + path);
};
export default { readFileSync };
`
);

writeFileSync(
  resolve(stubsDir, "node-path.js"),
  `
export const resolve = (...args) => args.join("/");
export const dirname = (p) => p.substring(0, p.lastIndexOf("/")) || ".";
export const join = (...args) => args.join("/");
export const basename = (p, ext) => {
  const base = p.split("/").pop() || p;
  return ext && base.endsWith(ext) ? base.slice(0, -ext.length) : base;
};
export default { resolve, dirname, join, basename };
`
);

writeFileSync(
  resolve(stubsDir, "node-url.js"),
  `
export const fileURLToPath = (url) => {
  return typeof url === "string"
    ? url.replace(/^file:\\/\\//, "")
    : (url && url.pathname) || "";
};
export const pathToFileURL = (p) => new URL("file://" + p);
export default { fileURLToPath, pathToFileURL };
`
);

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],

  resolve: {
    alias: {
      "node:fs": resolve(stubsDir, "node-fs.js"),
      "node:path": resolve(stubsDir, "node-path.js"),
      "node:url": resolve(stubsDir, "node-url.js"),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  build: {
    // Tauri uses Chromium on macOS so ES2021 is fine
    target: ["es2021", "chrome100", "safari13"],
    // do not minify small chunks
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
