<script setup>
import { useI18n } from "vue-i18n";
import { useCookiePanel } from "./index.js";
import { useDialogEscape } from "../../composables/useDialogStack.js";

const props = defineProps({
  visible: Boolean,
  cookies: Array,
  workspaceId: String,
});

const emit = defineEmits(["close", "refresh"]);

const { t } = useI18n();

const { deleteCookie, clearCookies } = useCookiePanel(props, emit);

// ESC 键关闭
useDialogEscape(
  () => props.visible,
  () => emit("close"),
);
</script>

<template>
  <div class="cookie-panel-overlay" v-if="visible" @click="$emit('close')">
    <div class="cookie-panel" @click.stop>
      <div class="panel-header">
        <h3>Cookies</h3>
        <button class="close-btn" @click="$emit('close')">×</button>
      </div>
      <div class="panel-actions">
        <button class="clear-btn" @click="clearCookies">
          {{ t("buttons.clearAll") }}
        </button>
      </div>
      <div class="panel-content">
        <div v-if="cookies.length === 0" class="empty-message">
          {{ t("empty.noCookies") }}
        </div>
        <div v-else class="cookie-list">
          <div
            class="cookie-item"
            v-for="cookie in cookies"
            :key="cookie.name + cookie.domain"
          >
            <div class="cookie-info">
              <span class="cookie-name">{{ cookie.name }}</span>
              <span class="cookie-domain">{{ cookie.domain }}</span>
            </div>
            <div class="cookie-value">{{ cookie.value }}</div>
            <div class="cookie-actions">
              <button
                class="delete-btn"
                @click="deleteCookie(cookie.name, cookie.domain)"
              >
                {{ t("common.delete") }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
