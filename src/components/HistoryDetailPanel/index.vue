<script setup>
import { useHistoryDetailSetup } from "./index.js";
import Icon from "../Icon/index.vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

const props = defineProps({
  entry: Object,
});

const {
  requestTab,
  responseTab,
  detectedRequestLanguage,
  detectedResponseLanguage,
  formatResponseTime,
  formatSize,
  getStatusClass,
  getMethodClass,
  formatTime,
  requestEditorContainer,
  responseEditorContainer,
} = useHistoryDetailSetup(props);

const requestTabs = [
  { key: "params", name: t("tabs.params") },
  { key: "headers", name: t("tabs.headers") },
  { key: "body", name: t("tabs.body") },
];

const responseTabs = [
  { key: "body", name: t("tabs.responseBody") },
  { key: "headers", name: t("tabs.responseHeaders") },
];

// 解析 URL 参数
const parseUrlParams = (url) => {
  if (!url || !url.includes("?")) return [];
  const queryString = url.split("?")[1].split("#")[0];
  const params = [];
  const pairs = queryString.split("&");
  for (const pair of pairs) {
    const [key, value] = pair.split("=");
    params.push({
      key: key || "",
      value: value || "",
    });
  }
  return params;
};
</script>

<template>
  <div class="history-detail" v-if="entry">
    <!-- 请求区 -->
    <div class="request-area">
      <!-- URL 输入行 -->
      <div class="url-bar">
        <div class="method-selector">
          <span class="method-display" :class="getMethodClass(entry.method)">{{
            entry.method
          }}</span>
        </div>
        <div class="url-display-wrapper">
          <span class="url-display">{{ entry.url }}</span>
        </div>
        <span class="history-time">{{ formatTime(entry.created_at) }}</span>
      </div>

      <!-- 请求配置标签页 -->
      <div class="request-tabs">
        <div
          v-for="tab in requestTabs"
          :key="tab.key"
          class="tab-item"
          :class="{ active: requestTab === tab.key }"
          @click="requestTab = tab.key"
        >
          {{ tab.name }}
        </div>
      </div>

      <!-- 标签页内容 -->
      <div class="tab-content">
        <!-- 参数（URL Query） -->
        <div v-show="requestTab === 'params'" class="params-panel">
          <div class="params-list">
            <div class="params-header">
              <span class="col-check"></span>
              <span class="col-key">{{ t("table.paramName") }}</span>
              <span class="col-value">{{ t("table.paramValue") }}</span>
              <span class="col-desc">{{ t("table.description") }}</span>
            </div>
            <div v-if="!entry.url.includes('?')" class="empty-params">
              {{ t("empty.noUrlParams") }}
            </div>
            <div
              v-for="(param, index) in parseUrlParams(entry.url)"
              :key="index"
              class="param-row"
            >
              <span class="col-check">
                <input type="checkbox" checked disabled />
              </span>
              <span class="col-key">
                <span class="readonly-value">{{ param.key }}</span>
              </span>
              <span class="col-value">
                <span class="readonly-value">{{ param.value }}</span>
              </span>
              <span class="col-desc">
                <span class="readonly-value"></span>
              </span>
            </div>
          </div>
        </div>

        <!-- 请求头 -->
        <div v-show="requestTab === 'headers'" class="headers-panel">
          <div class="params-list">
            <div class="params-header">
              <span class="col-check"></span>
              <span class="col-key">{{ t("table.headerName") }}</span>
              <span class="col-value">{{ t("table.headerValue") }}</span>
              <span class="col-desc">{{ t("table.description") }}</span>
            </div>
            <div
              v-if="!entry.headers || entry.headers.length === 0"
              class="empty-params"
            >
              {{ t("empty.noRequestHeaders") }}
            </div>
            <div
              v-for="(header, index) in entry.headers"
              :key="index"
              class="param-row"
            >
              <span class="col-check">
                <input type="checkbox" :checked="header.enabled" disabled />
              </span>
              <span class="col-key">
                <span class="readonly-value">{{ header.key }}</span>
              </span>
              <span class="col-value">
                <span class="readonly-value">{{ header.value }}</span>
              </span>
              <span class="col-desc">
                <span class="readonly-value">{{
                  header.description || ""
                }}</span>
              </span>
            </div>
          </div>
        </div>

        <!-- 请求体 -->
        <div v-show="requestTab === 'body'" class="body-panel">
          <div class="body-toolbar">
            <div class="body-type-display">
              {{ t("tabs.body") }} {{ t("common.type") }}:
              <span class="type-value">{{ entry.body_type || "none" }}</span>
            </div>
            <div class="language-indicator" v-if="entry.body">
              <span class="language-tag">{{ detectedRequestLanguage }}</span>
            </div>
          </div>
          <div v-if="!entry.body" class="body-empty">
            <span class="empty-text">{{ t("empty.noRequestBody") }}</span>
          </div>
          <div v-else class="editor-wrapper">
            <div
              ref="requestEditorContainer"
              class="monaco-editor-container"
            ></div>
          </div>
        </div>
      </div>
    </div>

    <!-- 响应区 -->
    <div class="response-area">
      <!-- 响应状态栏 -->
      <div class="response-status">
        <div class="status-info">
          <div class="status-item">
            <span class="status-label">{{ t("response.status") }}:</span>
            <span class="status-value" :class="getStatusClass(entry.status)"
              >{{ entry.status }} {{ entry.status_text }}</span
            >
          </div>
          <div class="status-item">
            <span class="status-label">{{ t("response.time") }}:</span>
            <span class="status-value">{{
              formatResponseTime(entry.time)
            }}</span>
          </div>
          <div class="status-item">
            <span class="status-label">{{ t("response.size") }}:</span>
            <span class="status-value">{{ formatSize(entry.size) }}</span>
          </div>
        </div>
      </div>

      <!-- 响应标签页 -->
      <div class="response-tabs">
        <div
          v-for="tab in responseTabs"
          :key="tab.key"
          class="tab-item"
          :class="{ active: responseTab === tab.key }"
          @click="responseTab = tab.key"
        >
          {{ tab.name }}
        </div>
        <div class="language-indicator" v-show="responseTab === 'body'">
          <span class="language-tag">{{ detectedResponseLanguage }}</span>
        </div>
      </div>

      <!-- 响应内容 -->
      <div class="response-content">
        <!-- 响应体 - Monaco Editor -->
        <div v-show="responseTab === 'body'" class="body-content">
          <div
            ref="responseEditorContainer"
            class="monaco-editor-container"
          ></div>
        </div>

        <!-- 响应头 -->
        <div v-show="responseTab === 'headers'" class="headers-content">
          <div class="headers-list" v-if="entry.response_headers">
            <div
              v-for="(value, key) in entry.response_headers"
              :key="key"
              class="header-row"
            >
              <span class="header-key">{{ key }}</span>
              <span class="header-value">{{ value }}</span>
            </div>
          </div>
          <div v-else class="empty-state">
            {{ t("empty.noResponseHeaders") }}
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- 无数据提示 -->
  <div class="history-empty" v-else>
    <span class="empty-icon"><Icon name="history" :size="48" /></span>
    <p class="empty-text">{{ t("empty.selectHistory") }}</p>
    <p class="empty-hint">{{ t("empty.selectHistoryHint") }}</p>
  </div>
</template>

<style scoped src="./style.css"></style>
