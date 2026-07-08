<script setup>
import { useI18n } from "vue-i18n";
import { useResponsePanelSetup } from "./index.js";
import Icon from "../Icon/index.vue";

const { t } = useI18n();

const props = defineProps({
  response: {
    type: Object,
    default: () => null,
  },
  loading: {
    type: Boolean,
    default: false,
  },
  testResults: {
    type: Array,
    default: () => [],
  },
  sseEvents: {
    type: Array,
    default: () => [],
  },
});

const emit = defineEmits(["save-response"]);

const {
  tabs,
  activeTab,
  statusClass,
  timingStages,
  sseEventBlocks,
  shouldShowSSEEvents,
  formatSize,
  formatTime,
  editorContainer,
  sseContainer,
  testStats,
  handleSaveResponse,
} = useResponsePanelSetup(props, emit);
</script>

<template>
  <div class="response-panel">
    <!-- 响应状态栏 -->
    <div class="response-status" v-if="response || loading">
      <div v-if="loading" class="loading-indicator">
        <span class="loading-spinner"></span>
        <span>{{ t("response.requesting") }}</span>
      </div>
      <template v-else-if="response">
        <div class="status-info">
          <div class="status-item">
            <span class="status-label">{{ t("response.status") }}:</span>
            <span class="status-value" :class="statusClass"
              >{{ response.status }} {{ response.statusText }}</span
            >
          </div>
          <div class="status-item">
            <span class="status-label">{{ t("response.time") }}:</span>
            <span class="status-value">{{ formatTime(response.time) }}</span>
          </div>
          <div class="status-item">
            <span class="status-label">{{ t("response.size") }}:</span>
            <span class="status-value">{{ formatSize(response.size) }}</span>
          </div>
          <div class="status-item" v-if="response.avgTime">
            <span class="status-label">{{ t("response.avgTime") }}:</span>
            <span class="status-value">{{ formatTime(response.avgTime) }}</span>
          </div>
        </div>
        <button class="save-response-btn" @click="handleSaveResponse">
          <Icon name="save" :size="14" />
          <span>{{ t("buttons.saveResponse") }}</span>
        </button>
      </template>
    </div>

    <!-- 响应标签页 -->
    <div class="response-tabs">
      <div
        v-for="tab in tabs"
        :key="tab.key"
        class="tab-item"
        :class="{ active: activeTab === tab.key }"
        @click="activeTab = tab.key"
      >
        {{ tab.name }}
      </div>
    </div>

    <!-- 响应内容 -->
    <div class="response-content">
      <!-- 空状态 -->
      <div v-if="!response && !loading" class="empty-state">
        <span class="empty-icon"><Icon name="send" :size="48" /></span>
        <p class="empty-text">{{ t("empty.noResponse") }}</p>
        <p class="empty-hint">{{ t("empty.noResponseHint") }}</p>
      </div>

      <!-- 加载状态 -->
      <div v-else-if="loading" class="loading-state">
        <div class="loading-spinner large"></div>
        <p>{{ t("response.sending") }}</p>
      </div>

      <!-- 响应体 - 两个容器都始终存在 -->
      <div
        v-show="response && !loading && activeTab === 'body'"
        class="body-content"
      >
        <!-- SSE 流式响应：显示事件列表 -->
        <div
          v-show="shouldShowSSEEvents"
          ref="sseContainer"
          class="sse-events-container"
        >
          <div
            v-for="(block, index) in sseEventBlocks"
            :key="index"
            class="sse-event-block"
          >
            <div class="sse-event-header">
              <span class="sse-event-index">#{{ index + 1 }}</span>
              <span v-if="block.time" class="sse-event-time">{{
                block.time
              }}</span>
            </div>
            <div class="sse-event-content">{{ block.data }}</div>
          </div>
        </div>
        <!-- 普通 HTTP 响应：Monaco Editor -->
        <div
          v-show="!shouldShowSSEEvents"
          ref="editorContainer"
          class="monaco-editor-container"
        ></div>
      </div>

      <!-- 响应头 -->
      <div
        v-show="response && !loading && activeTab === 'headers'"
        class="headers-content"
      >
        <div class="headers-list">
          <div
            v-for="(value, key) in response?.headers"
            :key="key"
            class="header-row"
          >
            <span class="header-key">{{ key }}</span>
            <span class="header-value">{{ value }}</span>
          </div>
        </div>
      </div>

      <!-- 测试结果 -->
      <div
        v-show="response && !loading && activeTab === 'tests'"
        class="tests-content"
      >
        <div v-if="testResults.length === 0" class="tests-empty">
          <span class="empty-text">{{ t("empty.noTests") }}</span>
          <span class="empty-hint">{{ t("empty.noTestsHint") }}</span>
        </div>
        <div v-else class="tests-summary">
          <span class="tests-total"
            >{{ t("tests.total") }}: {{ testStats.total }}</span
          >
          <span class="tests-passed"
            >{{ t("tests.passed") }}: {{ testStats.passed }}</span
          >
          <span class="tests-failed"
            >{{ t("tests.failed") }}: {{ testStats.failed }}</span
          >
        </div>
        <div v-if="testResults.length > 0" class="tests-list">
          <div
            v-for="(result, index) in testResults"
            :key="index"
            class="test-row"
            :class="{ passed: result.passed, failed: !result.passed }"
          >
            <span class="test-status-icon">{{
              result.passed ? "✓" : "✗"
            }}</span>
            <span class="test-name">{{ result.name }}</span>
            <span v-if="!result.passed && result.error" class="test-error">{{
              result.error
            }}</span>
          </div>
        </div>
      </div>

      <!-- 请求时间线 -->
      <div
        v-show="response && !loading && activeTab === 'timing'"
        class="timing-content"
      >
        <template v-if="timingStages.length">
          <div class="timing-bar-title">{{ t("timing.barTitle") }}</div>
          <div class="timing-bar">
            <div
              v-for="stage in timingStages"
              :key="stage.key"
              class="timing-bar-segment"
              :style="{ width: stage.percent + '%', background: stage.color }"
              :title="`${stage.label}: ${formatTime(stage.ms)}`"
            ></div>
          </div>
          <div class="timing-list">
            <div
              v-for="stage in timingStages"
              :key="stage.key"
              class="timing-row"
            >
              <span
                class="timing-dot"
                :style="{ background: stage.color }"
              ></span>
              <span class="timing-label">{{ stage.label }}</span>
              <span class="timing-value">{{ formatTime(stage.ms) }}</span>
              <span class="timing-percent">{{ stage.percent }}%</span>
            </div>
            <div class="timing-row timing-total">
              <span class="timing-dot"></span>
              <span class="timing-label">{{ t("timing.total") }}</span>
              <span class="timing-value">{{
                formatTime(response?.timing?.total_ms)
              }}</span>
              <span class="timing-percent">100%</span>
            </div>
          </div>
        </template>
        <div v-else class="timing-empty">{{ t("timing.noData") }}</div>
      </div>

      <!-- 其他标签页 -->
      <div
        v-show="
          response &&
          !loading &&
          activeTab !== 'body' &&
          activeTab !== 'headers' &&
          activeTab !== 'tests' &&
          activeTab !== 'timing'
        "
        class="placeholder-content"
      >
        <span class="placeholder-icon"
          ><Icon name="performance" :size="32"
        /></span>
        <p>{{ tabs.find((t) => t.key === activeTab)?.name }}</p>
        <p class="placeholder-hint">{{ t("common.developing") }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
