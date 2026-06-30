<script setup>
import { useWorkspaceImportSetup } from "./index.js";
import Icon from "../Icon/index.vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

const props = defineProps({
  visible: Boolean,
});

const emit = defineEmits(["close", "imported"]);

const {
  selectedFile,
  preview,
  error,
  loading,
  selectFile,
  confirmImport,
  close,
} = useWorkspaceImportSetup(props, emit);
</script>

<template>
  <div v-if="visible" class="dialog-overlay" @click.self="close">
    <div class="dialog workspace-import-dialog">
      <div class="dialog-header">
        <span class="dialog-title">{{ t("workspace.import") }}</span>
        <span class="dialog-close" @click="close">×</span>
      </div>

      <div class="dialog-body">
        <div class="form-group file-group">
          <button class="btn-select-file" @click="selectFile">
            <Icon name="folder" :size="16" />
            <span v-if="selectedFile" class="file-name">{{
              selectedFile
            }}</span>
            <span v-else>{{ t("workspace.selectFile") }}</span>
          </button>
        </div>

        <div v-if="preview" class="preview-section">
          <div class="preview-header">{{ t("workspace.importPreview") }}</div>

          <div class="preview-content">
            <div class="preview-item">
              <span class="preview-label">{{ t("common.name") }}:</span>
              <span class="preview-value">{{ preview.name }}</span>
            </div>

            <div class="preview-item">
              <span class="preview-label"
                >{{ t("workspace.exportedAt") }}:</span
              >
              <span class="preview-value">{{ preview.exported_at }}</span>
            </div>

            <div class="preview-stats">
              <div class="stats-header">{{ t("workspace.dataStats") }}</div>
              <div class="stats-grid">
                <div class="stat-item">
                  <span class="stat-label"
                    >{{ t("sidebar.environments") }}:</span
                  >
                  <span class="stat-value">{{
                    preview.stats.environments
                  }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label"
                    >{{ t("sidebar.collections") }}:</span
                  >
                  <span class="stat-value">{{
                    preview.stats.collections
                  }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">API:</span>
                  <span class="stat-value">{{ preview.stats.apis }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">{{ t("tabs.scripts") }}:</span>
                  <span class="stat-value">{{ preview.stats.scripts }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">{{ t("sidebar.history") }}:</span>
                  <span class="stat-value">{{
                    preview.stats.history_entries
                  }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-if="error" class="error-msg">{{ error }}</div>
      </div>

      <div class="dialog-footer">
        <button class="btn-secondary" @click="close">
          {{ t("common.cancel") }}
        </button>
        <button
          class="btn-primary"
          :disabled="loading || !preview"
          @click="confirmImport"
        >
          {{ loading ? t("common.loading") : t("buttons.import") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
