<script setup>
import { useGitBackupRestoreSetup } from "./index.js";

const props = defineProps({
  visible: Boolean,
  targetWorkspace: {
    type: Object,
    default: null,
  },
});

const emit = defineEmits(["close", "imported"]);

const {
  t,
  loading,
  restoring,
  deleting,
  backups,
  selected,
  newName,
  error,
  isOverwrite,
  formatSize,
  formatTimestamp,
  selectBackup,
  confirmRestore,
  deleteBackup,
  close,
} = useGitBackupRestoreSetup(props, emit);
</script>

<template>
  <div v-if="visible" class="dialog-overlay" @click.self="close">
    <div class="dialog git-restore-dialog">
      <div class="dialog-header">
        <span class="dialog-title">{{
          isOverwrite
            ? t("gitBackup.overwriteTitle")
            : t("gitBackup.restoreTitle")
        }}</span>
        <span
          class="dialog-close"
          :class="{ disabled: restoring || deleting }"
          @click="close"
          >×</span
        >
      </div>

      <div class="dialog-body">
        <div v-if="loading || restoring || deleting" class="loading-tip">
          {{
            restoring
              ? t("gitBackup.restoring")
              : deleting
                ? t("common.loading")
                : t("common.loading")
          }}
        </div>
        <template v-else>
          <div v-if="backups.length === 0" class="empty-tip">
            {{ error || t("gitBackup.noBackups") }}
          </div>
          <div v-else class="backup-list">
            <div
              v-for="item in backups"
              :key="item.file_name"
              class="backup-item"
              :class="{
                active: selected && selected.file_name === item.file_name,
              }"
              @click="selectBackup(item)"
            >
              <div class="backup-info">
                <div class="backup-name">{{ item.workspace_name }}</div>
                <div class="backup-time">
                  {{ formatTimestamp(item.timestamp) }}
                </div>
              </div>
              <div class="backup-size">{{ formatSize(item.size) }}</div>
              <button
                class="backup-delete-btn"
                :disabled="deleting"
                @click.stop="deleteBackup(item)"
              >
                ✕
              </button>
            </div>
          </div>

          <div v-if="isOverwrite && selected" class="overwrite-warn">
            {{ t("gitBackup.overwriteWarn", { name: targetWorkspace.name }) }}
          </div>

          <div v-if="selected && !isOverwrite" class="new-name-group">
            <span class="new-name-label">{{ t("gitBackup.newName") }}</span>
            <input
              type="text"
              class="new-name-input"
              v-model="newName"
              :placeholder="t('gitBackup.newNamePlaceholder')"
            />
            <div class="new-name-desc">{{ t("gitBackup.newNameDesc") }}</div>
          </div>

          <div v-if="error && backups.length > 0" class="error-msg">
            {{ error }}
          </div>
        </template>
      </div>

      <div class="dialog-footer">
        <button
          class="btn-secondary"
          :disabled="restoring || deleting"
          @click="close"
        >
          {{ t("common.cancel") }}
        </button>
        <button
          class="btn-primary"
          :disabled="!selected || restoring"
          @click="confirmRestore"
        >
          {{
            restoring
              ? t("gitBackup.restoring")
              : isOverwrite
                ? t("gitBackup.confirmOverwrite")
                : t("buttons.import")
          }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
