import { defineConfig } from "vite";
import { resolve } from "path";

export default defineConfig({
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    target: "es2022",
    rollupOptions: {
      input: {
        login: resolve(__dirname, "src/login/index.html"),
        settings: resolve(__dirname, "src/settings/index.html"),
        "quick-capture": resolve(__dirname, "src/quick-capture/index.html"),
      },
    },
  },
});
