<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { useI18nSetup } from "../../composables/useI18n";
import { useDialogEscape } from "../../composables/useDialogStack.js";
import Icon from "../Icon/index.vue";

const { t } = useI18nSetup();

defineProps({
  workspaces: {
    type: Array,
    default: () => [],
  },
  currentWorkspace: {
    type: Object,
    default: null,
  },
  environments: {
    type: Array,
    default: () => [],
  },
  activeEnvironment: {
    type: Object,
    default: null,
  },
});

const emit = defineEmits([
  "switchWorkspace",
  "switchEnvironment",
  "openSettings",
  "openAiSettings",
  "openGitBackup",
]);

// 统一下拉状态：'ws' | 'env' | 'lang' | 'help' | 'about' | null
const openMenu = ref(null);
const showScriptHelp = ref(false);
const showLicensePanel = ref(false);

const toggleMenu = (name) => {
  openMenu.value = openMenu.value === name ? null : name;
};
const closeMenu = () => {
  openMenu.value = null;
};

const pickWorkspace = (w) => {
  emit("switchWorkspace", w);
  closeMenu();
};
const pickEnvironment = (env) => {
  emit("switchEnvironment", env.id);
  closeMenu();
};
const closeScriptHelp = () => {
  showScriptHelp.value = false;
};
const closeLicense = () => {
  showLicensePanel.value = false;
};

useDialogEscape(showLicensePanel, closeLicense);
useDialogEscape(showScriptHelp, closeScriptHelp);
useDialogEscape(openMenu, () => {
  openMenu.value = null;
});

// 点击外部关闭下拉
const handleClickOutside = (e) => {
  if (!e.target.closest(".toolbar-pop")) {
    openMenu.value = null;
  }
};

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
});
</script>

<template>
  <div class="toolbar">
    <!-- 工作区下拉 -->
    <div
      class="toolbar-pop ws-select"
      :class="{ on: openMenu === 'ws' }"
      @click="toggleMenu('ws')"
    >
      <Icon name="workspace" :size="14" class="ws-ic" />
      <span class="ws-name">{{
        currentWorkspace?.name || t("menu.workspace")
      }}</span>
      <svg
        class="chev ws-chev"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M6 9l6 6 6-6" />
      </svg>
      <div v-if="openMenu === 'ws'" class="pop-menu ws-menu">
        <div
          v-for="w in workspaces"
          :key="w.id"
          class="pop-item"
          :class="{ on: w.id === currentWorkspace?.id }"
          @click.stop="pickWorkspace(w)"
        >
          <span class="pop-name">{{ w.name }}</span>
        </div>
        <div v-if="!workspaces.length" class="pop-item muted">
          <span class="pop-name">{{ t("environment.noEnvironment") }}</span>
        </div>
      </div>
    </div>

    <!-- 环境下拉 -->
    <div
      class="toolbar-pop env-select"
      :class="{ on: openMenu === 'env' }"
      @click="toggleMenu('env')"
    >
      <Icon name="environment" :size="14" class="env-ic" />
      <span class="env-name">{{
        activeEnvironment?.name || t("environment.notSelected")
      }}</span>
      <svg
        class="chev"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M6 9l6 6 6-6" />
      </svg>
      <div v-if="openMenu === 'env'" class="pop-menu">
        <div
          v-for="env in environments"
          :key="env.id"
          class="pop-item"
          :class="{ on: env.id === activeEnvironment?.id }"
          @click.stop="pickEnvironment(env)"
        >
          <span class="pop-name">{{ env.name }}</span>
        </div>
        <div v-if="!environments.length" class="pop-item muted">
          <span class="pop-name">{{ t("environment.noEnvironment") }}</span>
        </div>
      </div>
    </div>

    <span class="tb-spacer"></span>

    <!-- 右侧操作（仅设置入口；AI/Git/语言/帮助/关于 均收进设置中心） -->
    <div class="tb-right">
      <button
        class="iconbtn"
        :title="t('menu.preferences')"
        @click="emit('openSettings')"
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="3" />
          <path
            d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
          />
        </svg>
      </button>
    </div>

    <!-- 脚本 API 参考面板 -->
    <div v-if="showScriptHelp" class="script-help-panel">
      <div class="help-header">
        <span class="help-title">{{ t("script.apiRef.title") }}</span>
        <button class="close-btn" @click="closeScriptHelp">×</button>
      </div>
      <div class="help-content">
        <div class="help-section">
          <h3>{{ t("script.apiRef.preScriptSection") }}</h3>
          <p class="section-desc">{{ t("script.apiRef.preScriptDesc") }}</p>
          <div class="api-group">
            <h4>{{ t("script.apiRef.environment") }}</h4>
            <code
              >fm.environment.get(key) / fm.environment.set(key, value)</code
            >
            <code>fm.environment.remove(key) / fm.environment.getAll()</code>
          </div>
          <div class="api-group">
            <h4>{{ t("script.apiRef.collection") }}</h4>
            <code>fm.collection.get(key) / fm.collection.set(key, value)</code>
            <code>fm.collection.remove(key) / fm.collection.getAll()</code>
          </div>
          <div class="api-group">
            <h4>{{ t("script.apiRef.request") }}</h4>
            <code>fm.request.getUrl() / fm.request.setUrl(url)</code>
            <code
              >fm.request.getBaseUrl() / fm.request.setBaseUrl(baseUrl)</code
            >
            <code>fm.request.getPath() / fm.request.setPath(path)</code>
            <code>fm.request.getMethod() / fm.request.setMethod(method)</code>
            <code
              >fm.request.getHeader(key) / fm.request.setHeader(key,
              value)</code
            >
            <code>fm.request.removeHeader(key) / fm.request.getHeaders()</code>
            <code
              >fm.request.getParam(key) / fm.request.setParam(key, value)</code
            >
            <code>fm.request.removeParam(key) / fm.request.getParams()</code>
            <code>fm.request.getBody() / fm.request.setBody(body)</code>
          </div>
          <div class="api-group">
            <h4>{{ t("script.apiRef.tools") }}</h4>
            <code>fm.log(...args) - {{ t("script.apiRef.logDesc") }}</code>
            <code
              >fm.assert(condition, message) -
              {{ t("script.apiRef.assertDesc") }}</code
            >
            <code>fm.sleep(ms) - {{ t("script.apiRef.sleepDesc") }}</code>
          </div>
        </div>

        <div class="help-section">
          <h3>{{ t("script.apiRef.postScriptSection") }}</h3>
          <p class="section-desc">{{ t("script.apiRef.postScriptDesc") }}</p>
          <div class="api-group">
            <h4>{{ t("script.apiRef.response") }}</h4>
            <code>fm.response.getStatus() / fm.response.getStatusText()</code>
            <code>fm.response.getHeader(key) / fm.response.getHeaders()</code>
            <code>fm.response.getBody() / fm.response.getJson()</code>
            <code>fm.response.getTime() / fm.response.getSize()</code>
          </div>
        </div>

        <div class="help-section">
          <h3>{{ t("script.apiRef.executionOrder") }}</h3>
          <p class="section-desc">
            {{ t("script.apiRef.executionOrderDesc") }}
          </p>
        </div>
      </div>
    </div>

    <!-- 开源协议面板 -->
    <div v-if="showLicensePanel" class="license-panel">
      <div class="license-header">
        <span class="license-title">{{ t("license.mitTitle") }}</span>
        <button class="close-btn" @click="closeLicense">×</button>
      </div>
      <div class="license-content">
        <pre>{{ t("license.mitContent") }}</pre>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
