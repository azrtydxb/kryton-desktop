import { ipc, windows } from "../shared/ipc";

const form = document.getElementById("login-form") as HTMLFormElement;
const err = document.getElementById("error") as HTMLParagraphElement;

form.addEventListener("submit", async (e) => {
  e.preventDefault();
  err.hidden = true;
  const data = new FormData(form);
  try {
    const acct = await ipc.loginAndAdd({
      label: String(data.get("label")),
      serverUrl: String(data.get("serverUrl")),
      username: String(data.get("username")),
      password: String(data.get("password")),
    });
    await ipc.refreshMenu();
    await invokeOpenServer(acct.id);
  } catch (e: unknown) {
    err.textContent = String(e);
    err.hidden = false;
  }
});

async function invokeOpenServer(_id: string) {
  await windows.openServer(_id);
}
