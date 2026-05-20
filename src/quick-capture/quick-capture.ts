import { capture } from "../shared/ipc";
import { getCurrentWindow } from "@tauri-apps/api/window";

const form = document.getElementById("capture") as HTMLFormElement;
const status = document.getElementById("status") as HTMLSpanElement;

form.addEventListener("submit", async (e) => {
  e.preventDefault();
  status.textContent = "Saving…";
  const body = String(new FormData(form).get("body")).trim();
  if (!body) { status.textContent = "Empty"; return; }
  try {
    await capture.toActive(body);
    await getCurrentWindow().close();
  } catch (e: unknown) {
    status.textContent = String(e);
  }
});

document.addEventListener("keydown", async (e) => {
  if (e.key === "Escape") await getCurrentWindow().close();
});
