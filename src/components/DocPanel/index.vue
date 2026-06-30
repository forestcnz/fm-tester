<script setup>
import { useI18n } from "vue-i18n";
import { useDocPanelSetup } from "./index.js";
import Icon from "../Icon/index.vue";

const { t } = useI18n();

const props = defineProps({
  workspaceId: {
    type: String,
    default: "",
  },
  apiId: {
    type: String,
    default: "",
  },
  apiData: {
    type: Object,
    default: null,
  },
});

const {
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
} = useDocPanelSetup(props);
</script>

<template>
  <div class="doc-panel">
    <!-- 工具栏 -->
    <div class="doc-toolbar">
      <div class="toolbar-left">
        <button
          v-if="generating"
          class="cancel-btn"
          @click="cancelBackendGeneration"
        >
          {{ t("docPanel.cancel") }} {{ generatingTime }}s
        </button>
        <div v-else class="ai-generate-wrapper">
          <button
            class="ai-generate-btn"
            :disabled="!apiId"
            @click="generateDocWithAI"
          >
            <Icon name="sparkles" :size="14" />
            {{ t("docPanel.aiGenerate") }}
          </button>
        </div>
        <span v-if="lastUpdatedAt" class="last-updated">
          {{ t("docPanel.lastUpdated") }}: {{ lastUpdatedAt }}
        </span>
      </div>
      <div class="toolbar-right">
        <button
          class="toggle-btn"
          :disabled="generating"
          @click="toggleDocMode"
        >
          {{ docMode === "view" ? t("common.edit") : t("tabs.docs") }}
        </button>
        <button v-if="docMode === 'edit'" class="save-btn" @click="saveDoc">
          {{ t("common.save") }}
        </button>
      </div>
    </div>

    <!-- 展示模式 -->
    <div v-if="docMode === 'view'" class="doc-view-container">
      <!-- eslint-disable-next-line vue/no-v-html -->
      <div v-if="renderedDocHtml" class="doc-content" v-html="renderedDocHtml"></div>
      <div v-else class="doc-empty">
        <span class="empty-icon"><Icon name="file" :size="48" /></span>
        <p>{{ t("empty.noDoc") }}</p>
        <button class="edit-btn" :disabled="generating" @click="toggleDocMode">
          {{ t("common.edit") }}
        </button>
      </div>
    </div>

    <!-- 编辑模式 -->
    <div
      v-show="docMode === 'edit'"
      class="doc-editor-container"
      ref="docEditorContainer"
    ></div>
  </div>
</template>

<style scoped src="./style.css"></style>
