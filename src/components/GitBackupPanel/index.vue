<script setup>
import { useGitBackupSetup } from "./index.js";

const props = defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(["close"]);

const {
  t,
  loading,
  testing,
  repoUrl,
  branch,
  username,
  password,
  passwordPlaceholder,
  testConnection,
  saveSettings,
  close,
} = useGitBackupSetup(props, emit);
</script>

<template>
  <div v-if="visible" class="git-backup-panel">
    <div class="settings-header">
      <span class="settings-title">{{ t("gitBackup.title") }}</span>
      <button class="close-btn" @click="close">✕</button>
    </div>

    <div class="settings-content">
      <div class="settings-section">
        <div class="setting-item">
          <span class="setting-label">{{ t("gitBackup.repoUrl") }}</span>
          <input
            type="text"
            class="setting-input full-width"
            v-model="repoUrl"
            :disabled="loading"
            :placeholder="t('gitBackup.repoUrlPlaceholder')"
          />
        </div>
        <div class="setting-description">
          {{ t("gitBackup.repoUrlDesc") }}
        </div>

        <div class="setting-item">
          <span class="setting-label">{{ t("gitConfig.branch") }}</span>
          <input
            type="text"
            class="setting-input full-width"
            v-model="branch"
            :disabled="loading"
            placeholder="master"
          />
        </div>

        <div class="setting-item">
          <span class="setting-label">{{ t("git.username") }}</span>
          <input
            type="text"
            class="setting-input full-width"
            v-model="username"
            :disabled="loading"
            :placeholder="t('git.usernamePlaceholder')"
          />
        </div>

        <div class="setting-item">
          <span class="setting-label">{{ t("git.password") }}</span>
          <input
            type="password"
            class="setting-input full-width"
            v-model="password"
            :disabled="loading"
            :placeholder="passwordPlaceholder"
          />
        </div>
        <div class="setting-description">
          {{ t("gitBackup.passwordDesc") }}
        </div>
      </div>
    </div>

    <div class="settings-footer">
      <button
        class="test-btn"
        :disabled="loading || testing || !repoUrl"
        @click="testConnection"
      >
        {{ testing ? t("common.loading") : t("gitBackup.testConnection") }}
      </button>
      <button class="cancel-btn" @click="close">
        {{ t("common.cancel") }}
      </button>
      <button class="save-btn" :disabled="loading" @click="saveSettings">
        {{ t("common.save") }}
      </button>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
