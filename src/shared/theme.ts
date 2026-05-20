import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type Theme = "system" | "dark" | "light";

function resolve(theme: Theme): "dark" | "light" {
  if (theme === "system") {
    return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
  }
  return theme;
}

function apply(theme: Theme) {
  document.documentElement.setAttribute("data-theme", resolve(theme));
  document.documentElement.dataset.themePref = theme;
}

let current: Theme = "system";

window.matchMedia("(prefers-color-scheme: light)").addEventListener("change", () => {
  if (current === "system") apply(current);
});

export async function bootstrapTheme() {
  try {
    current = (await invoke<string>("get_theme")) as Theme;
  } catch {
    current = "system";
  }
  apply(current);
  await listen<string>("theme:changed", (ev) => {
    current = (ev.payload as Theme) ?? "system";
    apply(current);
  });
}

export async function setTheme(theme: Theme) {
  current = theme;
  apply(theme);
  await invoke("set_theme", { theme });
}

export function currentTheme(): Theme {
  return current;
}
