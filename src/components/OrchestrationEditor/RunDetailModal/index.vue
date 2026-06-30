<script setup>
import { watch } from "vue";
import { useRunDetailModalSetup } from "./index.js";
import { useDialogEscape } from "../../../composables/useDialogStack.js";

const props = defineProps({
  runDetail: Object,
});

const emit = defineEmits(["close"]);

const {
  expandedSteps,
  activeTabs,
  toggleStep,
  setActiveTab,
  isStepExpanded,
  getActiveTab,
  formatTime,
  getTotalTime,
  formatJson,
  formatUrl,
  getMethodClass,
  t,
} = useRunDetailModalSetup(props);

watch(
  () => props.runDetail,
  () => {
    if (props.runDetail) {
      expandedSteps.value = [];
      activeTabs.value = {};
    }
  },
  { immediate: true },
);

const closeModal = () => {
  emit("close");
};

// ESC 键关闭
useDialogEscape(() => props.runDetail != null, closeModal);
</script>

<template>
  <div class="modal-overlay" @click="closeModal">
    <div class="run-detail-modal" @click.stop>
      <div class="modal-header">
        <span class="modal-title">{{
          t("orchestration.runDetail.title")
        }}</span>
        <button class="close-btn" @click="closeModal">×</button>
      </div>

      <div class="modal-body" v-if="runDetail">
        <div class="run-summary">
          <div class="summary-item">
            <span class="status-badge" :class="runDetail.status">
              {{ t(`orchestration.runStatus.${runDetail.status}`) }}
            </span>
          </div>
          <div class="summary-item">
            <span class="summary-label">{{
              t("orchestration.runDetail.startTime")
            }}</span>
            <span class="summary-value">{{
              formatTime(runDetail.start_time)
            }}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">{{
              t("orchestration.runDetail.totalTime")
            }}</span>
            <span class="summary-value">{{ getTotalTime(runDetail) }}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">{{
              t("orchestration.runDetail.stats")
            }}</span>
            <span class="summary-value stats">
              <span class="stat success"
                >{{ runDetail.success_count }} {{ t("tests.passed") }}</span
              >
              <span class="stat failed" v-if="runDetail.failed_count > 0">
                {{ runDetail.failed_count }} {{ t("tests.failed") }}
              </span>
              <span class="stat skipped" v-if="runDetail.skipped_count > 0">
                {{ runDetail.skipped_count }}
                {{ t("orchestration.stepStatus.skipped") }}
              </span>
            </span>
          </div>
        </div>

        <div class="steps-detail-list">
          <div
            v-for="(step, index) in runDetail.steps"
            :key="step.step_id"
            class="step-detail-item"
            :class="[step.status, { expanded: isStepExpanded(step.step_id) }]"
          >
            <div class="step-summary-row" @click="toggleStep(step.step_id)">
              <span class="expand-icon">{{
                isStepExpanded(step.step_id) ? "▼" : "▶"
              }}</span>
              <span class="step-index">{{ index + 1 }}</span>
              <span class="step-name" :title="step.api_name">{{
                step.api_name
              }}</span>
              <span
                v-if="step.request_method"
                class="method-tag"
                :class="getMethodClass(step.request_method)"
              >
                {{ step.request_method }}
              </span>
              <span
                v-if="step.request_url"
                class="step-url"
                :title="step.request_url"
              >
                {{ formatUrl(step.request_url) }}
              </span>
              <span class="status-tag" :class="step.status">
                {{ t(`orchestration.stepStatus.${step.status}`) }}
              </span>
              <span class="response-time">{{ step.response_time }}ms</span>
              <span
                class="status-code"
                :class="
                  step.status_code >= 400 || step.status_code === 0
                    ? 'error'
                    : 'success'
                "
              >
                {{ step.status_code > 0 ? step.status_code : "-" }}
              </span>
            </div>

            <div v-if="isStepExpanded(step.step_id)" class="step-detail-panel">
              <div class="detail-tabs">
                <button
                  :class="{ active: getActiveTab(step.step_id) === 'request' }"
                  @click.stop="setActiveTab(step.step_id, 'request')"
                >
                  {{ t("orchestration.runDetail.request") }}
                </button>
                <button
                  :class="{ active: getActiveTab(step.step_id) === 'response' }"
                  @click.stop="setActiveTab(step.step_id, 'response')"
                >
                  {{ t("orchestration.runDetail.response") }}
                </button>
                <button
                  :class="{ active: getActiveTab(step.step_id) === 'tests' }"
                  @click.stop="setActiveTab(step.step_id, 'tests')"
                  :disabled="
                    !step.test_results || step.test_results.length === 0
                  "
                >
                  {{ t("orchestration.runDetail.tests") }}
                  <span
                    v-if="step.test_results && step.test_results.length > 0"
                    class="tab-count"
                  >
                    {{ step.test_results.length }}
                  </span>
                </button>
              </div>

              <div class="detail-content">
                <div
                  v-if="getActiveTab(step.step_id) === 'request'"
                  class="request-detail"
                >
                  <div class="detail-section">
                    <label>{{ t("orchestration.runDetail.requestUrl") }}</label>
                    <div class="detail-value url-value">
                      {{ step.request_url }}
                    </div>
                  </div>
                  <div class="detail-section" v-if="step.request_original_url">
                    <label>{{
                      t("orchestration.runDetail.originalUrl")
                    }}</label>
                    <div class="detail-value original-url">
                      {{ step.request_original_url }}
                    </div>
                  </div>
                  <div
                    class="detail-section"
                    v-if="
                      step.request_headers && step.request_headers.length > 0
                    "
                  >
                    <label>{{
                      t("orchestration.runDetail.requestHeaders")
                    }}</label>
                    <div class="headers-list">
                      <div
                        v-for="h in step.request_headers"
                        :key="h.key"
                        class="header-row"
                      >
                        <span class="header-key">{{ h.key }}</span>
                        <span class="header-value">{{ h.value }}</span>
                      </div>
                    </div>
                  </div>
                  <div class="detail-section" v-if="step.request_body">
                    <label>{{
                      t("orchestration.runDetail.requestBody")
                    }}</label>
                    <pre class="body-content">{{
                      formatJson(step.request_body)
                    }}</pre>
                  </div>
                  <div class="detail-section" v-if="step.request_body_type">
                    <label>{{ t("orchestration.runDetail.bodyType") }}</label>
                    <span class="body-type-tag">{{
                      step.request_body_type
                    }}</span>
                  </div>
                </div>

                <div
                  v-if="getActiveTab(step.step_id) === 'response'"
                  class="response-detail"
                >
                  <div class="detail-section">
                    <label>{{ t("orchestration.runDetail.statusCode") }}</label>
                    <span
                      class="status-code-badge"
                      :class="
                        step.status_code >= 400 || step.status_code === 0
                          ? 'error'
                          : 'success'
                      "
                    >
                      {{ step.status_code > 0 ? step.status_code : "N/A" }}
                    </span>
                  </div>
                  <div
                    class="detail-section"
                    v-if="
                      step.response_headers &&
                      Object.keys(step.response_headers).length > 0
                    "
                  >
                    <label>{{
                      t("orchestration.runDetail.responseHeaders")
                    }}</label>
                    <div class="headers-list">
                      <div
                        v-for="(value, key) in step.response_headers"
                        :key="key"
                        class="header-row"
                      >
                        <span class="header-key">{{ key }}</span>
                        <span class="header-value">{{ value }}</span>
                      </div>
                    </div>
                  </div>
                  <div class="detail-section" v-if="step.response_body">
                    <label>{{
                      t("orchestration.runDetail.responseBody")
                    }}</label>
                    <pre class="body-content">{{
                      formatJson(step.response_body)
                    }}</pre>
                  </div>
                  <div class="detail-section" v-if="step.failure_message">
                    <label>{{
                      t("orchestration.runDetail.failureReason")
                    }}</label>
                    <div class="failure-message">
                      {{ step.failure_message }}
                    </div>
                  </div>
                </div>

                <div
                  v-if="getActiveTab(step.step_id) === 'tests'"
                  class="tests-detail"
                >
                  <div
                    v-if="!step.test_results || step.test_results.length === 0"
                    class="empty-tests"
                  >
                    {{ t("orchestration.runDetail.noTests") }}
                  </div>
                  <div v-else class="test-results-list">
                    <div
                      v-for="test in step.test_results"
                      :key="test.name"
                      class="test-item"
                      :class="test.passed ? 'passed' : 'failed'"
                    >
                      <span class="test-icon">{{
                        test.passed ? "✓" : "✗"
                      }}</span>
                      <span class="test-name">{{ test.name }}</span>
                      <span
                        v-if="!test.passed && test.error"
                        class="test-error"
                        >{{ test.error }}</span
                      >
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
