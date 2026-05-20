import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ipc, windows } from "../shared/ipc";
import { bootstrapTheme, currentTheme, setTheme, type Theme } from "../shared/theme";

bootstrapTheme().then(renderTheme);

function renderTheme() {
  const container = document.getElementById("theme-picker") as HTMLDivElement | null;
  if (!container) return;
  const active = currentTheme();
  container.innerHTML = "";
  for (const t of ["system", "dark", "light"] as const) {
    const b = document.createElement("button");
    b.textContent = t;
    b.setAttribute("aria-pressed", String(t === active));
    b.addEventListener("click", async () => {
      await setTheme(t as Theme);
      renderTheme();
    });
    container.appendChild(b);
  }
}

listen<string>("update:available", async (ev) => {
  if (confirm(`Update ${ev.payload} is available. Install and restart?`)) {
    await invoke("apply_update");
  }
});

function esc(s: string): string {
  return s.replace(/[&<>"']/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c] as string,
  );
}

async function render() {
  const accounts = await ipc.listAccounts();
  const ul = document.getElementById("accounts") as HTMLUListElement;
  ul.innerHTML = "";
  if (accounts.length === 0) {
    const empty = document.createElement("li");
    empty.className = "account-row";
    empty.style.gridTemplateColumns = "1fr";
    empty.innerHTML = `<span class="meta">No servers yet. Add one to get started.</span>`;
    ul.appendChild(empty);
    return;
  }
  for (const a of accounts) {
    const li = document.createElement("li");
    li.className = "account-row";
    li.innerHTML = `
      <div>
        <span class="label">${esc(a.label)}</span>
        <span class="meta">${esc(a.server_url)} · ${esc(a.username)}</span>
      </div>
      <button class="btn" data-act="switch">Switch to</button>
      <button class="btn btn-danger" data-act="remove">Remove</button>
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
