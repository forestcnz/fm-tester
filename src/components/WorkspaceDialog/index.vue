<script setup>
import { useI18n } from "vue-i18n";
import { useWorkspaceDialogSetup } from "./index.js";

const { t } = useI18n();

const props = defineProps({
  visible: Boolean,
  mode: String, // 'create' or 'select'
});

const emit = defineEmits(["close", "created", "selected"]);

const {
  name,
  description,
  error,
  loading,
  workspaces,
  selectedWorkspaceId,
  createWorkspace,
  handleSelectWorkspaceItem,
  selectWorkspace,
  close,
} = useWorkspaceDialogSetup(props, emit);
</script>

<template>
  <div v-if="visible" class="dialog-overlay" @click.self="close">
    <div class="dialog">
      <div class="dialog-header">
        <span class="dialog-title">
          {{
            mode === "create"
              ? t("dialogs.newWorkspace")
              : t("dialogs.switchWorkspace")
          }}
        </span>
        <span class="dialog-close" @click="close">×</span>
      </div>

      <div class="dialog-body">
        <!-- 创建模式 -->
        <div v-if="mode === 'create'" class="create-form">
          <div class="form-group">
            <label>{{ t("common.name") }}</label>
            <input
              v-model="name"
              type="text"
              :placeholder="t('placeholder.workspaceName')"
              class="form-input"
            />
          </div>

          <div class="form-group">
            <label>{{ t("common.description") }}</label>
            <input
              v-model="description"
              type="text"
              :placeholder="t('placeholder.workspaceDesc')"
              class="form-input"
            />
          </div>
        </div>

        <!-- 选择模式 -->
        <div v-if="mode === 'select'" class="select-list">
          <div
            v-for="ws in workspaces"
            :key="ws.id"
            class="workspace-item"
            :class="{ selected: selectedWorkspaceId === ws.id }"
            @click="handleSelectWorkspaceItem(ws.id)"
          >
            <div class="ws-header">
              <span class="ws-name">{{ ws.name }}</span>
            </div>
            <div class="ws-desc">{{ ws.description }}</div>
            <div class="ws-time">
              {{ t("workspace.lastOpened") }} {{ ws.last_opened }}
            </div>
          </div>

          <div v-if="workspaces.length === 0" class="empty-tip">
            {{ t("empty.noWorkspaces") }}
          </div>
        </div>

        <!-- 错误提示 -->
        <div v-if="error" class="error-msg">{{ error }}</div>
      </div>

      <div class="dialog-footer">
        <button
          v-if="mode === 'create'"
          class="btn-primary"
          :disabled="loading"
          @click="createWorkspace"
        >
          {{ loading ? t("common.loading") : t("common.new") }}
        </button>

        <button
          v-if="mode === 'select'"
          class="btn-primary"
          :disabled="loading"
          @click="selectWorkspace"
        >
          {{ loading ? t("common.loading") : t("common.confirm") }}
        </button>

        <button class="btn-secondary" @click="close">
          {{ t("common.cancel") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
