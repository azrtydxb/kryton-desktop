#!/usr/bin/env node
/**
 * generate-updater-manifest.js
 *
 * Reads built artifacts from artifacts/macos/ and artifacts/windows/,
 * then writes latest.json — the Tauri updater manifest consumed by the
 * auto-update endpoint configured in tauri.conf.json.
 *
 * Environment variables required at runtime (set by the release workflow):
 *   RELEASE_TAG                       — e.g. "v1.2.3"
 *   GITHUB_REPO                       — e.g. "azrtydxb/kryton-desktop"
 *   TAURI_SIGNING_PRIVATE_KEY         — minisign private key (PEM string)
 *   TAURI_SIGNING_PRIVATE_KEY_PASSWORD — password for the private key (may be empty)
 *
 * The script shells out to `npx tauri signer sign` to produce .sig files for
 * any platform bundle that does not already have a companion .sig file.
 */

import { execSync } from "child_process";
import { existsSync, readFileSync, writeFileSync, readdirSync } from "fs";
import { join, basename } from "path";

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

const repoRoot = new URL("..", import.meta.url).pathname;
const macDir = join(repoRoot, "artifacts", "macos");
const winDir = join(repoRoot, "artifacts", "windows");

const releaseTag = process.env.RELEASE_TAG;
const githubRepo = process.env.GITHUB_REPO;

if (!releaseTag) {
  console.error("ERROR: RELEASE_TAG env var is not set");
  process.exit(1);
}
if (!githubRepo) {
  console.error("ERROR: GITHUB_REPO env var is not set");
  process.exit(1);
}

const version = releaseTag.replace(/^v/, "");
const releaseBaseUrl = `https://github.com/${githubRepo}/releases/download/${releaseTag}`;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * Sign a file with `tauri signer sign` and return the base64-encoded signature.
 * If a .sig file already exists next to the artifact (produced during the build),
 * it is read directly instead of re-signing.
 */
function getSignature(filePath) {
  const sigPath = filePath + ".sig";
  if (existsSync(sigPath)) {
    return readFileSync(sigPath, "utf8").trim();
  }

  // Fall back to signing on the fly.
  console.log(`  Signing ${basename(filePath)} …`);
  const result = execSync(
    `npx tauri signer sign --private-key-path /dev/stdin -- "${filePath}"`,
    {
      input: process.env.TAURI_SIGNING_PRIVATE_KEY,
      encoding: "utf8",
    }
  );
  // tauri signer sign writes the .sig file and prints the path.
  if (existsSync(sigPath)) {
    return readFileSync(sigPath, "utf8").trim();
  }
  // If no .sig file was written, parse the stdout (base64 signature).
  return result.trim();
}

/**
 * Find the first file in a directory matching a glob-like predicate.
 */
function findFirst(dir, predicate) {
  if (!existsSync(dir)) return null;
  const files = readdirSync(dir);
  const match = files.find(predicate);
  return match ? join(dir, match) : null;
}

// ---------------------------------------------------------------------------
// Locate artifacts
// ---------------------------------------------------------------------------

// macOS: updater bundle (.app.tar.gz)
const macUpdaterBundle = findFirst(macDir, (f) => f.endsWith(".app.tar.gz"));
// macOS: installer for humans (.dmg)
const macDmg = findFirst(macDir, (f) => f.endsWith(".dmg"));

// Windows MSI: updater bundle (.msi.zip) preferred; fall back to .msi
const winMsiZip = findFirst(winDir, (f) => f.endsWith(".msi.zip"));
const winMsi = findFirst(winDir, (f) => f.endsWith(".msi") && !f.endsWith(".zip"));

// Windows NSIS: updater bundle (.nsis.zip)
const winNsisZip = findFirst(winDir, (f) => f.endsWith(".nsis.zip"));
const winExe = findFirst(winDir, (f) => f.endsWith(".exe"));

// ---------------------------------------------------------------------------
// Build platform entries
// ---------------------------------------------------------------------------

const platforms = {};

if (macUpdaterBundle) {
  console.log(`macOS updater bundle: ${basename(macUpdaterBundle)}`);
  const sig = getSignature(macUpdaterBundle);
  const url = `${releaseBaseUrl}/${basename(macUpdaterBundle)}`;
  platforms["darwin-x86_64"] = { signature: sig, url };
  platforms["darwin-aarch64"] = { signature: sig, url };
} else {
  console.warn("WARNING: no macOS .app.tar.gz found in artifacts/macos/");
}

if (winMsiZip) {
  console.log(`Windows MSI zip: ${basename(winMsiZip)}`);
  const sig = getSignature(winMsiZip);
  platforms["windows-x86_64"] = {
    signature: sig,
    url: `${releaseBaseUrl}/${basename(winMsiZip)}`,
  };
} else if (winMsi) {
  console.log(`Windows MSI: ${basename(winMsi)}`);
  const sig = getSignature(winMsi);
  platforms["windows-x86_64"] = {
    signature: sig,
    url: `${releaseBaseUrl}/${basename(winMsi)}`,
  };
} else {
  console.warn("WARNING: no Windows MSI artifact found in artifacts/windows/");
}

// ---------------------------------------------------------------------------
// Write latest.json
// ---------------------------------------------------------------------------

const manifest = {
  version,
  notes: `See the release page for details: https://github.com/${githubRepo}/releases/tag/${releaseTag}`,
  pub_date: new Date().toISOString(),
  platforms,
};

const outPath = join(repoRoot, "latest.json");
writeFileSync(outPath, JSON.stringify(manifest, null, 2));
console.log(`\nWrote ${outPath}`);
console.log(JSON.stringify(manifest, null, 2));
