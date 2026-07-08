<script setup>
import { useSavedResponseDocPanelSetup } from "./index.js";
import Icon from "../Icon/index.vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

const props = defineProps({
  savedResponse: {
    type: Object,
    default: () => null,
  },
  workspaceId: {
    type: String,
    default: "",
  },
});

const emit = defineEmits(["close"]);

const { renderedDocHtml } = useSavedResponseDocPanelSetup(props, emit);
</script>

<template>
  <div class="saved-response-doc-panel" v-if="savedResponse">
    <!-- 工具栏 -->
    <div class="doc-toolbar">
      <div class="toolbar-left">
        <h3 class="doc-title">{{ savedResponse.name }}</h3>
        <span class="created-time">
          {{ t("savedResponse.savedAt") }}: {{ savedResponse.created_at }}
        </span>
      </div>
    </div>

    <!-- 文档展示 -->
    <div class="doc-view-container">
      <!-- eslint-disable-next-line vue/no-v-html -->
      <div
        v-if="renderedDocHtml"
        class="doc-content"
        v-html="renderedDocHtml"
      ></div>
      <div v-else class="doc-empty">
        <span class="empty-icon"><Icon name="file" :size="48" /></span>
        <p class="empty-text">{{ t("empty.noDocContent") }}</p>
      </div>
    </div>
  </div>

  <!-- 无数据提示 -->
  <div class="saved-response-doc-empty" v-else>
    <span class="empty-icon"><Icon name="doc" :size="48" /></span>
    <p class="empty-text">{{ t("empty.selectSavedResponse") }}</p>
    <p class="empty-hint">{{ t("empty.selectSavedResponseHint") }}</p>
  </div>
</template>

<style scoped src="./style.css"></style>
