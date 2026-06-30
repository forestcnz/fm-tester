<script setup>
import { useI18n } from "vue-i18n";
import { ref, computed, watch, onUnmounted } from "vue";
import { useOrchestrationEditorSetup } from "./index.js";
import { useOrchestrationSchedule } from "../../composables/useOrchestrationSchedule";
import { showToast } from "../../composables/useToast";
import Icon from "../Icon/index.vue";
import RunDetailModal from "./RunDetailModal/index.vue";
import ContextMenu from "../Sidebar/ContextMenu/index.vue";
import { useDialogEscape } from "../../composables/useDialogStack.js";

const { t } = useI18n();

const props = defineProps({
  workspaceId: String,
  orchestrationId: String,
});

const emit = defineEmits(["stepsChanged"]);

const {
  orchestration,
  steps,
  collections,
  runHistory,
  isRunning,
  currentStepIndex,
  runProgress,
  addStep,
  updateStep,
  removeStep,
  reorderSteps,
  runOrchestration,
  stopOrchestration,
  viewRunDetail,
  deleteRun,
  clearAllRuns,
  getApiInfo,
} = useOrchestrationEditorSetup(props, emit);

// 定时任务配置
const {
  schedule,
  cronError,
  isLoading: scheduleLoading,
  getNextRunTimes,
  updateSchedule,
  validateCron,
  formatTime,
  loadSchedule,
} = useOrchestrationSchedule(
  computed(() => props.workspaceId),
  computed(() => props.orchestrationId),
);

const showSchedulePanel = ref(false);
const cronInput = ref("");
const tempEnabled = ref(false); // 弹窗内临时启停状态

// 接下来 5 次执行时间（异步加载）
const nextRunTimes = ref([]);

// 监听 cronInput 变化，异步获取执行时间
watch([cronInput, cronError], async ([newCron, newError]) => {
  if (newCron && !newError) {
    const times = await getNextRunTimes(newCron, 5);
    nextRunTimes.value = times;
  } else {
    nextRunTimes.value = [];
  }
});

const showApiSelector = ref(false);
const showStepConfig = ref(false);
const showRunDetail = ref(false);
const selectedRun = ref(null);
const runDetailData = ref(null);
const configStepId = ref(null);
const stepConfigData = ref({});
const searchQuery = ref("");

const filteredApis = computed(() => {
  if (!searchQuery.value) return collections.value;
  const query = searchQuery.value.toLowerCase();
  return collections.value.filter(
    (api) =>
      api.name.toLowerCase().includes(query) ||
      api.url?.toLowerCase().includes(query),
  );
});

const openApiSelector = () => {
  searchQuery.value = "";
  showApiSelector.value = true;
};

const selectApi = async (api) => {
  showApiSelector.value = false;
  await addStep(api.id, api.name);
};

const openStepConfig = (step) => {
  configStepId.value = step.id;
  stepConfigData.value = {
    name: step.name || getApiInfo(step.api_id)?.name || "",
    enabled: step.enabled,
    wait_before: step.wait_before,
    retry_count: step.retry_count,
    retry_delay: step.retry_delay,
    on_failure: step.on_failure,
  };
  showStepConfig.value = true;
};

const saveStepConfig = async () => {
  showStepConfig.value = false;
  await updateStep(configStepId.value, stepConfigData.value);
};

const getMethodClass = (method) => method?.toLowerCase() || "";

const openRunDetail = async (run) => {
  selectedRun.value = run;
  const detail = await viewRunDetail(run.id);
  runDetailData.value = detail;
  showRunDetail.value = true;
};

const closeRunDetail = () => {
  showRunDetail.value = false;
  selectedRun.value = null;
  runDetailData.value = null;
};

const getStepRunStatus = (stepId) => {
  const progress = runProgress.value.find((p) => p.step_id === stepId);
  return progress?.status || null;
};

