<script setup>
import { useWorkspaceSettingsSetup } from "./index.js";
import Icon from "../Icon/index.vue";
import ScriptPanel from "../ScriptPanel/index.vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

const props = defineProps({
  workspace: {
    type: Object,
    required: true,
  },
  workspaceId: {
    type: String,
    required: true,
  },
});

const emit = defineEmits(["save", "close"]);

const { localSettings, handleScriptUpdate, saveSettings } =
  useWorkspaceSettingsSetup(props, emit);
</script>

<template>
  <div class="workspace-settings-panel">
    <!-- 工作区名称 -->
    <div class="settings-header">
      <span class="workspace-icon">
        <Icon name="workspace" :size="16" />
      </span>
      <span class="workspace-name">{{ localSettings.name }}</span>
      <span v-if="workspace.last_backup_at" class="workspace-backup-time">
        · {{ t("gitBackup.lastBackupAt") }} {{ workspace.last_backup_at }}
      </span>
    </div>

    <!-- 脚本面板 -->
    <div class="scripts-panel">
      <ScriptPanel
        :request="localSettings"
        @update:request="handleScriptUpdate"
        @save="saveSettings"
      />
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
