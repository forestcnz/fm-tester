<script setup>
import { useI18n } from "vue-i18n";
import { useWorkspacePanelSetup } from "./index.js";
import Icon from "../../Icon/index.vue";
import WorkspaceImportDialog from "../../WorkspaceImportDialog/index.vue";
import GitBackupRestoreDialog from "../../GitBackupRestoreDialog/index.vue";

const { t } = useI18n();

const props = defineProps({
  workspace: Object,
});

const emit = defineEmits([
  "selectWorkspace",
  "createWorkspace",
  "workspaceDeleted",
  "workspaceUpdated",
  "workspaceImported",
]);

const {
  workspaces,
  currentWorkspace,
  wsContextMenu,
  blankContextMenu,
  dragState,
  showImportDialog,
  restoreDialog,
  backingUp,
  renamingId,
  renameValue,
  confirmRename,
  cancelRename,
  loadWorkspaces,
  selectWorkspace,
  createWorkspace,
  openImportDialog,
  openWsContextMenu,
  handleWsContextAction,
  openBlankContextMenu,
  handleBlankContextAction,
  onMouseDown,
  handleImported,
  handleRestoreImported,
} = useWorkspacePanelSetup(props, emit);

defineExpose({
  loadWorkspaces,
});
</script>

<template>
  <div class="workspace-panel">
    <!-- 备份遮罩 -->
    <div v-if="backingUp" class="backup-overlay">
      <div class="backup-loading">{{ t("gitBackup.backingUp") }}</div>
    </div>

    <!-- 面板头部 -->
    <div class="panel-header">
      <span class="panel-title">{{ t("panels.workspaces") }}</span>
      <div class="panel-actions">
        <span
          class="action-btn import-btn"
          :title="t('workspace.import')"
          @click="openImportDialog"
        >
          <Icon name="import" :size="14" />
        </span>
        <span
          class="action-btn"
          :title="t('buttons.newWorkspace')"
          @click="createWorkspace"
        >
          <Icon name="add" :size="14" />
        </span>
      </div>
    </div>

    <!-- 工作区列表 -->
    <div class="env-list" @contextmenu.prevent="openBlankContextMenu">
      <div
        v-for="ws in workspaces"
        :key="ws.id"
        :data-item-id="ws.id"
        class="env-item"
        :class="{
          active: currentWorkspace?.id === ws.id,
          dragging: dragState.draggingId === ws.id,
          'drag-over-before':
            dragState.dragOverId === ws.id &&
            dragState.dragPosition === 'before',
          'drag-over-after':
            dragState.dragOverId === ws.id &&
            dragState.dragPosition === 'after',
        }"
        @mousedown="(e) => onMouseDown(e, ws)"
        @click="selectWorkspace(ws)"
        @contextmenu.prevent="(e) => openWsContextMenu(e, ws)"
      >
        <div class="env-header">
          <span class="env-icon">
            <Icon name="ws" />
          </span>
          <input
            v-if="renamingId === ws.id"
            v-model="renameValue"
            class="env-name ws-rename-input"
            @click.stop
            @mousedown.stop
            @keyup.enter="confirmRename(ws)"
            @keyup.escape="cancelRename"
            @blur="confirmRename(ws)"
          />
          <span v-else class="env-name">{{ ws.name }}</span>
        </div>
      </div>

      <div v-if="workspaces.length === 0" class="empty-panel">
        {{ t("empty.noWorkspaces") }}
      </div>
    </div>

    <!-- 工作区右键菜单 -->
    <div
      v-if="wsContextMenu.visible"
      class="context-menu"
      :style="{ left: wsContextMenu.x + 'px', top: wsContextMenu.y + 'px' }"
    >
      <div class="menu-item" @click="handleWsContextAction('export-ws')">
        <Icon name="export" :size="16" />
        {{ t("workspace.export") }}
      </div>
      <div class="menu-item" @click="handleWsContextAction('rename-ws')">
        <Icon name="edit" :size="16" />
        {{ t("contextMenu.renameWorkspace") }}
      </div>
      <div class="menu-divider"></div>
      <div
        class="menu-item"
        :class="{ disabled: backingUp }"
        @click="handleWsContextAction('backup-ws')"
      >
        <Icon name="push" :size="16" />
        {{ t("gitBackup.backupToGit") }}
      </div>
      <div class="menu-item" @click="handleWsContextAction('restore-ws')">
        <Icon name="pull" :size="16" />
        {{ t("gitBackup.restoreFromGit") }}
      </div>
      <div class="menu-divider"></div>
      <div class="menu-item delete" @click="handleWsContextAction('delete-ws')">
        <Icon name="delete" :size="16" />
        {{ t("contextMenu.deleteWorkspace") }}
      </div>
    </div>

    <!-- 空白处右键菜单 -->
    <div
      v-if="blankContextMenu.visible"
      class="context-menu"
      :style="{
        left: blankContextMenu.x + 'px',
        top: blankContextMenu.y + 'px',
      }"
    >
      <div class="menu-item" @click="handleBlankContextAction('restore-ws')">
        <Icon name="pull" :size="16" />
        {{ t("gitBackup.restoreFromGit") }}
      </div>
    </div>

    <!-- 导入对话框 -->
    <WorkspaceImportDialog
      :visible="showImportDialog"
      @close="showImportDialog = false"
      @imported="handleImported"
    />

    <!-- Git 恢复对话框（targetWorkspace 非空时为覆盖模式） -->
    <GitBackupRestoreDialog
      :visible="restoreDialog.visible"
      :target-workspace="restoreDialog.targetWorkspace"
      @close="restoreDialog.visible = false"
      @imported="handleRestoreImported"
    />
  </div>
</template>

<style src="./style.css" scoped></style>
