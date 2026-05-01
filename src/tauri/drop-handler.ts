/**
 * drop-handler.ts — handles file drag-and-drop onto an AccountWindow.
 *
 * .md / .markdown files → POST to /api/notes
 * image / PDF files     → POST to /api/attachments
 */

import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { readTextFile, readFile } from "@tauri-apps/plugin-fs";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface DropHandlerOptions {
  /** Base URL of the Kryton server, e.g. "https://my.kryton.app". */
  serverUrl: string;
  /** Returns the current auth token (or null if unauthenticated). */
  authToken: () => Promise<string | null>;
  /** Optional progress callback — fired for each file processed. */
  onProgress?: (current: number, total: number) => void;
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

/**
 * Subscribe to drag-drop events on the current webview window.
 * Should be called once per AccountWindow mount.
 *
 * Returns an unlisten function that removes the handler.
 */
export async function setupDropHandler(
  opts: DropHandlerOptions,
): Promise<() => void> {
  const win = getCurrentWebviewWindow();
  const unlisten = await win.onDragDropEvent(async (event) => {
    if (event.payload.type !== "drop") return;

    const paths = event.payload.paths;
    let processed = 0;

    for (const path of paths) {
      processed++;
      opts.onProgress?.(processed, paths.length);

      try {
        if (/\.(md|markdown)$/i.test(path)) {
          const content = await readTextFile(path);
          const filename = path.split("/").pop() ?? "imported.md";
          const token = await opts.authToken();
          await uploadNote(opts.serverUrl, token, filename, content);
        } else if (/\.(png|jpg|jpeg|gif|pdf|webp)$/i.test(path)) {
          const bytes = await readFile(path);
          const filename = path.split("/").pop() ?? "attachment";
          const token = await opts.authToken();
          await uploadAttachment(opts.serverUrl, token, filename, bytes);
        }
      } catch (e) {
        console.warn("[drop-handler] import failed for", path, e);
      }
    }
  });

  return unlisten;
}

// ---------------------------------------------------------------------------
// Upload helpers
// ---------------------------------------------------------------------------

async function uploadNote(
  serverUrl: string,
  token: string | null,
  filename: string,
  content: string,
): Promise<void> {
  const res = await fetch(`${serverUrl}/api/notes`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify({ path: filename, content }),
  });
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`POST /api/notes failed: ${res.status} ${text}`);
  }
}

async function uploadAttachment(
  serverUrl: string,
  token: string | null,
  filename: string,
  bytes: Uint8Array,
): Promise<void> {
  const fd = new FormData();
  fd.append("file", new Blob([bytes as BlobPart]), filename);
  fd.append("notePath", "");
  const res = await fetch(`${serverUrl}/api/attachments`, {
    method: "POST",
    headers: {
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: fd,
  });
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`POST /api/attachments failed: ${res.status} ${text}`);
  }
}
