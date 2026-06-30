import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { createRequire } from "module";

const require = createRequire(import.meta.url);
const monacoEditorPlugin = require("vite-plugin-monaco-editor");

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig({
  base: "./",  // Tauri 打包需要相对路径，确保所有资源路径正确

  plugins: [
    vue(),
    monacoEditorPlugin.default({
      languageWorkers: ["editorWorkerService", "json", "typescript"]
    })
  ],

  optimizeDeps: {
    exclude: ['monaco-editor']
  },

  build: {
    commonjsOptions: {
      include: ['monaco-editor']
    }
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  // 禁用缓存（避免 Vite 预构建缓存导致 monaco-editor 打包产物不一致，
  // 从而在 Tauri 打包后出现 Worker 加载失败 / 语法高亮丢失）
  cacheDir: false,
});
