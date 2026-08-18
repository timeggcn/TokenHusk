import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 约定：固定 5173 端口、禁止清屏（保证 Tauri CLI 能读到启动日志）。
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  envPrefix: ["VITE_", "TAURI_"],
});
