<script setup>
import { useI18n } from "vue-i18n";
import { useRequestPanelSetup } from "./index.js";
import Icon from "../Icon/index.vue";
import VariableHighlight from "../VariableHighlight/index.vue";
import ScriptPanel from "../ScriptPanel/index.vue";
import DocPanel from "../DocPanel/index.vue";
import StressTestPanel from "../StressTestPanel/index.vue";
import HeaderAutocomplete from "../HeaderAutocomplete/index.vue";

const { t } = useI18n();

const props = defineProps({
  request: {
    type: Object,
    default: () => ({}),
  },
  hasActiveTab: {
    type: Boolean,
    default: false,
  },
  variables: {
    type: Array,
    default: () => [],
  },
  requestTab: {
    type: String,
    default: "params",
  },
  workspaceId: {
    type: String,
    default: "",
  },
  apiId: {
    type: String,
    default: "",
  },
});

const emit = defineEmits(["update:request", "send", "save", "updateTab"]);

const {
  methods,
  tabs,
  bodyTypes,
  rawTypes,
  activeTab,
  localRequest,
  selectedRawType,
  methodClass,
  editorContainer,
  updateMethod,
  updateUrl,
  sendRequest,
  saveRequest,
  addParam,
  removeParam,
  addHeader,
  removeHeader,
  handleFormat,
  addFormField,
  removeFormField,
  addFormUrlField,
  removeFormUrlField,
  selectFormFieldFiles,
  removeFormFieldFile,
  handleScriptUpdate,
  saveScripts,
  showHeaderKeyAutocomplete,
  showHeaderValueAutocomplete,
  headerAutocompletePosition,
  headerAutocompleteWidth,
  headerSelectedIndex,
  filteredHeaderKeys,
  filteredHeaderValues,
  handleHeaderKeyInput,
  handleHeaderValueInput,
  handleHeaderValueChange,
  selectHeaderKey,
  selectHeaderValue,
  hideHeaderAutocomplete,
  handleHeaderKeyNavigation,
  headerValueRefs,
  variables,
} = useRequestPanelSetup(props, emit);
</script>

