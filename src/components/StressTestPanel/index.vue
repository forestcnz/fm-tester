<script setup>
import { useStressTestPanelSetup } from "./index.js";
import Icon from "../Icon/index.vue";
import { useDialogEscape } from "../../composables/useDialogStack.js";

const props = defineProps({
  workspaceId: { type: String, default: "" },
  apiId: { type: String, default: "" },
  apiName: { type: String, default: "" },
  method: { type: String, default: "GET" },
  url: { type: String, default: "" },
  headers: { type: Array, default: () => [] },
  body: { type: String, default: "" },
  bodyType: { type: String, default: "none" },
  formFields: { type: Array, default: () => [] },
  collectionVariables: { type: Array, default: () => [] },
});

const {
  config,
  isRunning,
  progress,
  result,
  historyResults,
  error,
  startTest,
  stopTest,
  deleteHistory,
  viewHistoryDetail,
  viewingHistoryId,
  backToList,
  getStatusClass,
  formatDate,
  showFailedDetails,
  toggleFailedDetails,
  chartRef,
  t,
} = useStressTestPanelSetup(props);

// ESC 键关闭失败详情弹窗
useDialogEscape(showFailedDetails, () => {
  showFailedDetails.value = false;
});
</script>

<template>
  <div class="stress-test-panel">
    <div v-if="error" class="error-message">{{ error }}</div>

    <div class="config-section">
      <div class="config-left">
        <div class="config-row">
          <label>{{ t("stress.concurrent") }}</label>
          <input
            type="number"
            v-model.number="config.concurrent"
            min="1"
            max="999"
            :disabled="isRunning"
          />
        </div>
        <div class="config-row">
          <label>{{ t("stress.rampUpSeconds") }}</label>
          <input
            type="number"
            v-model.number="config.rampUpSeconds"
            min="0"
            max="3600"
            :disabled="isRunning"
          />
          <span class="unit">{{ t("stress.seconds") }}</span>
        </div>
        <div class="config-row">
          <label>{{ t("stress.totalRequests") }}</label>
          <input
            type="number"
            v-model.number="config.totalRequests"
            min="1"
            max="1000000"
            :disabled="isRunning"
          />
          <span class="hint">{{ t("stress.or") }}</span>
          <input
            type="number"
            v-model.number="config.durationSeconds"
            min="1"
            max="86400"
            :disabled="isRunning"
          />
          <span class="unit">{{ t("stress.seconds") }}</span>
        </div>
        <div class="config-row">
          <label>{{ t("stress.timeoutMs") }}</label>
          <input
            type="number"
            v-model.number="config.timeoutMs"
            min="1000"
            max="600000"
            step="1000"
            :disabled="isRunning"
          />
          <span class="unit">{{ t("stress.ms") }}</span>
        </div>
      </div>
      <div class="cfg-hint-row">
        <div class="hint-it">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M13 2 3 14h9l-1 8 10-12h-9z" /></svg>
          {{ t("stress.preScriptHint") }}
        </div>
        <div class="hint-it">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 11l3 3 8-8" /><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" /></svg>
          {{ t("stress.postScriptHint") }}
        </div>
      </div>
    </div>

    <div class="control-buttons">
      <button
        class="start-btn"
        @click="startTest"
        :disabled="isRunning || !apiId"
      >
        <Icon name="play" :size="16" />
        {{ t("stress.start") }}
      </button>
      <button class="stop-btn" @click="stopTest" :disabled="!isRunning">
        <Icon name="stop" :size="16" />
        {{ t("stress.stop") }}
      </button>
    </div>

    <div class="progress-section">
      <div ref="chartRef" class="chart-container"></div>
      <div class="stats-row" v-if="isRunning || progress">
        <span
          >{{ t("stress.completed") }}:
          {{ progress?.completed_requests || 0 }}</span
        >
        <span
          >{{ t("stress.currentQps") }}:
          {{ (progress?.current_qps || 0).toFixed(2) }}</span
        >
        <span
          >{{ t("stress.avgTime") }}:
          {{ (progress?.current_avg_time_ms || 0).toFixed(2) }}ms</span
        >
        <span class="success"
          >{{ t("stress.success") }}:
          {{ progress?.successful_requests || 0 }}</span
        >
        <span class="failed"
          >{{ t("stress.failed") }}: {{ progress?.failed_requests || 0 }}</span
        >
      </div>
    </div>

    <!-- 历史记录区域：列表或详情 -->
    <div class="history-section">
      <!-- 详情显示 -->
      <div v-if="viewingHistoryId && result">
        <div class="detail-header">
          <button class="back-btn" @click="backToList">
            <Icon name="arrow-left" :size="16" />
            {{ t("stress.back") }}
          </button>
          <span class="detail-title">{{ result.api_name }}</span>
        </div>
        <div class="detail-content">
          <div class="metrics-grid">
            <div class="metric-card primary">
              <div class="metric-value">{{ (result.qps || 0).toFixed(2) }}</div>
              <div class="metric-label">{{ t("stress.qps") }}</div>
            </div>
            <div
              class="metric-card clickable"
              :class="{
                'has-failed': result.failed_request_details?.length > 0,
              }"
              @click="toggleFailedDetails"
            >
              <div class="metric-value">
                {{ ((result.success_rate || 0) * 100).toFixed(1) }}%
              </div>
              <div class="metric-label">{{ t("stress.successRate") }}</div>
            </div>
            <div class="metric-card">
              <div class="metric-value">
                {{ (result.avg_time_ms || 0).toFixed(1) }}ms
              </div>
              <div class="metric-label">{{ t("stress.avgTime") }}</div>
            </div>
            <div class="metric-card">
              <div class="metric-value">{{ result.total_requests || 0 }}</div>
              <div class="metric-label">{{ t("stress.totalRequests") }}</div>
            </div>
          </div>
          <div class="config-info" v-if="result.config">
            <div class="config-row">
              <span class="config-label">{{ t("stress.method") }}:</span>
              <span
                class="config-value method"
                :class="result.config.method?.toLowerCase()"
                >{{ result.config.method }}</span
              >
              <span class="config-url">{{ result.config.url }}</span>
            </div>
            <div class="config-row">
              <span class="config-label">{{ t("stress.concurrent") }}:</span>
              <span class="config-value">{{ result.config.concurrent }}</span>
              <span class="config-label">{{ t("stress.rampUpSeconds") }}:</span>
              <span class="config-value"
                >{{ result.config.ramp_up_seconds
                }}{{ t("stress.seconds") }}</span
              >
            </div>
            <div class="config-row">
              <span class="config-label">{{ t("stress.timeoutMs") }}:</span>
              <span class="config-value"
                >{{ result.config.timeout_ms }}{{ t("stress.ms") }}</span
              >
              <template v-if="result.config.total_requests">
                <span class="config-label"
                  >{{ t("stress.totalRequests") }}:</span
                >
                <span class="config-value">{{
                  result.config.total_requests
                }}</span>
              </template>
              <template v-else-if="result.config.duration_seconds">
                <span class="config-label"
                  >{{ t("stress.durationSeconds") }}:</span
                >
                <span class="config-value"
                  >{{ result.config.duration_seconds
                  }}{{ t("stress.seconds") }}</span
                >
              </template>
            </div>
          </div>
          <div class="time-grid">
            <div class="time-item">
              <span class="time-label">P50</span>
              <span class="time-value"
                >{{ (result.p50_time_ms || 0).toFixed(1) }}ms</span
              >
            </div>
            <div class="time-item">
              <span class="time-label">P90</span>
              <span class="time-value"
                >{{ (result.p90_time_ms || 0).toFixed(1) }}ms</span
              >
            </div>
            <div class="time-item">
              <span class="time-label">P95</span>
              <span class="time-value"
                >{{ (result.p95_time_ms || 0).toFixed(1) }}ms</span
              >
            </div>
            <div class="time-item">
              <span class="time-label">P99</span>
              <span class="time-value"
                >{{ (result.p99_time_ms || 0).toFixed(1) }}ms</span
              >
            </div>
            <div class="time-item">
              <span class="time-label">{{ t("stress.min") }}</span>
              <span class="time-value">{{ result.min_time_ms || 0 }}ms</span>
            </div>
            <div class="time-item">
              <span class="time-label">{{ t("stress.max") }}</span>
              <span class="time-value">{{ result.max_time_ms || 0 }}ms</span>
            </div>
          </div>
          <div
            class="status-grid"
            v-if="
              result.status_distribution &&
              Object.keys(result.status_distribution).length > 0
            "
          >
            <div
              v-for="[status, count] in Object.entries(
                result.status_distribution,
              )"
              :key="status"
              class="status-item"
            >
              <span
                class="status-code"
                :class="getStatusClass(Number(status))"
                >{{ status }}</span
              >
              <span class="status-count">{{ count }}</span>
              <span class="status-percent"
                >{{ ((count / result.total_requests) * 100).toFixed(1) }}%</span
              >
            </div>
          </div>
          <div
            class="error-grid"
            v-if="
              result.error_distribution &&
              Object.keys(result.error_distribution).length > 0
            "
          >
            <div
              v-for="[error, count] in Object.entries(
                result.error_distribution,
              )"
              :key="error"
              class="error-item"
            >
              <span class="error-text">{{ error }}</span>
              <span class="error-num">{{ count }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 列表显示 -->
      <div v-else>
        <h4>{{ t("stress.records") }}</h4>
        <div v-if="historyResults.length === 0" class="empty-history">
          {{ t("stress.noRecords") }}
        </div>
        <div v-else class="history-list">
          <RecycleScroller
            class="scroller"
            :items="historyResults"
            :item-size="40"
            key-field="id"
            :buffer="200"
          >
            <template #default="{ item: h }">
              <div class="history-item" @click="viewHistoryDetail(h.id)">
                <span class="history-name">{{ h.api_name }}</span>
                <span class="history-date">{{ formatDate(h.start_time) }}</span>
                <span class="history-qps"
                  >{{ (h.qps || 0).toFixed(2) }} QPS</span
                >
                <span class="history-rate">
                  {{ ((h.success_rate || 0) * 100).toFixed(1) }}%
                </span>
                <button
                  class="delete-btn"
                  @click.stop="deleteHistory(h.id)"
                  :title="t('common.delete')"
                >
                  ×
                </button>
              </div>
            </template>
          </RecycleScroller>
        </div>
      </div>
    </div>

    <!-- 失败请求详情弹窗 -->
    <div
      class="failed-modal-overlay"
      v-if="showFailedDetails && result?.failed_request_details?.length > 0"
      @click.self="showFailedDetails = false"
    >
      <div class="failed-modal">
        <div class="failed-modal-header">
          <span
            >{{ t("stress.failedRequests") }} ({{
              result.failed_request_details.length
            }})</span
          >
          <button class="close-btn" @click="showFailedDetails = false">
            ×
          </button>
        </div>
        <div class="failed-modal-body">
          <div
            v-for="(req, index) in result.failed_request_details"
            :key="index"
            class="failed-request-item"
          >
            <span class="failed-time">{{ req.time }}</span>
            <span class="failed-elapsed">{{ req.elapsed_ms }}ms</span>
            <span v-if="req.status" class="failed-status">{{
              req.status
            }}</span>
            <span class="failed-error">{{ req.error }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
