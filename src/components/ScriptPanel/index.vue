<script setup>
import { ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import * as monaco from "monaco-editor";
import { showToast } from "../../composables/useToast";
import { getMonacoTheme } from "../../composables/useTheme.js";

const { t } = useI18n();

const props = defineProps({
  request: Object,
  targetType: { type: String, default: "" },
  targetName: { type: String, default: "" },
});

const emit = defineEmits(["update:request", "save"]);

const scriptType = ref("pre");
const editorContainer = ref(null);
let editor = null;
let completionProvider = null;

// AI 优化相关状态
const aiOptimizing = ref(false);
let streamUnlisten = null;
const getFmCompletions = (isPostScript) => {
  const completions = [
    // environment
    {
      label: "fm.environment.get",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.environment.get("${1:key}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "获取环境变量",
    },
    {
      label: "fm.environment.set",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.environment.set("${1:key}", "${2:value}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "设置环境变量",
    },
    {
      label: "fm.environment.remove",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.environment.remove("${1:key}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "删除环境变量",
    },
    {
      label: "fm.environment.getAll",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: "fm.environment.getAll()",
      documentation: "获取所有环境变量",
    },
    // collection
    {
      label: "fm.collection.get",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.collection.get("${1:key}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "获取集合变量",
    },
    {
      label: "fm.collection.set",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.collection.set("${1:key}", "${2:value}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "设置集合变量",
    },
    {
      label: "fm.collection.remove",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.collection.remove("${1:key}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "删除集合变量",
    },
    {
      label: "fm.collection.getAll",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: "fm.collection.getAll()",
      documentation: "获取所有集合变量",
    },
    // request
    {
      label: "fm.request.getUrl",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: "fm.request.getUrl()",
      documentation: "获取请求 URL",
    },
    {
      label: "fm.request.setUrl",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.request.setUrl("${1:url}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "设置请求 URL",
    },
    {
      label: "fm.request.getBaseUrl",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: "fm.request.getBaseUrl()",
      documentation: "获取 baseUrl",
    },
    {
      label: "fm.request.setBaseUrl",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.request.setBaseUrl("${1:baseUrl}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "设置 baseUrl",
    },
    {
      label: "fm.request.getPath",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: "fm.request.getPath()",
      documentation: "获取请求路径",
    },
    {
      label: "fm.request.setPath",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.request.setPath("${1:path}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "设置请求路径",
    },
    {
      label: "fm.request.getMethod",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: "fm.request.getMethod()",
      documentation: "获取请求方法",
    },
    {
      label: "fm.request.setMethod",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.request.setMethod("${1:method}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "设置请求方法",
    },
    {
      label: "fm.request.getHeader",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.request.getHeader("${1:key}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "获取请求头",
    },
    {
      label: "fm.request.setHeader",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.request.setHeader("${1:key}", "${2:value}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "设置请求头",
    },
    {
      label: "fm.request.removeHeader",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.request.removeHeader("${1:key}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "移除请求头",
    },
    {
      label: "fm.request.getHeaders",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: "fm.request.getHeaders()",
      documentation: "获取所有请求头",
    },
    // params
    {
      label: "fm.request.getParam",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.request.getParam("${1:key}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "获取 Query 参数",
    },
    {
      label: "fm.request.setParam",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.request.setParam("${1:key}", "${2:value}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "设置 Query 参数",
    },
    {
      label: "fm.request.removeParam",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.request.removeParam("${1:key}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "移除 Query 参数",
    },
    {
      label: "fm.request.getParams",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: "fm.request.getParams()",
      documentation: "获取所有启用的 Query 参数（返回 JSON 字符串）",
    },
    {
      label: "fm.request.getBody",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: "fm.request.getBody()",
      documentation: "获取请求体",
    },
    {
      label: "fm.request.setBody",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.request.setBody("${1:body}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "设置请求体",
    },
    // log
    {
      label: "fm.log",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.log("${1:message}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "输出日志到 Console",
    },
    // assert
    {
      label: "fm.assert",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: 'fm.assert(${1:condition}, "${2:message}")',
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "断言检查",
    },
    // sleep
    {
      label: "fm.sleep",
      kind: monaco.languages.CompletionItemKind.Method,
      insertText: "fm.sleep(${1:ms})",
      insertTextRules:
        monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
      documentation: "异步等待（毫秒）",
    },
  ];

  // 后置脚本额外提供 response API
  if (isPostScript) {
    completions.push(
      {
        label: "fm.response.getStatus",
        kind: monaco.languages.CompletionItemKind.Method,
        insertText: "fm.response.getStatus()",
        documentation: "获取响应状态码",
      },
      {
        label: "fm.response.getStatusText",
        kind: monaco.languages.CompletionItemKind.Method,
        insertText: "fm.response.getStatusText()",
        documentation: "获取响应状态文本",
      },
      {
        label: "fm.response.getHeader",
        kind: monaco.languages.CompletionItemKind.Method,
        insertText: 'fm.response.getHeader("${1:key}")',
        insertTextRules:
          monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
        documentation: "获取响应头",
      },
      {
        label: "fm.response.getHeaders",
        kind: monaco.languages.CompletionItemKind.Method,
        insertText: "fm.response.getHeaders()",
        documentation: "获取所有响应头",
      },
      {
        label: "fm.response.getBody",
        kind: monaco.languages.CompletionItemKind.Method,
        insertText: "fm.response.getBody()",
        documentation: "获取响应体（字符串）",
      },
      {
        label: "fm.response.getJson",
        kind: monaco.languages.CompletionItemKind.Method,
        insertText: "fm.response.getJson()",
        documentation: "获取响应体（JSON 对象）",
      },
      {
        label: "fm.response.getTime",
        kind: monaco.languages.CompletionItemKind.Method,
        insertText: "fm.response.getTime()",
        documentation: "获取响应时间（ms）",
      },
      {
        label: "fm.response.getSize",
        kind: monaco.languages.CompletionItemKind.Method,
        insertText: "fm.response.getSize()",
        documentation: "获取响应大小（bytes）",
      },
    );
  }

  return completions;
};

// 注册自动补全
const registerCompletionProvider = (isPostScript) => {
  // 先注销旧的
  if (completionProvider) {
    completionProvider.dispose();
  }

  completionProvider = monaco.languages.registerCompletionItemProvider(
    "javascript",
    {
      provideCompletionItems: (model, position) => {
        const textUntilPosition = model.getValueInRange({
          startLineNumber: 1,
          startColumn: 1,
          endLineNumber: position.lineNumber,
          endColumn: position.column,
        });

        // 检查是否在 fm. 后面
        const match = textUntilPosition.match(/fm\.(\w*)$/);
        if (!match) {
          return { suggestions: [] };
        }

        const word = match[1];
        const suggestions = getFmCompletions(isPostScript)
          .filter((item) => item.label.startsWith(`fm.${word}`) || word === "")
          .map((item) => ({
            ...item,
            range: {
              startLineNumber: position.lineNumber,
              startColumn: position.column - word.length - 3, // -3 for "fm."
              endLineNumber: position.lineNumber,
              endColumn: position.column,
            },
          }));

        return { suggestions };
      },
    },
  );
};

// 初始化编辑器
const initEditor = () => {
  if (!editorContainer.value || editor) return;

  const value =
    scriptType.value === "pre"
      ? props.request?.preScript || ""
      : props.request?.postScript || "";

  // 获取当前主题
  const currentTheme = localStorage.getItem("fm-tester-theme") || "paper";

  editor = monaco.editor.create(editorContainer.value, {
    value,
    language: "javascript",
    theme: getMonacoTheme(currentTheme),
    fontSize: 13,
    fontFamily: "IBM Plex Mono, Consolas, monospace",
    lineNumbers: "on",
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    automaticLayout: true,
    tabSize: 2,
    wordWrap: "on",
  });

  // 注册自动补全
  registerCompletionProvider(scriptType.value === "post");

  // 注册 Ctrl+S 保存快捷键
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
    saveScript();
  });
};

// 监听主题变化，更新编辑器主题
const handleThemeChange = (event) => {
  if (editor) {
    editor.updateOptions({ theme: event.detail.monacoTheme });
  }
};

// 监听 request 变化，切换接口时更新编辑器内容
watch(
  () => props.request,
  (newRequest) => {
    if (editor && newRequest) {
      const newValue =
        scriptType.value === "pre"
          ? newRequest.preScript || ""
          : newRequest.postScript || "";
      editor.setValue(newValue);
    }
  },
  { deep: true },
);

// 切换脚本类型时加载新内容
watch(scriptType, (newType) => {
  if (editor) {
    const newValue =
      newType === "pre"
        ? props.request?.preScript || ""
        : props.request?.postScript || "";
    editor.setValue(newValue);
    editor.layout();
    // 更新自动补全
    registerCompletionProvider(newType === "post");
  }
});

// 保存脚本
const saveScript = () => {
  if (!editor) return;
  const content = editor.getValue();
  emit("update:request", {
    ...props.request,
    [scriptType.value === "pre" ? "preScript" : "postScript"]: content,
  });
  emit("save", scriptType.value);
};

// AI 优化脚本
const optimizeWithAi = async () => {
  if (!editor) return;

  aiOptimizing.value = true;
  const currentContent = editor.getValue();
  let streamedContent = "";

  try {
    // 监听流式响应
    streamUnlisten = await listen("ai-chat-stream", (event) => {
      if (editor) {
        streamedContent += event.payload;
        editor.setValue(streamedContent);
      }
    });

    // 蒯用后端 AI 优化（不传 api key，后端自动从 settings 解密）
    const result = await invoke("optimize_script_ai", {
      scriptContent: currentContent,
      scriptType: scriptType.value,
    });

    // 设置最终结果
    editor.setValue(result);

    // 提示成功
    showToast(t("script.aiOptimizeSuccess"), "success");
  } catch (e) {
    console.error("AI optimize error:", e);
    // 恢复原始内容
    editor.setValue(currentContent);
    showToast(t("script.aiOptimizeFailed") + ": " + e, "error");
  } finally {
    aiOptimizing.value = false;
    if (streamUnlisten) {
      streamUnlisten();
      streamUnlisten = null;
    }
  }
};

onMounted(() => {
  // 监听主题变化事件
  window.addEventListener("fm-theme-change", handleThemeChange);

  nextTick(() => {
    setTimeout(() => {
      // 确保容器有尺寸
      if (editorContainer.value && editorContainer.value.offsetHeight > 0) {
        initEditor();
      } else {
        // 如果没有尺寸，再等一下
        setTimeout(() => initEditor(), 200);
      }
    }, 50);
  });
});

onUnmounted(() => {
  // 移除主题变化监听器
  window.removeEventListener("fm-theme-change", handleThemeChange);

  editor?.dispose();
  completionProvider?.dispose();
  if (streamUnlisten) {
    streamUnlisten();
  }
});
</script>

<template>
  <div class="script-panel">
    <div class="script-tabs">
      <div
        class="script-btn"
        :class="{ active: scriptType === 'pre' }"
        @click="scriptType = 'pre'"
      >
        {{ t("script.preScript") }}
      </div>
      <div
        class="script-btn"
        :class="{ active: scriptType === 'post' }"
        @click="scriptType = 'post'"
      >
        {{ t("script.postScript") }}
      </div>
    </div>

    <div class="editor-wrapper">
      <div class="editor-header">
        <button class="save-btn" @click="saveScript">
          {{ t("common.save") }}
        </button>
        <button
          class="ai-btn"
          :disabled="aiOptimizing"
          @click="optimizeWithAi"
          :title="t('script.aiOptimizeDesc')"
        >
          {{ aiOptimizing ? t("script.aiOptimizing") : t("script.aiOptimize") }}
        </button>
      </div>
      <div ref="editorContainer" class="editor-container"></div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
