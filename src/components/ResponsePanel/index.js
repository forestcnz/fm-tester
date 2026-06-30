import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { useI18n } from "vue-i18n";
import * as monaco from "monaco-editor";
import { getMonacoTheme } from "../../composables/useTheme.js";

// 语言映射 - Content-Type 到 Monaco 语言
const contentTypeToLanguage = {
  "application/json": "json",
  "application/xml": "xml",
  "text/xml": "xml",
  "text/html": "html",
  "application/xhtml+xml": "html",
  "application/javascript": "javascript",
  "text/javascript": "javascript",
  "text/plain": "plaintext",
  "text/css": "css",
};

// 导出 composable 函数
export function useResponsePanelSetup(props, emit) {
  const { t } = useI18n();

  const tabs = computed(() => {
    const list = [
      { key: "body", name: t("tabs.responseBody") },
      { key: "headers", name: t("tabs.responseHeaders") },
      { key: "tests", name: t("tabs.testResults") },
    ];
    list.push({ key: "timing", name: t("tabs.timing") });
    return list;
  });

  const activeTab = ref("body");
  const editorContainer = ref(null);
  const sseContainer = ref(null);
  let monacoEditor = null;

  // 统一判断是否应该显示 SSE 事件列表
  const shouldShowSSEEvents = computed(() => {
    return sseEventBlocks.value.length > 0;
  });

  // 测试结果统计
  const testStats = computed(() => {
    const results = props.testResults || [];
    const passed = results.filter((r) => r.passed).length;
    const failed = results.filter((r) => !r.passed).length;
    const total = results.length;
    return { passed, failed, total };
  });

  // 保存响应
  const handleSaveResponse = () => {
    emit("save-response");
  };

  const statusClass = computed(() => {
    if (!props.response) return "";
    const status = props.response.status;
    if (status >= 200 && status < 300) return "success";
    if (status >= 300 && status < 400) return "redirect";
    if (status >= 400 && status < 500) return "client-error";
    if (status >= 500) return "server-error";
    return "";
  });

  const formattedBody = computed(() => {
    if (!props.response?.body) return "";

    const isSSE =
      props.response?.headers?.["content-type"]?.includes(
        "text/event-stream",
      ) ||
      props.response?.headers?.["Content-Type"]?.includes("text/event-stream");

    if (isSSE) {
      return props.response.body;
    }

    try {
      return JSON.stringify(JSON.parse(props.response.body), null, 2);
    } catch {
      return props.response.body;
    }
  });

  const sseEventBlocks = computed(() => {
    const events = props.sseEvents || [];
    if (events.length === 0) return [];

    return events.map((ev) => {
      const date = new Date(ev.time);
      const displayTime = isNaN(date.getTime()) ? "" : date.toLocaleString();
      return { time: displayTime, data: ev.data ?? "" };
    });
  });

  // 根据 Content-Type 检测语言
  const detectedLanguage = computed(() => {
    if (!props.response?.headers) return "plaintext";

    const contentType =
      props.response.headers["content-type"] ||
      props.response.headers["Content-Type"] ||
      "";

    if (contentType.includes("text/event-stream")) {
      return "plaintext";
    }

    for (const [pattern, lang] of Object.entries(contentTypeToLanguage)) {
      if (contentType.includes(pattern)) {
        return lang;
      }
    }

    if (props.response?.body) {
      try {
        JSON.parse(props.response.body);
        return "json";
      } catch {
        if (props.response.body.trim().startsWith("<")) {
          return "html";
        }
      }
    }

    return "plaintext";
  });

  const formatSize = (bytes) => {
    if (!bytes) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    let i = 0;
    let size = bytes;
    while (size >= 1024 && i < units.length - 1) {
      size /= 1024;
      i++;
    }
    return `${size.toFixed(2)} ${units[i]}`;
  };

  const formatTime = (ms) => {
    if (!ms) return "0 ms";
    if (ms < 1000) return `${ms} ms`;
    return `${(ms / 1000).toFixed(2)} s`;
  };

  // 请求时间线各阶段（含占比与主题色）
  const timingStages = computed(() => {
    const tg = props.response?.timing;
    if (!tg) return [];
    const total =
      tg.total_ms ||
      tg.dns_ms + tg.connect_ms + tg.tls_ms + tg.ttfb_ms + tg.download_ms ||
      1;
    const stages = [
      { key: "dns", ms: tg.dns_ms, color: "var(--method-post)" },
      { key: "connect", ms: tg.connect_ms, color: "var(--method-put)" },
      { key: "tls", ms: tg.tls_ms, color: "var(--method-get)" },
      { key: "ttfb", ms: tg.ttfb_ms, color: "var(--primary)" },
      { key: "download", ms: tg.download_ms, color: "var(--method-delete)" },
    ];
    return stages.map((s) => ({
      ...s,
      label: t(`timing.${s.key}`),
      percent: Math.round((s.ms / total) * 100),
    }));
  });

  // 初始化 Monaco Editor
  const initMonacoEditor = () => {
    if (!editorContainer.value) return;

    // 获取当前主题
    const currentTheme =
      localStorage.getItem("fm-tester-theme") || "arctic-ice";

    // 创建编辑器实例（只读模式）
    monacoEditor = monaco.editor.create(editorContainer.value, {
      value: formattedBody.value || "",
      language: detectedLanguage.value,
      theme: getMonacoTheme(currentTheme),
      fontSize: 13,
      fontFamily: "Consolas, Monaco, monospace",
      lineNumbers: "on",
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      tabSize: 2,
      wordWrap: "on",
      // 折叠配置
      folding: true,
      foldingStrategy: "indentation",
      showFoldingControls: "always",
      unfoldOnClickAfterEndOfLine: true,
      // 左侧区域配置（显示折叠图标）
      glyphMargin: true,
      // 只读配置
      readOnly: true,
      domReadOnly: true,
      renderWhitespace: "selection",
      scrollbar: {
        vertical: "auto",
        horizontal: "auto",
        verticalScrollbarSize: 10,
        horizontalScrollbarSize: 10,
      },
      padding: { top: 12, bottom: 12 },
      bracketPairColorization: { enabled: true },
    });
  };

  // 更新编辑器内容和语言
  const updateEditorContent = () => {
    if (!monacoEditor) return;

    const newValue = formattedBody.value || "";
    const currentValue = monacoEditor.getValue();

    if (newValue !== currentValue) {
      monacoEditor.setValue(newValue);
    }

    const model = monacoEditor.getModel();
    if (model) {
      monaco.editor.setModelLanguage(model, detectedLanguage.value);
    }
  };

  // 监听主题变化，更新编辑器主题
  const handleThemeChange = (event) => {
    if (monacoEditor) {
      monacoEditor.updateOptions({ theme: event.detail.monacoTheme });
    }
  };

  // SSE 容器滚动到底部
  const scrollSSEToBottom = () => {
    if (!sseContainer.value) return;
    sseContainer.value.scrollTop = sseContainer.value.scrollHeight;
  };

  // SSE 模式专用 watcher：仅监听 body 字符串变化（高频触发滚动）
  // 此处不使用 deep watch，避免每次 SSE chunk 都做完整响应对象的深度 diff
  watch(
    () => props.sseEvents?.length,
    () => {
      if (!props.response || activeTab.value !== "body") return;
      if (shouldShowSSEEvents.value) {
        requestAnimationFrame(() => {
          setTimeout(() => scrollSSEToBottom(), 0);
        });
      }
    },
  );

  // 普通响应 / 编辑器初始化 / 切换显示模式
  // 不再使用 deep：response 切换 API 时是整体引用变化，
  // 普通响应内的字段变化（status/headers 等）不需要重置编辑器
  watch(
    [() => props.response, activeTab, shouldShowSSEEvents],
    ([newResponse, newTab, showSSE], [_oldResponse, _oldTab, oldShowSSE]) => {
      if (!newResponse || newTab !== "body") {
        return;
      }

      const wasSSE = oldShowSSE;
      const isSSE = showSSE;

      // SSE → 普通响应：复用已存在的编辑器
      if (wasSSE && !isSSE) {
        nextTick(() => {
          if (monacoEditor) {
            updateEditorContent();
            monacoEditor.layout();
          } else if (editorContainer.value) {
            initMonacoEditor();
          }
        });
      }
      // 普通响应内容更新
      else if (!isSSE) {
        nextTick(() => {
          if (editorContainer.value) {
            if (!monacoEditor) {
              initMonacoEditor();
            } else {
              updateEditorContent();
              monacoEditor.layout();
            }
          }
        });
      }
      // SSE 事件更新由上方专用 watcher 处理（无需在此 deep 比较）
    },
  );

  // 组件挂载时添加主题监听
  onMounted(() => {
    window.addEventListener("fm-theme-change", handleThemeChange);
  });

  // 组件卸载时销毁编辑器
  onUnmounted(() => {
    // 移除主题变化监听器
    window.removeEventListener("fm-theme-change", handleThemeChange);

    if (monacoEditor) {
      monacoEditor.dispose();
      monacoEditor = null;
    }
  });

  return {
    tabs,
    activeTab,
    statusClass,
    timingStages,
    formattedBody,
    sseEventBlocks,
    shouldShowSSEEvents,
    detectedLanguage,
    formatSize,
    formatTime,
    editorContainer,
    sseContainer,
    testStats,
    handleSaveResponse,
  };
}
