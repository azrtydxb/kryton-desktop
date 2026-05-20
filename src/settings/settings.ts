import { invoke } from "@tauri-apps/api/core";
import { ipc, windows } from "../shared/ipc";

async function render() {
  const accounts = await ipc.listAccounts();
  const ul = document.getElementById("accounts") as HTMLUListElement;
  ul.innerHTML = "";
  for (const a of accounts) {
    const li = document.createElement("li");
    li.innerHTML = `
      <strong>${a.label}</strong>
      <small>${a.server_url} — ${a.username}</small>
      <button data-act="switch">Switch to</button>
      <button data-act="remove">Remove</button>
    `;
    li.querySelector('[data-act="switch"]')!.addEventListener("click", async () => {
      await windows.switchTo(a.id);
    });
    li.querySelector('[data-act="remove"]')!.addEventListener("click", async () => {
      if (!confirm(`Remove ${a.label}? This deletes the saved password.`)) return;
      await windows.closeServer(a.id);
      await ipc.removeAccount(a.id);
      await ipc.refreshMenu();
      await render();
    });
    ul.appendChild(li);
  }
}

document.getElementById("add")!.addEventListener("click", async () => {
  await invoke<void>("open_add_server");
});

render();
