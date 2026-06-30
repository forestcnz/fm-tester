<template>
  <div class="environment-panel">
    <!-- 面板头部 -->
    <div class="panel-header">
      <span class="panel-title">{{ t("panels.environments") }}</span>
      <div class="panel-actions">
        <span
          class="action-btn"
          :title="t('buttons.newEnvironment')"
          @click="openCreateEnvDialog"
        >
          <Icon name="add" :size="14" />
        </span>
      </div>
    </div>

    <!-- 提示：需要先选择工作区 -->
    <div v-if="!workspace" class="empty-panel">
      {{ t("empty.selectWorkspace") }}
    </div>

    <!-- 环境列表 -->
    <div
      v-else
      class="env-list"
      @contextmenu="(e) => openEnvContextMenu(e, null, 'root')"
    >
      <div
        v-for="env in environments"
        :key="env.id"
        :data-item-id="env.id"
        class="env-item"
        :class="{
          active: activeEnvironmentId === env.id,
          dragging: dragState.draggingId === env.id,
          'drag-over-before':
            dragState.dragOverId === env.id &&
            dragState.dragPosition === 'before',
          'drag-over-after':
            dragState.dragOverId === env.id &&
            dragState.dragPosition === 'after',
        }"
        @mousedown="(e) => onMouseDown(e, env)"
        @click="selectEnvironment(env.id)"
        @contextmenu.prevent="(e) => openEnvContextMenu(e, env, 'env')"
      >
        <div class="env-header">
          <span class="env-icon"><Icon name="environment" /></span>
          <span class="env-name">{{ env.name }}</span>
        </div>
      </div>

      <div v-if="environments.length === 0" class="empty-panel">
        {{ t("empty.noEnvironments") }}
      </div>
    </div>

    <!-- 环境编辑对话框 -->
    <div
      v-if="showEnvDialog"
      class="create-dialog-overlay"
      @click.self="showEnvDialog = false"
    >
      <div class="create-dialog env-dialog">
        <div class="dialog-header">
          <span>{{
            editingEnv
              ? t("dialogs.editEnvironment")
              : t("dialogs.newEnvironment")
          }}</span>
          <span class="dialog-close" @click="showEnvDialog = false">×</span>
        </div>

        <div class="dialog-body">
          <div class="dialog-row">
            <label>{{ t("common.name") }}</label>
            <input
              v-model="editingEnvName"
              type="text"
              :placeholder="t('placeholder.environmentName')"
            />
          </div>
        </div>

        <div class="dialog-footer">
          <button class="btn-cancel" @click="showEnvDialog = false">
            {{ t("common.cancel") }}
          </button>
          <button class="btn-confirm" @click="handleSaveEnv">
            {{ t("common.save") }}
          </button>
        </div>
      </div>
    </div>

    <!-- 环境右键菜单 -->
    <div
      v-if="envContextMenu.visible"
      class="context-menu"
      :style="{ left: envContextMenu.x + 'px', top: envContextMenu.y + 'px' }"
      @click.stop
    >
      <!-- 根级别菜单 -->
      <template v-if="envContextMenu.type === 'root'">
        <div class="menu-item" @click="handleEnvContextAction('new-env')">
          <span class="menu-icon"><Icon name="environment" :size="14" /></span>
          <span>{{ t("buttons.newEnvironment") }}</span>
        </div>
      </template>

      <!-- 环境菜单 -->
      <template v-if="envContextMenu.type === 'env'">
        <div class="menu-item" @click="handleEnvContextAction('edit-env')">
          <span class="menu-icon"><Icon name="edit" :size="14" /></span>
          <span>{{ t("common.rename") }}</span>
        </div>
        <div class="menu-divider"></div>
        <div
          class="menu-item delete"
          @click="handleEnvContextAction('delete-env')"
        >
          <span class="menu-icon"><Icon name="delete" :size="14" /></span>
          <span>{{ t("contextMenu.deleteEnvironment") }}</span>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup>
import { useI18n } from "vue-i18n";
import { useEnvironmentPanelSetup } from "./index.js";
import Icon from "../../Icon/index.vue";

const { t } = useI18n();

const props = defineProps({
  workspace: {
    type: Object,
    default: null,
  },
});

const emit = defineEmits(["selectEnvironment", "environmentUpdated"]);

const {
  environments,
  activeEnvironmentId,
  showEnvDialog,
  editingEnv,
  editingEnvName,
  envContextMenu,
  dragState,
  loadEnvironments,
  selectEnvironment,
  openCreateEnvDialog,
  handleSaveEnv,
  openEnvContextMenu,
  handleEnvContextAction,
  onMouseDown,
} = useEnvironmentPanelSetup(props, emit);

// 暴露方法供父组件调用
defineExpose({
  loadEnvironments,
});
</script>

<style scoped src="./style.css"></style>
