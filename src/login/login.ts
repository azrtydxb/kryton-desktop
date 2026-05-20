import { ipc } from "../shared/ipc";

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
    // Phase 3 will open the server window here. For now: just show placeholder.
    await invokeOpenServer(acct.id);
  } catch (e: unknown) {
    err.textContent = String(e);
    err.hidden = false;
  }
});

async function invokeOpenServer(_id: string) {
  // Stub. Filled in in Phase 3.
  document.body.innerHTML =
    "<p>Account added. Phase 3 will open the server window here.</p>";
}