// ESC 键关闭弹窗
useDialogEscape(showApiSelector, () => {
  showApiSelector.value = false;
});
useDialogEscape(showStepConfig, () => {
  showStepConfig.value = false;
});
useDialogEscape(showRunDetail, closeRunDetail);
useDialogEscape(showSchedulePanel, () => {
  showSchedulePanel.value = false;
});

// 步骤右键菜单
const stepContextMenu = ref({ visible: false, x: 0, y: 0, stepId: null });

const openStepContextMenu = (e, step) => {
  e.preventDefault();
  e.stopPropagation();
  stepContextMenu.value = {
    visible: true,
    x: e.clientX,
    y: e.clientY,
    stepId: step.id,
  };
};

const closeStepContextMenu = () => {
  stepContextMenu.value.visible = false;
};

const handleStepContextAction = async (action) => {
  const stepId = stepContextMenu.value.stepId;
  closeStepContextMenu();
  if (action === "delete" && stepId) {
    await removeStep(stepId);
  }
};

useDialogEscape(() => stepContextMenu.value.visible, closeStepContextMenu);

// 拖拽排序
const DRAG_THRESHOLD = 4;
const dragState = ref({
  draggingId: null,
  dragOverId: null,
  dragOverPosition: null,
});
const isDragging = ref(false);
const dragStartY = ref(0);
const dragStartId = ref(null);
const dragListenersAdded = ref(false); // 跟踪监听器是否已添加

const onMouseDown = (e, step) => {
  if (e.button !== 0) return;
  dragStartY.value = e.clientY;
  dragStartId.value = step.id;
  isDragging.value = false;
  document.addEventListener("mousemove", onMouseMove);
  document.addEventListener("mouseup", onMouseUp);
  dragListenersAdded.value = true;
};

const onMouseMove = (e) => {
  if (!dragStartId.value) return;

  const deltaY = Math.abs(e.clientY - dragStartY.value);
  if (!isDragging.value && deltaY < DRAG_THRESHOLD) return;

  if (!isDragging.value) {
    isDragging.value = true;
    dragState.value = {
      draggingId: dragStartId.value,
      dragOverId: null,
      dragOverPosition: null,
    };
  }

  const stepEls = document.querySelectorAll(".step-item");
  for (const el of stepEls) {
    const rect = el.getBoundingClientRect();
    const id = el.dataset.stepId;
    if (id && id !== dragState.value.draggingId) {
      const midY = rect.top + rect.height / 2;
      if (e.clientY >= rect.top && e.clientY <= rect.bottom) {
        dragState.value.dragOverId = id;
        dragState.value.dragOverPosition =
          e.clientY < midY ? "before" : "after";
        return;
      }
    }
  }
  dragState.value.dragOverId = null;
  dragState.value.dragOverPosition = null;
};

const onMouseUp = async () => {
  document.removeEventListener("mousemove", onMouseMove);
  document.removeEventListener("mouseup", onMouseUp);
  dragListenersAdded.value = false;

  if (isDragging.value && dragState.value.dragOverId) {
    const dragIndex = steps.value.findIndex(
      (s) => s.id === dragState.value.draggingId,
    );
    const overIndex = steps.value.findIndex(
      (s) => s.id === dragState.value.dragOverId,
    );
    if (dragIndex !== -1 && overIndex !== -1 && dragIndex !== overIndex) {
      const newOrder = [...steps.value.map((s) => s.id)];
      const [removed] = newOrder.splice(dragIndex, 1);
      const insertIndex =
        dragState.value.dragOverPosition === "before"
          ? dragIndex < overIndex
            ? overIndex - 1
            : overIndex
          : dragIndex < overIndex
            ? overIndex
            : overIndex + 1;
      newOrder.splice(insertIndex, 0, removed);
      await reorderSteps(newOrder);
    }
  }

  dragState.value = {
    draggingId: null,
    dragOverId: null,
    dragOverPosition: null,
  };
  isDragging.value = false;
  dragStartId.value = null;
};

const getTotalTime = (run) => {
  if (!run?.total_time) return "0ms";
  if (run.total_time < 1000) return `${run.total_time}ms`;
  return `${(run.total_time / 1000).toFixed(2)}s`;
};

