import { ipc, windows } from "../shared/ipc";

const form = document.getElementById("login-form") as HTMLFormElement;
const err = document.getElementById("error") as HTMLParagraphElement;
const submitBtn = document.getElementById("submit-btn") as HTMLButtonElement;
const submitLabel = document.getElementById("submit-label") as HTMLSpanElement;

form.addEventListener("submit", async (e) => {
  e.preventDefault();
  err.hidden = true;
  submitBtn.disabled = true;
  submitLabel.textContent = "Connecting…";

  const data = new FormData(form);
  try {
    const acct = await ipc.loginAndAdd({
      label: String(data.get("label")),
      serverUrl: String(data.get("serverUrl")),
      username: String(data.get("username")),
      password: String(data.get("password")),
    });
    await ipc.refreshMenu();
    await windows.openServer(acct.id);
  } catch (e: unknown) {
    err.textContent = String(e);
    err.hidden = false;
    submitBtn.disabled = false;
    submitLabel.textContent = "Add server";
  }
});
