import { ref, computed, watch, onMounted, onUnmounted } from "vue";
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

// Body 类型到 Monaco 语言映射
const bodyTypeToLanguage = {
  json: "json",
  xml: "xml",
  html: "html",
  text: "plaintext",
  javascript: "javascript",
  raw: "plaintext",
};

export function useHistoryDetailSetup(props) {
  // 请求标签页
  const requestTab = ref("params");
  // 响应标签页
  const responseTab = ref("body");

  // Monaco Editor 容器引用
  const requestEditorContainer = ref(null);
  const responseEditorContainer = ref(null);
  let requestMonacoEditor = null;
  let responseMonacoEditor = null;

  // 格式化请求体
  const formattedRequestBody = computed(() => {
    if (!props.entry?.body) return "";

    // 判断是否为 JSON 类型
    let isJson = false;

    // 先从请求头判断
    if (props.entry?.headers) {
      const contentTypeHeader = props.entry.headers.find(
        (h) => h.key.toLowerCase() === "content-type",
      );
      if (contentTypeHeader?.value?.includes("application/json")) {
        isJson = true;
      }
    }

    // 再从 body_type 判断
    if (!isJson && props.entry?.body_type?.toLowerCase() === "json") {
      isJson = true;
    }

    // JSON 类型才格式化
    if (isJson) {
      try {
        return JSON.stringify(JSON.parse(props.entry.body), null, 2);
      } catch {
        return props.entry.body;
      }
    }

    // 其他类型保持原始内容
    return props.entry.body;
  });

  // 检测请求体语言
  const detectedRequestLanguage = computed(() => {
    // 先从请求头中获取 Content-Type
    if (props.entry?.headers) {
      const contentTypeHeader = props.entry.headers.find(
        (h) => h.key.toLowerCase() === "content-type",
      );
      if (contentTypeHeader?.value) {
        for (const [pattern, lang] of Object.entries(contentTypeToLanguage)) {
          if (contentTypeHeader.value.includes(pattern)) {
            return lang;
          }
        }
      }
    }

    // 根据 body_type 判断
    if (props.entry?.body_type) {
      const lang = bodyTypeToLanguage[props.entry.body_type.toLowerCase()];
      if (lang) return lang;
    }

    // 尝试自动检测
    if (props.entry?.body) {
      try {
        JSON.parse(props.entry.body);
        return "json";
      } catch {
        if (props.entry.body.trim().startsWith("<")) {
          return "xml";
        }
      }
    }

    return "plaintext";
  });

  // 格式化响应体
  const formattedResponseBody = computed(() => {
    if (!props.entry?.response_body) return "";

    // 只有 JSON 类型才格式化
    const contentType =
      props.entry?.response_headers?.["content-type"] ||
      props.entry?.response_headers?.["Content-Type"] ||
      "";

    if (contentType.includes("application/json")) {
      try {
        return JSON.stringify(JSON.parse(props.entry.response_body), null, 2);
      } catch {
        return props.entry.response_body;
      }
    }

    // 其他类型保持原始内容
    return props.entry.response_body;
  });

  // 检测响应体语言
  const detectedResponseLanguage = computed(() => {
    if (!props.entry?.response_headers) return "plaintext";

    const contentType =
      props.entry.response_headers["content-type"] ||
      props.entry.response_headers["Content-Type"] ||
      "";

    // 遍历映射表查找匹配的语言
    for (const [pattern, lang] of Object.entries(contentTypeToLanguage)) {
      if (contentType.includes(pattern)) {
        return lang;
      }
    }

    // 尝试自动检测
    if (props.entry?.response_body) {
      try {
        JSON.parse(props.entry.response_body);
        return "json";
      } catch {
        if (props.entry.response_body.trim().startsWith("<")) {
          return "html";
        }
      }
    }

    return "plaintext";
  });

  // 格式化响应时间
  const formatResponseTime = (time) => {
    if (!time) return "0 ms";
    if (time < 1000) return `${time} ms`;
    return `${(time / 1000).toFixed(2)} s`;
  };

  // 格式化响应大小
  const formatSize = (size) => {
    if (!size) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    let i = 0;
    let s = size;
    while (s >= 1024 && i < units.length - 1) {
      s /= 1024;
      i++;
    }
    return `${s.toFixed(2)} ${units[i]}`;
  };

  // 获取状态码样式类
  const getStatusClass = (status) => {
    if (!status) return "";
    const code = parseInt(status);
    if (code >= 200 && code < 300) return "success";
    if (code >= 300 && code < 400) return "redirect";
    if (code >= 400 && code < 500) return "client-error";
    if (code >= 500) return "server-error";
    return "";
  };

  // 获取方法样式类
  const getMethodClass = (method) => method?.toLowerCase() || "";

  // 格式化时间戳
  const formatTime = (timestamp) => {
    if (!timestamp) return "";
    const date = new Date(timestamp);
    return date.toLocaleString("zh-CN");
  };

  // 初始化请求体 Monaco Editor
  const initRequestEditor = () => {
    if (!requestEditorContainer.value) return;

    // 获取当前主题
    const currentTheme =
      localStorage.getItem("fm-tester-theme") || "arctic-ice";

    requestMonacoEditor = monaco.editor.create(requestEditorContainer.value, {
      value: formattedRequestBody.value || "",
      language: detectedRequestLanguage.value,
      theme: getMonacoTheme(currentTheme),
      fontSize: 13,
      fontFamily: "Consolas, Monaco, monospace",
      lineNumbers: "on",
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      tabSize: 2,
      wordWrap: "on",
      folding: true,
      foldingStrategy: "indentation",
      showFoldingControls: "always",
      glyphMargin: true,
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

  // 更新请求体编辑器内容
  const updateRequestEditorContent = () => {
    if (!requestMonacoEditor) return;

    const newValue = formattedRequestBody.value || "";
    const currentValue = requestMonacoEditor.getValue();

    if (newValue !== currentValue) {
      requestMonacoEditor.setValue(newValue);
    }

    const model = requestMonacoEditor.getModel();
    if (model) {
      monaco.editor.setModelLanguage(model, detectedRequestLanguage.value);
    }
  };

  // 初始化响应体 Monaco Editor
  const initResponseEditor = () => {
    if (!responseEditorContainer.value) return;

    // 获取当前主题
    const currentTheme =
      localStorage.getItem("fm-tester-theme") || "arctic-ice";

    responseMonacoEditor = monaco.editor.create(responseEditorContainer.value, {
      value: formattedResponseBody.value || "",
      language: detectedResponseLanguage.value,
      theme: getMonacoTheme(currentTheme),
      fontSize: 13,
      fontFamily: "Consolas, Monaco, monospace",
      lineNumbers: "on",
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      tabSize: 2,
      wordWrap: "on",
      folding: true,
      foldingStrategy: "indentation",
      showFoldingControls: "always",
      glyphMargin: true,
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

  // 更新响应体编辑器内容
  const updateResponseEditorContent = () => {
    if (!responseMonacoEditor) return;

    const newValue = formattedResponseBody.value || "";
    const currentValue = responseMonacoEditor.getValue();

    if (newValue !== currentValue) {
      responseMonacoEditor.setValue(newValue);
    }

    const model = responseMonacoEditor.getModel();
    if (model) {
      monaco.editor.setModelLanguage(model, detectedResponseLanguage.value);
    }
  };

  // 监听主题变化，更新编辑器主题
  const handleThemeChange = (event) => {
    if (requestMonacoEditor) {
      requestMonacoEditor.updateOptions({ theme: event.detail.monacoTheme });
    }
    if (responseMonacoEditor) {
      responseMonacoEditor.updateOptions({ theme: event.detail.monacoTheme });
    }
  };

  // 监听 entry 变化
  watch(
    () => props.entry,
    (newEntry) => {
      if (!newEntry) return;

      // 初始化/更新请求体编辑器
      if (requestTab.value === "body") {
        setTimeout(() => {
          if (requestEditorContainer.value) {
            if (!requestMonacoEditor) {
              initRequestEditor();
            } else {
              updateRequestEditorContent();
              requestMonacoEditor.layout();
            }
          }
        }, 50);
      }

      // 初始化/更新响应体编辑器
      if (responseTab.value === "body") {
        setTimeout(() => {
          if (responseEditorContainer.value) {
            if (!responseMonacoEditor) {
              initResponseEditor();
            } else {
              updateResponseEditorContent();
              responseMonacoEditor.layout();
            }
          }
        }, 50);
      }
    },
    { deep: true },
  );

  // 监听 requestTab 变化
  watch(requestTab, (newTab) => {
    if (newTab === "body" && props.entry) {
      setTimeout(() => {
        if (requestEditorContainer.value) {
          if (!requestMonacoEditor) {
            initRequestEditor();
          } else {
            requestMonacoEditor.layout();
            updateRequestEditorContent();
          }
        }
      }, 50);
    }
  });

  // 监听 responseTab 变化
  watch(responseTab, (newTab) => {
    if (newTab === "body" && props.entry) {
      setTimeout(() => {
        if (responseEditorContainer.value) {
          if (!responseMonacoEditor) {
            initResponseEditor();
          } else {
            responseMonacoEditor.layout();
            updateResponseEditorContent();
          }
        }
      }, 50);
    }
  });

  // 组件挂载时添加主题监听
  onMounted(() => {
    window.addEventListener("fm-theme-change", handleThemeChange);
  });

  // 组件卸载时销毁编辑器
  onUnmounted(() => {
    // 移除主题变化监听器
    window.removeEventListener("fm-theme-change", handleThemeChange);

    if (requestMonacoEditor) {
      requestMonacoEditor.dispose();
      requestMonacoEditor = null;
    }
    if (responseMonacoEditor) {
      responseMonacoEditor.dispose();
      responseMonacoEditor = null;
    }
  });

  return {
    requestTab,
    responseTab,
    formattedRequestBody,
    detectedRequestLanguage,
    formattedResponseBody,
    detectedResponseLanguage,
    formatResponseTime,
    formatSize,
    getStatusClass,
    getMethodClass,
    formatTime,
    requestEditorContainer,
    responseEditorContainer,
  };
}