// 定时配置相关方法
const toggleSchedulePanel = async () => {
  showSchedulePanel.value = !showSchedulePanel.value;
  if (showSchedulePanel.value) {
    // 打开弹窗时重新加载定时配置数据
    await loadSchedule();
    // 初始化临时状态
    cronInput.value = schedule.value.cron_expression || "";
    tempEnabled.value = schedule.value.enabled || false;
    // 清空错误状态并获取执行时间
    cronError.value = "";
    if (cronInput.value) {
      const times = await getNextRunTimes(cronInput.value, 5);
      nextRunTimes.value = times;
    } else {
      nextRunTimes.value = [];
    }
  }
};

const handleCronInput = (value) => {
  cronInput.value = value;
  validateCron(value);
};

const saveScheduleConfig = async () => {
  if (cronError.value) {
    return;
  }

  // 如果要启用，必须先验证 cron 表达式
  if (tempEnabled.value && !cronInput.value) {
    showToast(t("toast.cronExpressionRequired"), "warning");
    return;
  }

  const success = await updateSchedule({
    enabled: tempEnabled.value,
    cron_expression: cronInput.value,
  });

  if (success) {
    showSchedulePanel.value = false;
  }
};

// 组件卸载时强制清理事件监听器，防止内存泄漏
onUnmounted(() => {
  if (dragListenersAdded.value) {
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
    dragListenersAdded.value = false;
  }
});
</script>

