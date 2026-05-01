import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Kryton } from "@azrtydxb/core";
import { LauncherApp } from "./LauncherApp";
import { AccountWindow } from "./AccountWindow";

console.log("Kryton class:", Kryton);

const windowLabel = getCurrentWebviewWindow().label;

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
