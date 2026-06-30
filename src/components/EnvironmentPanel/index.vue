<script setup>
import { useEnvPanelSetup } from "./index.js";
import Icon from "../Icon/index.vue";
import ScriptPanel from "../ScriptPanel/index.vue";

const props = defineProps({
  activeEnvironment: Object,
  workspaceId: String,
});

const emit = defineEmits(["saveVariables"]);

const {
  t,
  activeTab,
  tabs,
  localSettings,
  addVariable,
  removeVariable,
  addHeader,
  removeHeader,
  handleScriptUpdate,
  saveScripts,
} = useEnvPanelSetup(props, emit);
</script>

<template>
  <div class="env-panel">
    <div v-if="props.activeEnvironment" class="env-content">
      <div class="env-header">
        <span class="env-icon">
          <Icon name="environment" :size="16" />
        </span>
        <span class="env-name">{{ localSettings.name }}</span>
      </div>

      <div class="env-tabs">
        <div
          v-for="tab in tabs"
          :key="tab.key"
          class="tab-item"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key"
        >
          {{ tab.name }}
        </div>
      </div>

      <div class="tab-content">
        <div v-show="activeTab === 'variables'" class="variables-panel">
          <div class="panel-toolbar">
            <button class="add-btn" @click="addVariable">
              {{ t("buttons.addVariable") }}
            </button>
            <span class="panel-hint">{{ t("environment.hint") }}</span>
          </div>
          <div class="params-list">
            <div class="params-header">
              <span class="col-check"></span>
              <span class="col-key">{{ t("table.variableName") }}</span>
              <span class="col-value">{{ t("table.variableValue") }}</span>
              <span class="col-desc">{{ t("table.description") }}</span>
              <span class="col-action"></span>
            </div>
            <div
              v-if="localSettings.variables.length === 0"
              class="empty-params"
            >
              {{ t("empty.noVariables") }}
            </div>
            <div
              v-for="(v, index) in localSettings.variables"
              :key="index"
              class="param-row"
            >
              <span class="col-check">
                <input type="checkbox" v-model="v.enabled" />
              </span>
              <span class="col-key">
                <input type="text" v-model="v.key" placeholder="变量名" />
              </span>
              <span class="col-value">
                <input type="text" v-model="v.value" placeholder="变量值" />
              </span>
              <span class="col-desc">
                <input type="text" v-model="v.description" placeholder="描述" />
              </span>
              <span class="col-action">
                <button class="remove-btn" @click="removeVariable(index)">
                  ×
                </button>
              </span>
            </div>
          </div>
        </div>

        <div v-show="activeTab === 'headers'" class="headers-panel">
          <div class="panel-toolbar">
            <button class="add-btn" @click="addHeader"><svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M12 5v14M5 12h14"/></svg>
              {{ t("buttons.addHeader") }}
            </button>
            <span class="panel-hint">{{
              t("environmentSettings.headersHint")
            }}</span>
          </div>
          <div class="params-list">
            <div class="params-header">
              <span class="col-check"></span>
              <span class="col-key">{{ t("table.headerName") }}</span>
              <span class="col-value">{{ t("table.headerValue") }}</span>
              <span class="col-desc">{{ t("table.description") }}</span>
              <span class="col-action"></span>
            </div>
            <div
              v-if="localSettings.commonHeaders.length === 0"
              class="empty-params"
            >
              {{ t("empty.noEnvironmentHeaders") }}
            </div>
            <div
              v-for="(header, index) in localSettings.commonHeaders"
              :key="index"
              class="param-row"
            >
              <span class="col-check">
                <input type="checkbox" v-model="header.enabled" />
              </span>
              <span class="col-key">
                <input
                  type="text"
                  v-model="header.key"
                  :placeholder="t('placeholder.headerName')"
                />
              </span>
              <span class="col-value">
                <input
                  type="text"
                  v-model="header.value"
                  :placeholder="t('placeholder.headerValue')"
                />
              </span>
              <span class="col-desc">
                <input
                  type="text"
                  v-model="header.description"
                  :placeholder="t('placeholder.description')"
                />
              </span>
              <span class="col-action">
                <button class="remove-btn" @click="removeHeader(index)">
                  ×
                </button>
              </span>
            </div>
          </div>
        </div>

        <div v-if="activeTab === 'scripts'" class="scripts-panel">
          <ScriptPanel
            :request="localSettings"
            @update:request="handleScriptUpdate"
            @save="saveScripts"
          />
        </div>
      </div>
    </div>

    <div class="no-env-selected" v-else>
      {{ t("empty.selectEnvironment") }}
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