<template>
  <div class="request-panel">
    <!-- 空状态 -->
    <div v-if="!hasActiveTab" class="empty-state">
      <span class="empty-icon"><Icon name="api" :size="48" /></span>
      <p class="empty-text">{{ t("empty.selectApi") }}</p>
      <p class="empty-hint">{{ t("empty.selectApiHint") }}</p>
    </div>

    <!-- 有内容时显示 -->
    <template v-else>
      <!-- URL 输入行 -->
      <div class="url-bar">
        <div class="method-selector">
          <select
            :value="localRequest.method"
            @change="updateMethod($event.target.value)"
            class="method-select"
            :class="methodClass"
          >
            <option v-for="m in methods" :key="m" :value="m">{{ m }}</option>
          </select>
        </div>
        <VariableHighlight
          mode="input"
          :text="localRequest.url"
          :variables="variables"
          @input="updateUrl"
          class="url-mode"
          :placeholder="t('placeholder.url')"
        />
        <button class="send-btn" @click="sendRequest">
          {{ t("buttons.send") }}
        </button>
        <button class="save-btn" @click="saveRequest">
          {{ t("common.save") }}
        </button>
      </div>

      <!-- 请求配置标签页 -->
      <div class="request-tabs">
        <div
          v-for="tab in tabs"
          :key="tab.key"
          class="tab-item"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key"
        >
          {{ tab.name }}
          <span
            v-if="tab.key === 'params' && localRequest.params?.length"
            class="tab-cnt"
            >{{ localRequest.params.length }}</span
          >
          <span
            v-if="tab.key === 'headers' && localRequest.headers?.length"
            class="tab-cnt"
            >{{ localRequest.headers.length }}</span
          >
        </div>
      </div>

      <!-- 标签页内容 -->
      <div class="tab-content">
        <!-- 参数 -->
        <div v-show="activeTab === 'params'" class="params-panel">
          <div class="params-toolbar">
            <button class="add-btn" @click="addParam">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
              >
                <path d="M12 5v14M5 12h14" />
              </svg>
              {{ t("buttons.addParam") }}
            </button>
          </div>
          <div class="params-list">
            <div class="params-header">
              <span class="col-check"></span>
              <span class="col-key">{{ t("table.paramName") }}</span>
              <span class="col-value">{{ t("table.paramValue") }}</span>
              <span class="col-desc">{{ t("table.description") }}</span>
              <span class="col-action"></span>
            </div>
            <div v-if="localRequest.params.length === 0" class="empty-params">
              {{ t("empty.noParams") }}
            </div>
            <div
              v-for="(param, index) in localRequest.params"
              :key="index"
              class="param-row"
            >
              <span class="col-check">
                <input type="checkbox" v-model="param.enabled" />
              </span>
              <span class="col-key">
                <input
                  type="text"
                  v-model="param.key"
                  :placeholder="t('placeholder.paramName')"
                />
              </span>
              <span class="col-value">
                <input
                  type="text"
                  v-model="param.value"
                  :placeholder="t('placeholder.paramValue')"
                />
              </span>
              <span class="col-desc">
                <input
                  type="text"
                  v-model="param.description"
                  :placeholder="t('placeholder.description')"
                />
              </span>
              <span class="col-action">
                <button class="remove-btn" @click="removeParam(index)">
                  ×
                </button>
              </span>
            </div>
          </div>
        </div>

        <!-- 请求头 -->
        <div v-show="activeTab === 'headers'" class="headers-panel">
          <div class="params-toolbar">
            <button class="add-btn" @click="addHeader">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
              >
                <path d="M12 5v14M5 12h14" />
              </svg>
              {{ t("buttons.addHeader") }}
            </button>
          </div>
          <div class="params-list">
            <div class="params-header">
              <span class="col-check"></span>
              <span class="col-key">{{ t("table.headerName") }}</span>
              <span class="col-value">{{ t("table.headerValue") }}</span>
              <span class="col-desc">{{ t("table.description") }}</span>
              <span class="col-action"></span>
            </div>
            <div v-if="localRequest.headers.length === 0" class="empty-params">
              {{ t("empty.noHeaders") }}
            </div>
            <div
              v-for="(header, index) in localRequest.headers"
              :key="index"
              class="param-row"
            >
              <span class="col-check">
                <input type="checkbox" v-model="header.enabled" />
              </span>
              <span class="col-key header-key-cell">
                <input
                  type="text"
                  v-model="header.key"
                  :placeholder="t('placeholder.headerName')"
                  @input="handleHeaderKeyInput(header.key, index, $event)"
                  @keydown="handleHeaderKeyNavigation($event, index)"
                  @blur="hideHeaderAutocomplete"
                  @focus="handleHeaderKeyInput(header.key, index, $event)"
                />
              </span>
              <span class="col-value header-value-cell">
                <VariableHighlight
                  :ref="
                    (el) => {
                      if (el) headerValueRefs[index] = el;
                    }
                  "
                  mode="input"
                  :modelValue="header.value"
                  :variables="variables"
                  :placeholder="t('placeholder.headerValue')"
                  @input="
                    (val) => {
                      header.value = val;
                      handleHeaderValueChange(header.key, val, index);
                    }
                  "
                  @focus="
                    handleHeaderValueInput(
                      header.key,
                      header.value,
                      index,
                      $event,
                    )
                  "
                  @keydown="handleHeaderKeyNavigation($event, index)"
                  @blur="hideHeaderAutocomplete"
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
          <!-- Header Key 自动补全 -->
          <HeaderAutocomplete
            :visible="showHeaderKeyAutocomplete"
            type="key"
            :position="headerAutocompletePosition"
            :width="headerAutocompleteWidth"
            :items="filteredHeaderKeys"
            :selectedIndex="headerSelectedIndex"
            @select="selectHeaderKey"
            @close="hideHeaderAutocomplete"
          />
          <!-- Header Value 自动补全 -->
          <HeaderAutocomplete
            :visible="showHeaderValueAutocomplete"
            type="value"
            :position="headerAutocompletePosition"
            :width="headerAutocompleteWidth"
            :items="filteredHeaderValues"
            :selectedIndex="headerSelectedIndex"
            @select="selectHeaderValue"
            @close="hideHeaderAutocomplete"
          />
        </div>

        <!-- 请求体 -->
        <div v-show="activeTab === 'body'" class="body-panel">
          <div class="body-toolbar">
            <div class="body-type-selector">
              <label
                v-for="type in bodyTypes"
                :key="type.key"
                class="radio-label"
                :class="{ active: localRequest.bodyType === type.key }"
              >
                <input
                  type="radio"
                  :value="type.key"
                  v-model="localRequest.bodyType"
                />
                {{ type.name }}
              </label>
            </div>
          </div>
          <!-- none 类型时不显示编辑器 -->
          <div v-show="localRequest.bodyType === 'none'" class="body-empty">
            <span class="empty-text">{{ t("empty.noBody") }}</span>
          </div>
          <!-- form-data 类型显示键值对表格 -->
          <div
            v-show="localRequest.bodyType === 'form-data'"
            class="form-data-panel"
          >
            <div class="form-toolbar">
              <button class="add-btn" @click="addFormField">
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.2"
                  stroke-linecap="round"
                >
                  <path d="M12 5v14M5 12h14" />
                </svg>
                {{ t("buttons.addField") }}
              </button>
            </div>
            <div class="form-list">
              <div class="form-header">
                <span class="col-check"></span>
                <span class="col-key">{{ t("table.fieldName") }}</span>
                <span class="col-value">{{ t("table.fieldValue") }}</span>
                <span class="col-type">{{ t("table.fieldType") }}</span>
                <span class="col-action"></span>
              </div>
              <div
                v-if="localRequest.formData?.length === 0"
                class="empty-params"
              >
                {{ t("empty.noFields") }}
              </div>
              <div
                v-for="(field, index) in localRequest.formData"
                :key="index"
                class="form-row"
              >
                <span class="col-check">
                  <input type="checkbox" v-model="field.enabled" />
                </span>
                <span class="col-key">
                  <input
                    type="text"
                    v-model="field.key"
                    :placeholder="t('placeholder.fieldName')"
                  />
                </span>
                <span class="col-value">
                  <!-- 文本类型显示输入框 -->
                  <template v-if="field.type === 'text'">
                    <input
                      type="text"
                      v-model="field.value"
                      :placeholder="t('placeholder.fieldValue')"
                    />
                  </template>
                  <!-- 文件类型显示文件选择按钮和已选文件列表 -->
                  <template v-else>
                    <div class="file-value-cell">
                      <button
                        v-if="!field.files || field.files.length === 0"
                        class="select-file-btn small"
                        @click="selectFormFieldFiles(index)"
                      >
                        <Icon name="file" :size="14" />
                        {{ t("buttons.selectFile") }}
                      </button>
                      <div v-else class="selected-files-list">
                        <div
                          v-for="(file, fIndex) in field.files"
                          :key="fIndex"
                          class="selected-file-item"
                        >
                          <span class="file-name">{{ file.name }}</span>
                          <button
                            class="remove-file-btn small"
                            @click="removeFormFieldFile(index, fIndex)"
                          >
                            ×
                          </button>
                        </div>
                      </div>
                    </div>
                  </template>
                </span>
                <span class="col-type">
                  <select v-model="field.type">
                    <option value="text">{{ t("fieldType.text") }}</option>
                    <option value="file">{{ t("fieldType.file") }}</option>
                  </select>
                </span>
                <span class="col-action">
                  <button class="remove-btn" @click="removeFormField(index)">
                    ×
                  </button>
                </span>
              </div>
            </div>
          </div>
          <!-- x-www-form-urlencoded 类型显示键值对表格 -->
          <div
            v-show="localRequest.bodyType === 'x-www-form-urlencoded'"
            class="form-urlencoded-panel"
          >
            <div class="form-toolbar">
              <button class="add-btn" @click="addFormUrlField">
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.2"
                  stroke-linecap="round"
                >
                  <path d="M12 5v14M5 12h14" />
                </svg>
                {{ t("buttons.addField") }}
              </button>
            </div>
            <div class="form-list">
              <div class="form-header">
                <span class="col-check"></span>
                <span class="col-key">{{ t("table.fieldName") }}</span>
                <span class="col-value">{{ t("table.fieldValue") }}</span>
                <span class="col-action"></span>
              </div>
              <div
                v-if="localRequest.formUrlEncoded?.length === 0"
                class="empty-params"
              >
                {{ t("empty.noFields") }}
              </div>
              <div
                v-for="(field, index) in localRequest.formUrlEncoded"
                :key="index"
                class="form-row"
              >
                <span class="col-check">
                  <input type="checkbox" v-model="field.enabled" />
                </span>
                <span class="col-key">
                  <input
                    type="text"
                    v-model="field.key"
                    :placeholder="t('placeholder.fieldName')"
                  />
                </span>
                <span class="col-value">
                  <input
                    type="text"
                    v-model="field.value"
                    :placeholder="t('placeholder.fieldValue')"
                  />
                </span>
                <span class="col-action">
                  <button class="remove-btn" @click="removeFormUrlField(index)">
                    ×
                  </button>
                </span>
              </div>
            </div>
          </div>
          <!-- raw 类型显示 Monaco Editor -->
          <div v-show="localRequest.bodyType === 'raw'" class="editor-wrapper">
            <div class="editor-toolbar">
              <select v-model="selectedRawType" class="raw-select">
                <option v-for="t in rawTypes" :key="t" :value="t">
                  {{ t }}
                </option>
              </select>
              <button class="format-btn" @click="handleFormat">
                {{ t("buttons.format") }}
              </button>
            </div>
            <div class="monaco-editor-container" ref="editorContainer"></div>
          </div>
        </div>

        <!-- 脚本 -->
        <div v-if="activeTab === 'scripts'" class="scripts-panel">
          <ScriptPanel
            :request="localRequest"
            @update:request="handleScriptUpdate"
            @save="saveScripts"
          />
        </div>

        <!-- 文档 -->
        <div v-show="activeTab === 'docs'" class="docs-panel">
          <DocPanel
            :workspaceId="workspaceId"
            :apiId="apiId"
            :apiData="localRequest"
          />
        </div>

        <!-- 压测 -->
        <div v-show="activeTab === 'stress'" class="stress-panel">
          <StressTestPanel
            :workspaceId="workspaceId"
            :apiId="apiId"
            :apiName="localRequest.name"
            :method="localRequest.method"
            :url="localRequest.url"
            :headers="localRequest.headers"
            :body="localRequest.body"
            :bodyType="localRequest.bodyType"
            :formFields="localRequest.formData"
          />
        </div>

        <!-- 其他标签页（占位符） -->
        <div
          v-show="
            activeTab !== 'params' &&
            activeTab !== 'headers' &&
            activeTab !== 'body' &&
            activeTab !== 'scripts' &&
            activeTab !== 'docs' &&
            activeTab !== 'stress'
          "
          class="placeholder-panel"
        >
          <div class="placeholder-content">
            <span class="placeholder-icon">📝</span>
            <p>{{ tabs.find((tab) => tab.key === activeTab)?.name }}</p>
            <p class="placeholder-hint">{{ t("common.developing") }}</p>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped src="./style.css"></style>
