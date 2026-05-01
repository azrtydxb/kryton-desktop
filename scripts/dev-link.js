#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
import { execSync } from "node:child_process";
import { resolve, isAbsolute } from "node:path";

const action = process.argv[2];
if (!["link", "unlink", "verify"].includes(action)) {
  console.error("Usage: node scripts/dev-link.js [link|unlink|verify]");
  process.exit(1);
}

// Local path resolution:
//   KRYTON_LOCAL_PATH env var can override the base path.
//   Default: sibling packages directory ../kryton/packages/<pkg-name>
const basePathInput = process.env.KRYTON_LOCAL_PATH ?? "../kryton/packages";
const basePath = isAbsolute(basePathInput) ? basePathInput : resolve(basePathInput);

const corePath = resolve(basePath, "core");
const reactPath = resolve(basePath, "core-react");
const uiPath = resolve(basePath, "ui");

const pkgFile = resolve("package.json");
const pkg = JSON.parse(readFileSync(pkgFile, "utf8"));

function deps() { return pkg.dependencies ?? (pkg.dependencies = {}); }

function isLinked() {
  const c = deps()["@azrtydxb/core"];
  const r = deps()["@azrtydxb/core-react"];
  const u = deps()["@azrtydxb/ui"];
  return (
    (c && c.startsWith("file:")) ||
    (r && r.startsWith("file:")) ||
    (u && u.startsWith("file:"))
  );
}

if (action === "verify") {
  if (isLinked()) {
    console.error("ERROR: package.json contains file: deps for @azrtydxb/*");
    console.error("Run `npm run dev:unlink` before committing.");
    process.exit(1);
  }
  console.log("OK: no file: deps for @azrtydxb/*");
  process.exit(0);
}

if (action === "link") {
  deps()["@azrtydxb/core"] = `file:${corePath}`;
  deps()["@azrtydxb/core-react"] = `file:${reactPath}`;
  deps()["@azrtydxb/ui"] = `file:${uiPath}`;
  writeFileSync(pkgFile, JSON.stringify(pkg, null, 2) + "\n");
  execSync("npm install --legacy-peer-deps", { stdio: "inherit" });
  console.log(`Linked. core=${corePath} core-react=${reactPath} ui=${uiPath}`);
  console.log("Run `npm run dev:unlink` to restore published versions.");
  process.exit(0);
}

if (action === "unlink") {
  const headPkg = JSON.parse(execSync("git show HEAD:package.json", { encoding: "utf8" }));
  const coreVer = headPkg.dependencies?.["@azrtydxb/core"];
  const reactVer = headPkg.dependencies?.["@azrtydxb/core-react"];
  const uiVer = headPkg.dependencies?.["@azrtydxb/ui"];
  if (!coreVer || !reactVer || !uiVer) {
    console.error("Cannot find @azrtydxb/* deps in HEAD package.json");
    process.exit(1);
  }
  deps()["@azrtydxb/core"] = coreVer;
  deps()["@azrtydxb/core-react"] = reactVer;
  deps()["@azrtydxb/ui"] = uiVer;
  writeFileSync(pkgFile, JSON.stringify(pkg, null, 2) + "\n");
  execSync("npm install --legacy-peer-deps", { stdio: "inherit" });
  console.log(`Unlinked. core=${coreVer} core-react=${reactVer} ui=${uiVer}`);
}
