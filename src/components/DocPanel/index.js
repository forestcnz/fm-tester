import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import * as monaco from "monaco-editor";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { showToast } from "../../composables/useToast.js";
import { getMonacoTheme } from "../../composables/useTheme.js";
import { renderMarkdown } from "../../utils/markdown.js";

export function useDocPanelSetup(props) {
  const { t } = useI18n();

  // 文档内容
  const docContent = ref("");

  // 文档模式：'view' 展示模式 | 'edit' 编辑模式
  const docMode = ref("view");

  // AI生成状态
  const generating = ref(false);

  // 生成计时（本地计时）
  const generatingTime = ref(0);

  // 最新编辑保存时间
  const lastUpdatedAt = ref(null);

  // 计时器
  let generatingTimer = null;

  // 事件监听器
  let unlistenStart = null;
  let unlistenComplete = null;
  let unlistenError = null;

  // Monaco 编辑器容器
  const docEditorContainer = ref(null);
  let docEditor = null;

  // 渲染后的 Markdown HTML（已通过 DOMPurify 消毒）
  const renderedDocHtml = computed(() => {
    const content = docContent.value || "";
    if (!content) return "";
    return renderMarkdown(content);
  });

  // 开始本地计时
  const startTimer = (initialSeconds = 0) => {
    generatingTime.value = initialSeconds;
    if (generatingTimer) {
      clearInterval(generatingTimer);
    }
    generatingTimer = setInterval(() => {
      generatingTime.value++;
    }, 1000);
  };

  // 停止计时
  const stopTimer = () => {
    if (generatingTimer) {
      clearInterval(generatingTimer);
      generatingTimer = null;
    }
    generatingTime.value = 0;
  };

  // 加载 API 文档
  const loadApiDoc = async () => {
    if (!props.workspaceId || !props.apiId) return;
    try {
      const content = await invoke("get_api_doc", {
        workspaceId: props.workspaceId,
        apiId: props.apiId,
      });
      docContent.value = content || "";
      // 同步到编辑器
      if (docEditor) {
        if (docEditor.getValue() !== docContent.value) {
          docEditor.setValue(docContent.value);
        }
      }
    } catch (e) {
      // 其他异常情况（如网络错误、数据库连接错误等）
      console.warn("加载 API 文档异常:", e);
      docContent.value = "";
    }
  };

  // 加载文档元数据（最新编辑保存时间）
  const loadDocMetadata = async () => {
    if (!props.workspaceId || !props.apiId) return;
    try {
      const metadata = await invoke("get_api_doc_metadata", {
        workspaceId: props.workspaceId,
        apiId: props.apiId,
      });
      lastUpdatedAt.value = metadata.updated_at || null;
    } catch (e) {
      console.error("加载文档元数据失败:", e);
      lastUpdatedAt.value = null;
    }
  };

  // 初始化 Monaco 编辑器
  const initDocEditor = () => {
    if (!docEditorContainer.value) return;

    // 获取当前主题
    const currentTheme =
      localStorage.getItem("fm-tester-theme") || "arctic-ice";

    docEditor = monaco.editor.create(docEditorContainer.value, {
      value: docContent.value || "",
      language: "markdown",
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
      scrollbar: {
        vertical: "auto",
        horizontal: "auto",
        verticalScrollbarSize: 10,
        horizontalScrollbarSize: 10,
      },
      padding: { top: 12, bottom: 12 },
    });

    // 监听内容变化
    docEditor.onDidChangeModelContent(() => {
      docContent.value = docEditor.getValue();
    });
  };

  // 监听主题变化，更新编辑器主题
  const handleThemeChange = (event) => {
    if (docEditor) {
      docEditor.updateOptions({ theme: event.detail.monacoTheme });
    }
  };

  // 切换文档模式
  const toggleDocMode = () => {
    // AI生成中不允许切换到编辑模式
    if (generating.value) return;

    if (docMode.value === "view") {
      docMode.value = "edit";
      // 切换到编辑模式时初始化编辑器
      setTimeout(() => {
        if (docEditorContainer.value) {
          if (!docEditor) {
            initDocEditor();
          } else {
            docEditor.layout();
            if (docEditor.getValue() !== docContent.value) {
              docEditor.setValue(docContent.value);
            }
          }
        }
      }, 50);
    } else {
      docMode.value = "view";
    }
  };

  // 保存文档
  const saveDoc = async () => {
    if (!props.workspaceId || !props.apiId) return;
    try {
      await invoke("save_api_doc", {
        workspaceId: props.workspaceId,
        apiId: props.apiId,
        content: docContent.value,
      });
      // 刷新元数据（最新编辑保存时间）
      await loadDocMetadata();
      // 自动切换到查看模式
      docMode.value = "view";
      showToast(t("toast.saved"), "success");
    } catch (e) {
      console.error("保存 API 文档失败:", e);
      showToast(t("toast.saveFailed"), "error");
    }
  };

  // AI生成文档（调用后端命令）
  const generateDocWithAI = async () => {
    if (!props.workspaceId || !props.apiId || generating.value) return;

    generating.value = true;
    startTimer(0);

    try {
      // 直接调用后端命令，后端会处理所有逻辑并发送事件
      await invoke("generate_api_doc_with_ai", {
        workspaceId: props.workspaceId,
        apiId: props.apiId,
      });
    } catch (e) {
      console.error("AI生成文档失败:", e);
      generating.value = false;
      stopTimer();
      // 如果是"生成已取消"，不显示错误
      if (e !== "生成已取消") {
        showToast(`${t("docPanel.generateFailed")}: ${e}`, "error");
      }
    }
  };

  // 取消后端生成任务
  const cancelBackendGeneration = async () => {
    try {
      await invoke("cancel_doc_generation", {
        apiId: props.apiId,
      });
      generating.value = false;
      stopTimer();
      showToast(t("docPanel.generationCancelled"), "info");
    } catch (e) {
      console.error("取消生成失败:", e);
    }
  };

  // 监听 apiId 变化，加载文档
  watch(
    () => props.apiId,
    async (newId, oldId) => {
      if (newId && newId !== oldId) {
        // 切换 API 时重置显示状态
        docMode.value = "view";
        generating.value = false;
        stopTimer();
        lastUpdatedAt.value = null;

        loadApiDoc();
        loadDocMetadata();

        // 首次调用检查后端是否有正在进行的生成任务
        try {
          const status = await invoke("get_doc_generation_status", {
            apiId: newId,
          });
          // 如果后端正在生成，则显示生成状态并继续计时
          if (status.generating) {
            generating.value = true;
            startTimer(status.elapsed_seconds || 0);
          }
        } catch {
          // 忽略错误
        }
      }
    },
    { immediate: true },
  );

  // 初始化事件监听
  onMounted(async () => {
    // 监听主题变化事件
    window.addEventListener("fm-theme-change", handleThemeChange);

    // 监听生成开始事件
    unlistenStart = await listen("doc-generation-start", (event) => {
      if (event.payload === props.apiId) {
        generating.value = true;
        startTimer(0);
      }
    });

    // 监听生成完成事件
    unlistenComplete = await listen("doc-generation-complete", (event) => {
      generating.value = false;
      stopTimer();
      docContent.value = event.payload || "";
      // 切换到编辑模式
      docMode.value = "edit";
      setTimeout(() => {
        if (docEditorContainer.value) {
          if (!docEditor) {
            initDocEditor();
          } else {
            docEditor.layout();
            docEditor.setValue(docContent.value);
          }
        }
      }, 50);
      showToast(t("docPanel.generateAndSaved"), "success");
      // 刷新元数据
      loadDocMetadata();
    });

    // 监听生成错误事件
    unlistenError = await listen("doc-generation-error", (event) => {
      generating.value = false;
      stopTimer();
      showToast(`${t("docPanel.generateFailed")}: ${event.payload}`, "error");
    });
  });

  // 组件卸载时清理
  onUnmounted(() => {
    // 移除主题变化监听器
    window.removeEventListener("fm-theme-change", handleThemeChange);

    if (docEditor) {
      docEditor.dispose();
    }
    stopTimer();
    // 清理事件监听
    if (unlistenStart) unlistenStart();
    if (unlistenComplete) unlistenComplete();
    if (unlistenError) unlistenError();
  });

  return {
    docContent,
    docMode,
    renderedDocHtml,
    docEditorContainer,
    generating,
    generatingTime,
    lastUpdatedAt,
    toggleDocMode,
    saveDoc,
    generateDocWithAI,
    cancelBackendGeneration,
    loadDocMetadata,
  };
}
