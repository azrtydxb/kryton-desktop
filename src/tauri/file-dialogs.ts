/**
 * file-dialogs.ts — Tauri file-dialog helpers for Kryton Desktop.
 *
 * DC-E2: Wraps @tauri-apps/plugin-dialog + @tauri-apps/plugin-fs to expose:
 *   - pickMarkdownFile()  – open-file dialog filtered to .md/.markdown
 *   - exportNote()        – save-file dialog that writes content to disk
 */

import { open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface PickedFile {
  /** Absolute path chosen by the user. */
  path: string;
  /** Raw text content of the file. */
  content: string;
}

// ---------------------------------------------------------------------------
// pickMarkdownFile
// ---------------------------------------------------------------------------

/**
 * Open a native file-picker dialog filtered to Markdown files.
 * Returns the selected file's path + content, or `null` if the user
 * dismissed the dialog.
 */
export async function pickMarkdownFile(): Promise<PickedFile | null> {
  const selected = await open({
    title: "Open Markdown File",
    multiple: false,
    filters: [
      { name: "Markdown", extensions: ["md", "markdown"] },
    ],
  });

  if (selected === null) {
    // User cancelled.
    return null;
  }

  // In Tauri 2.x plugin-dialog, `open` with multiple:false returns a string
  // path or null.
  const path = typeof selected === "string" ? selected : (selected as { path: string }).path;
  const content = await readTextFile(path);
  return { path, content };
}

// ---------------------------------------------------------------------------
// exportNote
// ---------------------------------------------------------------------------

/**
 * Open a native save-file dialog and write `content` to the chosen path.
 * Returns the path the file was saved to, or `null` if the user cancelled.
 *
 * @param filename  Suggested filename (e.g. "My Note.md").
 * @param content   The text content to write.
 */
export async function exportNote(
  filename: string,
  content: string,
): Promise<string | null> {
  const savePath = await save({
    title: "Export Note",
    defaultPath: filename,
    filters: [
      { name: "Markdown", extensions: ["md", "markdown"] },
      { name: "All Files", extensions: ["*"] },
    ],
  });

  if (savePath === null) {
    // User cancelled.
    return null;
  }

  await writeTextFile(savePath, content);
  return savePath;
}
