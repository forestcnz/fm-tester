<script setup>
import { useI18n } from "vue-i18n";
import { useHistoryPanelSetup } from "./index.js";
import Icon from "../../Icon/index.vue";

const { t } = useI18n();

const props = defineProps({
  workspace: Object,
});

const emit = defineEmits(["selectHistory"]);

// 使用 composable
const {
  dates,
  historyByDate,
  loading,
  toggleDateExpand,
  isDateExpanded,
  selectHistory,
  deleteHistoryEntry,
  clearDateHistory,
  clearAllHistory,
  loadMore,
  hasMoreByDate,
  getMethodClass,
  formatDateDisplay,
} = useHistoryPanelSetup(props, emit);
</script>

<template>
  <div class="history-panel">
    <!-- 面板头部 -->
    <div class="panel-header">
      <span class="panel-title">{{ t("panels.history") }}</span>
      <div class="panel-actions">
        <span
          class="action-btn"
          :title="t('buttons.clearAll')"
          @click="clearAllHistory"
          v-if="dates.length > 0"
        >
          <Icon name="delete" :size="14" />
        </span>
      </div>
    </div>

    <!-- 提示：需要先选择工作区 -->
    <div v-if="!props.workspace" class="empty-panel">
      {{ t("empty.selectWorkspace") }}
    </div>

    <!-- 加载状态 -->
    <div v-else-if="loading" class="loading-panel">
      {{ t("common.loading") }}
    </div>

    <!-- 空状态 -->
    <div v-else-if="dates.length === 0" class="empty-panel">
      {{ t("empty.noHistory") }}
    </div>

    <!-- 日期分组列表 -->
    <div v-else class="date-list">
      <div v-for="date in dates" :key="date" class="date-group">
        <!-- 日期头部 -->
        <div class="date-header" @click="toggleDateExpand(date)">
          <span class="expand-icon">
            <Icon
              :name="isDateExpanded(date) ? 'arrow-down' : 'arrow-right'"
              :size="12"
            />
          </span>
          <span class="date-label">{{ formatDateDisplay(date) }}</span>
          <span
            class="clear-btn"
            :title="t('buttons.clearToday')"
            @click.stop="clearDateHistory(date)"
          >
            <Icon name="delete" :size="12" />
          </span>
        </div>

        <!-- 历史记录列表 -->
        <div v-if="isDateExpanded(date)" class="history-list">
          <RecycleScroller
            class="scroller"
            :items="historyByDate[date] || []"
            :item-size="40"
            key-field="id"
            :buffer="200"
          >
            <template #default="{ item }">
              <div class="history-item" @click="selectHistory(item)">
                <span class="method-tag" :class="getMethodClass(item.method)">
                  <template v-if="item.method === 'WebSocket'"> WS </template>
                  <template v-else>
                    {{ item.method }}
                  </template>
                </span>
                <span class="item-url" :title="item.url">{{ item.url }}</span>
                <span
                  class="delete-btn"
                  :title="t('common.delete')"
                  @click.stop="deleteHistoryEntry(date, item.id)"
                >
                  <Icon name="delete" :size="12" />
                </span>
              </div>
            </template>
          </RecycleScroller>

          <!-- 加载更多按钮 -->
          <div
            v-if="hasMoreByDate[date]"
            class="load-more-btn"
            @click="loadMore(date)"
          >
            {{ t("common.loadMore") || "加载更多" }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