<template>
  <div class="orchestration-editor">
    <div class="editor-header">
      <div class="header-title">
        <Icon name="orchestration" :size="20" />
        <span>{{ orchestration?.name || t("nav.orchestration") }}</span>
        <span v-if="schedule.enabled" class="schedule-badge">{{
          t("schedule.enabled")
        }}</span>
      </div>
      <div class="header-actions">
        <button
          class="action-btn schedule-btn"
          :class="{ active: schedule.enabled }"
          @click="toggleSchedulePanel"
        >
          <span>{{ t("schedule.title") }}</span>
        </button>
        <button
          class="action-btn run-btn"
          :disabled="isRunning || steps.length === 0"
          @click="runOrchestration"
        >
          <span>{{
            isRunning ? t("orchestration.running") : t("orchestration.run")
          }}</span>
        </button>
        <button
          v-if="isRunning"
          class="action-btn stop-btn"
          @click="stopOrchestration"
        >
          <span>{{ t("orchestration.stop") }}</span>
        </button>
      </div>
    </div>

    <div class="editor-body">
      <!-- 定时配置弹窗 -->
      <div
        v-if="showSchedulePanel"
        class="schedule-modal-overlay"
        @click.self="showSchedulePanel = false"
      >
        <div class="schedule-modal">
          <div class="schedule-modal-header">
            <span class="schedule-modal-title">{{
              t("schedule.configTitle")
            }}</span>
            <button
              class="schedule-modal-close"
              @click="showSchedulePanel = false"
            >
              ×
            </button>
          </div>

          <div class="schedule-modal-body">
            <!-- 启用开关 -->
            <div class="schedule-field">
              <label>{{ t("schedule.enableSchedule") }}</label>
              <div class="toggle-switch" @click="tempEnabled = !tempEnabled">
                <div class="toggle-track" :class="{ enabled: tempEnabled }">
                  <div class="toggle-thumb"></div>
                </div>
                <span class="toggle-text">{{
                  tempEnabled ? t("common.enabled") : t("schedule.disabled")
                }}</span>
              </div>
            </div>

            <!-- Cron 表达式输入 -->
            <div class="schedule-field cron-field-wrapper">
              <label>{{ t("schedule.cronExpression") }}</label>
              <input
                v-model="cronInput"
                class="cron-input"
                :placeholder="t('schedule.cronPlaceholder')"
                @input="handleCronInput($event.target.value)"
              />
              <div v-if="cronError" class="cron-error">{{ cronError }}</div>
            </div>

            <!-- 预测执行时间 -->
            <div class="next-run-times">
              <div class="next-run-title">{{ t("schedule.nextRunTimes") }}</div>
              <div class="next-run-list">
                <div
                  v-for="(time, index) in nextRunTimes"
                  :key="index"
                  class="next-run-item"
                >
                  <span class="run-index">{{ index + 1 }}</span>
                  <span class="run-time">{{ time || "--" }}</span>
                </div>
                <div
                  v-for="i in 5 - nextRunTimes.length"
                  :key="'empty-' + i"
                  class="next-run-item"
                >
                  <span class="run-index">{{ nextRunTimes.length + i }}</span>
                  <span class="run-time">--</span>
                </div>
              </div>
            </div>

            <!-- 常用示例 -->
            <div class="schedule-examples">
              <span class="examples-title"
                >{{ t("schedule.commonExamples") }}:</span
              >
              <div class="examples-list">
                <button
                  class="example-btn"
                  @click="
                    cronInput = '0 * * * * ?';
                    handleCronInput('0 * * * * ?');
                  "
                >
                  {{ t("schedule.everyMinute") }}
                </button>
                <button
                  class="example-btn"
                  @click="
                    cronInput = '0 */5 * * * ?';
                    handleCronInput('0 */5 * * * ?');
                  "
                >
                  {{ t("schedule.every5Minutes") }}
                </button>
                <button
                  class="example-btn"
                  @click="
                    cronInput = '0 */30 * * * ?';
                    handleCronInput('0 */30 * * * ?');
                  "
                >
                  {{ t("schedule.every30Minutes") }}
                </button>
                <button
                  class="example-btn"
                  @click="
                    cronInput = '0 0 * * * ?';
                    handleCronInput('0 0 * * * ?');
                  "
                >
                  {{ t("schedule.everyHour") }}
                </button>
                <button
                  class="example-btn"
                  @click="
                    cronInput = '0 0 2 * * ?';
                    handleCronInput('0 0 2 * * ?');
                  "
                >
                  {{ t("schedule.everyDayAt2AM") }}
                </button>
                <button
                  class="example-btn"
                  @click="
                    cronInput = '0 0 9 ? * 2';
                    handleCronInput('0 0 9 ? * 2');
                  "
                >
                  {{ t("schedule.everyMonday9AM") }}
                </button>
                <button
                  class="example-btn"
                  @click="
                    cronInput = '0 30 9 ? * 2-6';
                    handleCronInput('0 30 9 ? * 2-6');
                  "
                >
                  {{ t("schedule.workdayAt930") }}
                </button>
                <button
                  class="example-btn"
                  @click="
                    cronInput = '0 0 0 1 * ?';
                    handleCronInput('0 0 0 1 * ?');
                  "
                >
                  {{ t("schedule.everyMonthFirstDay") }}
                </button>
              </div>
            </div>
          </div>

          <div class="schedule-modal-footer">
            <button class="cancel-btn" @click="showSchedulePanel = false">
              {{ t("common.cancel") }}
            </button>
            <button
              class="save-btn"
              :disabled="cronError || scheduleLoading"
              @click="saveScheduleConfig"
            >
              {{ t("common.save") }}
            </button>
          </div>
        </div>
      </div>

      <div class="steps-section">
        <div class="section-header">
          <span class="section-title">{{ t("orchestration.steps") }}</span>
          <button class="add-step-btn" @click="openApiSelector">
            <Icon name="add" :size="14" />
            <span>{{ t("orchestration.addApi") }}</span>
          </button>
        </div>

        <div class="steps-list">
          <div v-if="steps.length === 0" class="empty-steps">
            {{ t("orchestration.apiSelector.noApis") }}
          </div>

          <div
            v-for="(step, index) in steps"
            :key="step.id"
            :data-step-id="step.id"
            class="step-item"
            :class="{
              dragging: dragState.draggingId === step.id,
              'dragover-before':
                dragState.dragOverId === step.id &&
                dragState.dragOverPosition === 'before',
              'dragover-after':
                dragState.dragOverId === step.id &&
                dragState.dragOverPosition === 'after',
              disabled: !step.enabled,
              running: currentStepIndex === index && isRunning,
              success: getStepRunStatus(step.id) === 'success',
              failed: getStepRunStatus(step.id) === 'failed',
              skipped: getStepRunStatus(step.id) === 'skipped',
            }"
            @mousedown="(e) => onMouseDown(e, step)"
            @contextmenu.prevent="(e) => openStepContextMenu(e, step)"
          >
            <div class="step-index">
              <span
                v-if="currentStepIndex === index && isRunning"
                class="running-indicator"
              ></span>
              <span v-else>{{ index + 1 }}</span>
            </div>
            <div class="step-info">
              <div class="step-name">
                {{ step.name || getApiInfo(step.api_id)?.name }}
              </div>
              <div class="step-api">
                <span
                  class="method-tag"
                  :class="getMethodClass(getApiInfo(step.api_id)?.method)"
                >
                  {{ getApiInfo(step.api_id)?.method }}
                </span>
                <span class="api-url">{{ getApiInfo(step.api_id)?.url }}</span>
              </div>
            </div>
            <div class="step-config">
              <span class="config-item" v-if="step.wait_before > 0">
                {{ t("orchestration.stepConfig.waitBefore") }}:
                {{ step.wait_before }}ms
              </span>
              <span class="config-item" v-if="step.retry_count > 0">
                {{ t("orchestration.stepConfig.retryCount") }}:
                {{ step.retry_count }}
              </span>
            </div>
            <div class="step-actions">
              <button
                class="config-btn"
                @mousedown.stop
                @click.stop="openStepConfig(step)"
              >
                <Icon name="settings" :size="14" />
              </button>
              <button
                class="delete-btn"
                @mousedown.stop
                @click.stop="removeStep(step.id)"
              >
                <Icon name="delete" :size="14" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="history-section">
        <div class="section-header">
          <span class="section-title">{{ t("orchestration.runHistory") }}</span>
          <button
            v-if="runHistory.length > 0"
            class="clear-history-btn"
            @click="clearAllRuns"
          >
            {{ t("buttons.clearAll") }}
          </button>
        </div>

        <div class="history-list">
          <div v-if="runHistory.length === 0" class="empty-history">
            {{ t("orchestration.noRunHistory") }}
          </div>

          <RecycleScroller
            v-else
            class="scroller"
            :items="runHistory"
            :item-size="84"
            key-field="id"
            :buffer="200"
          >
            <template #default="{ item: run }">
              <div class="history-item" @click="openRunDetail(run)">
                <div class="history-header">
                  <span class="history-status" :class="run.status">
                    {{ t(`orchestration.runStatus.${run.status}`) }}
                  </span>
                  <span class="history-time">{{
                    formatTime(run.start_time)
                  }}</span>
                </div>
                <div class="history-footer">
                  <div class="history-stats">
                    <span class="stat success">
                      {{ run.success_count }} {{ t("tests.passed") }}
                    </span>
                    <span class="stat failed" v-if="run.failed_count > 0">
                      {{ run.failed_count }} {{ t("tests.failed") }}
                    </span>
                    <span class="stat skipped" v-if="run.skipped_count > 0">
                      {{ run.skipped_count }}
                      {{ t("orchestration.stepStatus.skipped") }}
                    </span>
                  </div>
                  <span class="history-time-total">{{
                    getTotalTime(run)
                  }}</span>
                  <button
                    class="history-delete-btn"
                    @click.stop="deleteRun(run.id)"
                  >
                    <Icon name="delete" :size="14" />
                  </button>
                </div>
              </div>
            </template>
          </RecycleScroller>
        </div>
      </div>
    </div>

    <div
      v-if="showApiSelector"
      class="modal-overlay"
      @click="showApiSelector = false"
    >
      <div class="modal-content api-selector" @click.stop>
        <div class="modal-header">
          <span class="modal-title">{{
            t("orchestration.apiSelector.title")
          }}</span>
          <button class="close-btn" @click="showApiSelector = false">×</button>
        </div>
        <div class="modal-body">
          <input
            v-model="searchQuery"
            class="search-input"
            :placeholder="t('orchestration.apiSelector.searchPlaceholder')"
          />
          <div class="api-list">
            <div v-if="filteredApis.length === 0" class="empty-apis">
              {{ t("orchestration.apiSelector.noApis") }}
            </div>
            <div
              v-for="api in filteredApis"
              :key="api.id"
              class="api-item"
              @click="selectApi(api)"
            >
              <span class="method-tag" :class="getMethodClass(api.method)">
                {{ api.method }}
              </span>
              <span class="api-name">{{ api.name }}</span>
              <span class="api-url">{{ api.url }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="showStepConfig"
      class="modal-overlay"
      @click="showStepConfig = false"
    >
      <div class="modal-content step-config-modal" @click.stop>
        <div class="modal-header">
          <span class="modal-title">{{
            t("orchestration.stepConfig.title")
          }}</span>
          <button class="close-btn" @click="showStepConfig = false">×</button>
        </div>
        <div class="modal-body">
          <div class="config-field">
            <label>{{ t("orchestration.stepConfig.stepName") }}</label>
            <input v-model="stepConfigData.name" class="config-input" />
          </div>
          <div class="config-field">
            <label>{{ t("common.enabled") }}</label>
            <input
              type="checkbox"
              v-model="stepConfigData.enabled"
              class="config-checkbox"
            />
          </div>
          <div class="config-field">
            <label>{{ t("orchestration.stepConfig.waitBefore") }} (ms)</label>
            <input
              type="number"
              v-model="stepConfigData.wait_before"
              class="config-input"
              min="0"
            />
          </div>
          <div class="config-field">
            <label>{{ t("orchestration.stepConfig.retryCount") }}</label>
            <input
              type="number"
              v-model="stepConfigData.retry_count"
              class="config-input"
              min="0"
            />
          </div>
          <div class="config-field">
            <label>{{ t("orchestration.stepConfig.retryDelay") }} (ms)</label>
            <input
              type="number"
              v-model="stepConfigData.retry_delay"
              class="config-input"
              min="0"
            />
          </div>
          <div class="config-field">
            <label>{{ t("orchestration.stepConfig.onFailure") }}</label>
            <div class="radio-group">
              <label class="radio-item">
                <input
                  type="radio"
                  v-model="stepConfigData.on_failure"
                  value="stop"
                />
                <span>{{ t("orchestration.stepConfig.stopExecution") }}</span>
              </label>
              <label class="radio-item">
                <input
                  type="radio"
                  v-model="stepConfigData.on_failure"
                  value="continue"
                />
                <span>{{
                  t("orchestration.stepConfig.continueExecution")
                }}</span>
              </label>
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button class="cancel-btn" @click="showStepConfig = false">
            {{ t("common.cancel") }}
          </button>
          <button class="save-btn" @click="saveStepConfig">
            {{ t("common.save") }}
          </button>
        </div>
      </div>
    </div>

    <RunDetailModal
      v-if="showRunDetail"
      :runDetail="runDetailData"
      @close="closeRunDetail"
    />

    <div
      v-if="stepContextMenu.visible"
      class="step-context-overlay"
      @click="closeStepContextMenu"
      @contextmenu.prevent
    ></div>
    <ContextMenu
      v-if="stepContextMenu.visible"
      :visible="stepContextMenu.visible"
      :x="stepContextMenu.x"
      :y="stepContextMenu.y"
      :items="[
        {
          label: t('common.delete'),
          action: 'delete',
          icon: 'delete',
          danger: true,
        },
      ]"
      @action="handleStepContextAction"
      @close="closeStepContextMenu"
    />
  </div>
</template>

<style scoped src="./style.css"></style>
<style scoped src="./schedule-modal.css"></style>
