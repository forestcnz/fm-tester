<script setup>
import { ref, watch, nextTick, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import * as monaco from "monaco-editor";
import Icon from "../Icon/index.vue";
import { useDialogEscape } from "../../composables/useDialogStack.js";
import { getMonacoTheme } from "../../composables/useTheme.js";

const props = defineProps({
  visible: Boolean,
  logs: Array,
});

const emit = defineEmits(["close", "clear"]);

const { t } = useI18n();

// ESC 键关闭
useDialogEscape(
  () => props.visible,
  () => emit("close"),
);

// 面板高度
const panelHeight = ref(200);
const minHeight = 100;
const maxHeight = 400;
const isDragging = ref(false);

// 展开的日志索引
const expandedIndex = ref(-1);

// Monaco Editor 引用
const editorContainers = ref({});
let monacoEditors = {};

// 监听主题变化，更新所有展开的编辑器主题
const handleThemeChange = (event) => {
  for (const key in monacoEditors) {
    if (monacoEditors[key]) {
      monacoEditors[key].updateOptions({ theme: event.detail.monacoTheme });
    }
  }
};

// 拖拽逻辑
const startDrag = (e) => {
  isDragging.value = true;
  document.addEventListener("mousemove", onDrag);
  document.addEventListener("mouseup", stopDrag);
  e.preventDefault();
};

const onDrag = (e) => {
  if (!isDragging.value) return;
  const newHeight = panelHeight.value - e.movementY;
  panelHeight.value = Math.min(Math.max(minHeight, newHeight), maxHeight);

  // 拖动时实时更新编辑器高度
  if (expandedIndex.value >= 0) {
    updateEditorHeight(expandedIndex.value);
  }
};

const stopDrag = () => {
  isDragging.value = false;
  document.removeEventListener("mousemove", onDrag);
  document.removeEventListener("mouseup", stopDrag);
};

// 解析消息
const parseMessage = (message) => {
  try {
    const parsed = JSON.parse(message);
    if (typeof parsed === "object" && parsed !== null) {
      return { type: "object", data: parsed };
    }
  } catch { /* ignore */ }
  return { type: "text", data: message };
};

// 深度解析 JSON（解析嵌套的 JSON 字符串）
const deepParseJson = (obj) => {
  if (obj === null || obj === undefined) return obj;

  if (typeof obj === "string") {
    try {
      const parsed = JSON.parse(obj);
      if (typeof parsed === "object") {
        return deepParseJson(parsed);
      }
    } catch { /* ignore */ }
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map((item) => deepParseJson(item));
  }

  if (typeof obj === "object") {
    const result = {};
    for (const [key, value] of Object.entries(obj)) {
      result[key] = deepParseJson(value);
    }
    return result;
  }

  return obj;
};

// 获取摘要
const getSummary = (log) => {
  const parsed = parseMessage(log.message);
  if (parsed.type === "object") {
    return parsed.data.message || JSON.stringify(parsed.data);
  }
  return log.message;
};

// 切换展开
const toggleExpand = async (index) => {
  if (expandedIndex.value === index) {
    expandedIndex.value = -1;
  } else {
    expandedIndex.value = index;
    await nextTick();
    initEditor(index);
  }
};

// 初始化 Monaco Editor
const initEditor = (index) => {
  const container = editorContainers.value[index];
  if (!container) return;

  // 销毁旧的编辑器
  if (monacoEditors[index]) {
    monacoEditors[index].dispose();
    delete monacoEditors[index];
  }

  const log = props.logs[index];
  const parsed = parseMessage(log.message);

  let content;
  let language = "plaintext";

  if (parsed.type === "object") {
    // 只显示 data 字段的内容
    const dataContent = parsed.data.data || parsed.data;
    const deepParsed = deepParseJson(dataContent);
    content = JSON.stringify(deepParsed, null, 2);
    language = "json";
  } else {
    content = log.message;
  }

  // 获取当前主题
  const currentTheme = localStorage.getItem("fm-tester-theme") || "arctic-ice";

  monacoEditors[index] = monaco.editor.create(container, {
    value: content,
    language: language,
    theme: getMonacoTheme(currentTheme),
    fontSize: 12,
    fontFamily: "Consolas, Monaco, monospace",
    lineNumbers: "off",
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
    scrollbar: {
      vertical: "hidden",
      horizontal: "auto",
      verticalScrollbarSize: 0,
      horizontalScrollbarSize: 8,
    },
    padding: { top: 8, bottom: 8 },
  });

  // 根据内容自适应高度
  updateEditorHeight(index);
};

// 更新编辑器高度 - 根据面板剩余空间计算
const updateEditorHeight = (index) => {
  const editor = monacoEditors[index];
  const container = editorContainers.value[index];
  if (!editor || !container) return;

  const panel = container.closest(".console-panel");
  const content = container.closest(".panel-content");
  if (!panel || !content) return;

  // 获取面板总高度
  const panelHeight = parseInt(panel.style.height) || 200;

  // 获取所有日志摘要的高度
  const logItems = content.querySelectorAll(".log-item");
  let totalSummaryHeight = 0;
  logItems.forEach((item) => {
    totalSummaryHeight += item.offsetHeight || 30;
  });

  // 计算剩余可用高度 = 面板高度 - header(28px) - 所有摘要高度
  const headerHeight = 28;
  const availableHeight = panelHeight - headerHeight - totalSummaryHeight;

  // 设置最小高度 60px
  const finalHeight = Math.max(availableHeight, 60);

  // 设置容器高度
  container.style.height = finalHeight + "px";

  // 重新布局编辑器
  setTimeout(() => {
    editor.layout();
  }, 0);
};

// 清理编辑器
onMounted(() => {
  // 监听主题变化事件
  window.addEventListener("fm-theme-change", handleThemeChange);
});

onUnmounted(() => {
  // 移除主题变化监听器
  window.removeEventListener("fm-theme-change", handleThemeChange);

  for (const key in monacoEditors) {
    monacoEditors[key]?.dispose();
  }
});

// 监听可见性
watch(
  () => props.visible,
  (visible) => {
    if (!visible) {
      expandedIndex.value = -1;
      for (const key in monacoEditors) {
        monacoEditors[key]?.dispose();
        delete monacoEditors[key];
      }
    }
  },
);

// 监听日志变化
watch(
  () => props.logs,
  () => {
    expandedIndex.value = -1;
    for (const key in monacoEditors) {
      monacoEditors[key]?.dispose();
      delete monacoEditors[key];
    }
  },
);
</script>

<template>
  <div
    class="console-panel"
    v-if="visible"
    :style="{ height: panelHeight + 'px' }"
  >
    <div
      class="resize-handle"
      :class="{ dragging: isDragging }"
      @mousedown="startDrag"
    ></div>

    <div class="panel-header">
      <span class="header-title">{{ t("panels.console") }}</span>
      <div class="header-actions">
        <button class="clear-btn" @click="$emit('clear')">
          {{ t("common.clear") }}
        </button>
        <button class="close-btn" @click="$emit('close')">×</button>
      </div>
    </div>

    <div class="panel-content">
      <div v-if="logs.length === 0" class="empty-state">
        <span>{{ t("empty.noLogs") }}</span>
      </div>
      <div v-else class="log-list">
        <div class="log-item-wrapper" v-for="(log, index) in logs" :key="index">
          <!-- 摘要行 -->
          <div
            class="log-item"
            :class="{ expanded: expandedIndex === index }"
            @click="toggleExpand(index)"
          >
            <span
              class="expand-arrow"
              :class="{ expanded: expandedIndex === index }"
            >
              <Icon name="arrow-right" :size="12" />
            </span>
            <span class="log-time">{{ log.time }}</span>
            <span class="log-type" :class="log.type">{{
              log.type.toUpperCase()
            }}</span>
            <span class="log-summary">{{ getSummary(log) }}</span>
          </div>

          <!-- 详情内容 -->
          <div class="log-detail" v-if="expandedIndex === index">
            <div
              class="detail-container"
              :ref="(el) => (editorContainers[index] = el)"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
