<script setup>
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const version = "v0.1.3";

defineProps({
  workspaceName: {
    type: String,
    default: "",
  },
});

defineEmits(["open-cookie-panel", "open-console-panel"]);
</script>

<template>
  <div class="status-bar">
    <div class="status-left">
      <span class="version">{{ version }}</span>
    </div>
    <div class="status-center">
      <span class="workspace-name" v-if="workspaceName">{{
        workspaceName
      }}</span>
    </div>
    <div class="status-right">
      <div
        class="status-icon"
        title="Cookies"
        @click="$emit('open-cookie-panel')"
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path
            d="M21 12a9 9 0 1 1-9-9 4 4 0 0 0 4 4 4 4 0 0 0 4 4 1 1 0 0 0 1 1z"
          />
          <circle cx="9" cy="10" r="0.9" fill="currentColor" stroke="none" />
          <circle cx="13.5" cy="14" r="0.9" fill="currentColor" stroke="none" />
          <circle
            cx="8.5"
            cy="14.5"
            r="0.9"
            fill="currentColor"
            stroke="none"
          />
        </svg>
      </div>
      <div
        class="status-icon"
        :title="t('panels.console')"
        @click="$emit('open-console-panel')"
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="2" y="3" width="20" height="14" rx="2" />
          <path d="M6 8l3 3-3 3M12 14h4" />
        </svg>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
