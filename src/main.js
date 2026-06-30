import { createApp } from "vue";
import App from "./App.vue";
import i18n from "./i18n";
import {
  RecycleScroller,
  DynamicScroller,
  DynamicScrollerItem,
} from "vue-virtual-scroller";
import "vue-virtual-scroller/dist/vue-virtual-scroller.css";
import "./styles/themes.css"; // 引入主题系统（墨砚 Inkwell 设计 token）

// 墨砚字体（@fontsource 本地打包，离线可用）
import "@fontsource-variable/manrope/index.css";
import "@fontsource-variable/fraunces/index.css";
import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/ibm-plex-mono/500.css";
import "@fontsource/ibm-plex-mono/600.css";
import "./styles/inkwell.css"; // 墨砚通用原语（.btn/.field/.card/.chip/.kv-table/.lrow/.empty）
import { useMonacoTheme } from "./composables/useMonacoTheme.js";

// 配置 Monaco Editor Worker
// Tauri 打包后启用 CSP，worker-src 'self' 无法匹配自定义协议（https://tauri.localhost），
// 直接 new Worker(url) 会被 CSP 拒绝。通过 blob URL 创建 Worker 绕过此限制：
// blob Worker 内部用 importScripts 加载实际脚本（同源，受 default-src 'self' 允许）。
window.MonacoEnvironment = {
  getWorker: function (workerId, label) {
    const workerPath = "./monacoeditorwork/";
    let workerFile;

    if (label === "json") {
      workerFile = "json.worker.bundle.js";
    } else if (label === "typescript" || label === "javascript") {
      workerFile = "ts.worker.bundle.js";
    } else {
      workerFile = "editor.worker.bundle.js";
    }

    const fullPath = new URL(workerPath + workerFile, window.location.href)
      .href;
    const blob = new Blob(['importScripts("' + fullPath + '");'], {
      type: "application/javascript",
    });
    return new Worker(URL.createObjectURL(blob), { type: "classic" });
  },
};

// 预注册所有 Monaco Editor 自定义主题
// 这样任何组件创建编辑器时都可以直接使用这些主题
const { registerMonacoThemes } = useMonacoTheme();
registerMonacoThemes();

const app = createApp(App);

// 注册虚拟滚动组件
app.component("RecycleScroller", RecycleScroller);
app.component("DynamicScroller", DynamicScroller);
app.component("DynamicScrollerItem", DynamicScrollerItem);

app.use(i18n);
app.mount("#app");
window.removeSplash?.();
