<script setup>
import { useAiSettingsSetup } from "./index.js";
import "./style.css";

const props = defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(["close", "saved"]);

const {
  t,
  aiApiEndpoint,
  aiApiKey,
  apiKeyPlaceholder,
  hasApiKey,
  aiModel,
  aiModels,
  loadingModels,
  showModelDropdown,
  aiTimeout,
  customHeaders,
  fetchModels,
  selectModel,
  toggleModelDropdown,
  closeModelDropdown,
  addHeader,
  removeHeader,
  loading,
  saveSettings,
  close,
} = useAiSettingsSetup(props, emit);
</script>

<template>
  <div v-if="visible" class="ai-settings-panel">
    <div class="settings-header">
      <span class="settings-title">{{ t("menu.aiSettings") }}</span>
      <button class="close-btn" @click="close">✕</button>
    </div>

    <div class="settings-content">
      <!-- AI 设置 -->
      <div class="settings-section">
        <div class="setting-item">
          <span class="setting-label">{{ t("settings.aiApiEndpoint") }}</span>
          <input
            type="text"
            class="setting-input full-width"
            v-model="aiApiEndpoint"
            :disabled="loading"
            placeholder="https://api.openai.com/v1"
          />
        </div>

        <div class="setting-description">
          {{ t("settings.aiApiEndpointDesc") }}
        </div>

        <div class="setting-item">
          <span class="setting-label">{{ t("settings.aiApiKey") }}</span>
          <input
            type="text"
            class="setting-input full-width"
            v-model="aiApiKey"
            :disabled="loading"
            :placeholder="apiKeyPlaceholder"
          />
        </div>

        <div class="setting-description">
          {{ t("settings.aiApiKeyDesc") }}
        </div>

        <div class="setting-item">
          <span class="setting-label">{{ t("settings.aiModel") }}</span>
          <div class="model-combo-wrapper">
            <div class="model-input-container">
              <input
                type="text"
                class="model-input"
                v-model="aiModel"
                :disabled="loading"
                :placeholder="t('settings.aiModelPlaceholder')"
                @focus="closeModelDropdown"
              />
              <button
                class="model-dropdown-btn"
                :disabled="loading || aiModels.length === 0"
                @click="toggleModelDropdown"
              >
                ▼
              </button>
              <div v-if="showModelDropdown" class="model-dropdown">
                <div
                  v-for="model in aiModels"
                  :key="model"
                  class="model-dropdown-item"
                  @click="selectModel(model)"
                >
                  {{ model }}
                </div>
              </div>
            </div>
            <button
              class="fetch-models-btn"
              :disabled="
                loading ||
                loadingModels ||
                !aiApiEndpoint ||
                (!aiApiKey && !hasApiKey)
              "
              @click="fetchModels"
            >
              {{
                loadingModels
                  ? t("settings.aiFetchingModels")
                  : t("settings.aiFetchModels")
              }}
            </button>
          </div>
        </div>

        <div class="setting-description">
          {{ t("settings.aiModelDesc") }}
        </div>

        <!-- AI 超时时间 -->
        <div class="setting-item">
          <span class="setting-label">{{ t("settings.aiTimeout") }}</span>
          <div class="timeout-wrapper">
            <input
              type="number"
              class="setting-input timeout-input"
              v-model="aiTimeout"
              :disabled="loading"
              min="1"
              max="3600"
            />
            <span class="timeout-unit">{{ t("settings.aiTimeoutUnit") }}</span>
          </div>
        </div>

        <div class="setting-description">
          {{ t("settings.aiTimeoutDesc") }}
        </div>

        <!-- 自定义请求头 -->
        <div class="custom-headers-section">
          <div class="setting-label">{{ t("settings.aiCustomHeaders") }}</div>
          <div class="setting-description">
            {{ t("settings.aiCustomHeadersDesc") }}
          </div>

          <div class="headers-toolbar">
            <button
              class="add-header-btn"
              @click="addHeader"
              :disabled="loading"
            >
              {{ t("buttons.addHeader") }}
            </button>
          </div>

          <div class="headers-list">
            <div v-if="customHeaders.length === 0" class="empty-headers">
              {{ t("empty.noHeaders") }}
            </div>
            <div
              v-for="(header, index) in customHeaders"
              :key="index"
              class="header-row"
            >
              <input
                type="checkbox"
                v-model="header.enabled"
                class="header-checkbox"
              />
              <input
                type="text"
                v-model="header.key"
                class="header-key-input"
                :placeholder="t('placeholder.headerName')"
              />
              <input
                type="text"
                v-model="header.value"
                class="header-value-input"
                :placeholder="t('placeholder.headerValue')"
              />
              <button
                class="remove-header-btn"
                @click="removeHeader(index)"
                :disabled="loading"
              >
                ×
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="settings-footer">
      <button class="cancel-btn" @click="close">
        {{ t("common.cancel") }}
      </button>
      <button class="save-btn" :disabled="loading" @click="saveSettings">
        {{ t("common.save") }}
      </button>
    </div>
  </div>
</template>
