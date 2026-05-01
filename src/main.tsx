import React from "react";
import ReactDOM from "react-dom/client";
import "./globals.css";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Kryton } from "@azrtydxb/core";
import { LauncherApp } from "./LauncherApp";
import { AccountWindow } from "./AccountWindow";
import { check as checkUpdate } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

console.log("Kryton class:", Kryton);

// Check for updates on startup (launcher window only; silent — no prompt until
// an update is confirmed available). Production builds only: the updater
// endpoint returns 404 in dev, so errors are swallowed silently.
const windowLabel = getCurrentWebviewWindow().label;

if (windowLabel === "launcher") {
  checkUpdate()
    .then(async (update) => {
      if (update?.available) {
        console.log(
          `[updater] new version available: ${update.version} — downloading…`
        );
        await update.downloadAndInstall();
        await relaunch();
      }
    })
    .catch((err) => {
      // Silently ignore — e.g. offline, dev build, no endpoint configured.
      console.debug("[updater] check skipped:", err);
    });
}

let app: React.ReactNode;

if (windowLabel.startsWith("account-")) {
  const accountId = windowLabel.slice("account-".length);
  app = <AccountWindow accountId={accountId} />;
} else {
  // "launcher" label, or any unexpected label, renders the launcher
  app = <LauncherApp />;
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>{app}</React.StrictMode>
);
