<script setup>
import { useI18n } from "vue-i18n";
import { useAppSetup } from "./App.js";
import TitleBar from "./components/TitleBar/index.vue";
import MenuBar from "./components/MenuBar/index.vue";
import TabsBar from "./components/TabsBar/index.vue";
import Sidebar from "./components/Sidebar/index.vue";
import RequestPanel from "./components/RequestPanel/index.vue";
import ResponsePanel from "./components/ResponsePanel/index.vue";
import StatusBar from "./components/StatusBar/index.vue";
import WorkspaceDialog from "./components/WorkspaceDialog/index.vue";
import EnvironmentPanel from "./components/EnvironmentPanel/index.vue";
import CookiePanel from "./components/CookiePanel/index.vue";
import ConsolePanel from "./components/ConsolePanel/index.vue";
import SaveResponseDialog from "./components/SaveResponseDialog/index.vue";
import HistoryDetailPanel from "./components/HistoryDetailPanel/index.vue";
import CollectionSettingsPanel from "./components/CollectionSettingsPanel/index.vue";
import WorkspaceSettingsPanel from "./components/WorkspaceSettingsPanel/index.vue";
import SettingsCenter from "./components/SettingsCenter/index.vue";
import ChatPanel from "./components/ChatPanel/index.vue";
import OrchestrationEditor from "./components/OrchestrationEditor/index.vue";
import SavedResponseDocPanel from "./components/SavedResponseDocPanel/index.vue";
import WebSocketDetailPanel from "./components/WebSocketDetailPanel/index.vue";
import Toast from "./components/Toast/index.vue";
import { ref, onMounted, onUnmounted } from "vue";
import { useKeyboardShortcuts } from "./composables/useKeyboardShortcuts.js";

// 使用 composable
const { t } = useI18n();
const {
  currentWorkspace,
  workspaces,
  showWorkspaceDialog,
  workspaceDialogMode,
  sidebarRef,
  tabs,
  displayTabs,
  activeTab,
  currentRequest,
  currentRequestTab,
  response,
  loading,
  testResults,
  sseEvents,
  environments,
  activeEnvironment,
  selectedEnvironment,
  availableVariables,
  loadEnvironments,
  saveEnvVariables,
  cookies,
  showCookiePanel,
  loadCookies,
  openCookiePanel,
  closeCookiePanel,
  showConsolePanel,
  consoleLogs,
  openConsolePanel,
  closeConsolePanel,
  clearConsoleLogs,
  showSaveResponseDialog,
  saveResponseDefaultName,
  onSaveResponse,
  handleSaveResponse,
  onSelectSavedResponse,
  showSavedResponseDoc,
  selectedSavedResponse,
  closeSavedResponseDoc,
  onSelectHistory,
  selectedHistoryEntry,
  selectedCollection,
  selectCollection,
  showCollectionSettings,
  onCollectionSettingsSaved,
  currentNavKey,
  showRequestResponse,
  showHistoryDetail,
  showWorkspaceInfo,
  selectedWorkspace,
  showEnvironmentInfo,
  onSwitchEnvironment,
  onSelectEnvironment,
  onSelectWorkspace,
  openCreateWorkspace,
  onWorkspaceCreated,
  onWorkspaceDeleted,
  onWorkspaceUpdated,
  onBranchSwitched,
  onSwitchWorkspace,
  onNavChange,
  closeWorkspaceDialog,
  closeTab,
  closeAllTabs,
  closeOtherTabs,
  selectApi,
  sendRequest,
  saveRequest,
  updateRequest,
  onRenameApi,
  onDeleteApis,
  onDeleteCollection,
  onUpdateRequestTab,
  showChatPanel,
  chatSessionId,
  onSelectChatSession,
  onNewChatSession,
  onSessionCreated,
  showOrchestrationPanel,
  selectedOrchestration,
  onSelectOrchestration,
  onOrchestrationStepsChanged,
  showSettingsPanel,
  settingsCategory,
  openSettings,
  closeSettings,
  openAiSettings,
  openGitBackup,
  // WebSocket 导航（已独立到侧边栏）
  showWebSocketPanel,
  selectedWsConfig,
  onSelectWsConfig,
  onCreateWsConfig,
  // 工作区导入
  onWorkspaceImported,
} = useAppSetup();

// 全局键盘快捷键
useKeyboardShortcuts({
  onSave: () => {
    if (
      tabs.value.length > 0 &&
      displayTabs.value[activeTab.value]?.tabType === "api"
    ) {
      saveRequest(currentRequest);
    }
  },
});

// 侧边栏宽度拖拽
const sidebarWidth = ref(262);
const isDragging = ref(false);
const dragStartX = ref(0);
const dragStartWidth = ref(0);

const startDrag = (e) => {
  isDragging.value = true;
  dragStartX.value = e.clientX;
  dragStartWidth.value = sidebarWidth.value;
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
};

// 统一的鼠标事件处理器（合并 handleDrag/handleResize 和 endDrag/endResize，
// 避免双重 mousemove/mouseup 监听造成的判断浪费与互相干扰）
const handleMouseMove = (e) => {
  if (isDragging.value) {
    const diff = e.clientX - dragStartX.value;
    const newWidth = Math.max(160, Math.min(480, dragStartWidth.value + diff));
    sidebarWidth.value = newWidth;
    return;
  }
  if (isResizing.value) {
    const container = e.target.closest(".content-area");
    if (!container) return;
    const containerHeight = container.clientHeight;
    const diff = e.clientY - resizeStartY.value;
    const diffPercent = (diff / containerHeight) * 100;
    const newHeight = Math.max(
      20,
      Math.min(80, resizeStartHeight.value + diffPercent),
    );
    requestPanelHeight.value = newHeight;
  }
};

const handleMouseUp = () => {
  if (isDragging.value) {
    isDragging.value = false;
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    localStorage.setItem("sidebarWidth", sidebarWidth.value);
    return;
  }
  if (isResizing.value) {
    isResizing.value = false;
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    localStorage.setItem("requestPanelHeight", requestPanelHeight.value);
  }
};

// 请求/响应区域高度拖拽
const requestPanelHeight = ref(50);
const isResizing = ref(false);
const resizeStartY = ref(0);
const resizeStartHeight = ref(0);

const startResize = (e) => {
  isResizing.value = true;
  resizeStartY.value = e.clientY;
  resizeStartHeight.value = requestPanelHeight.value;
  document.body.style.cursor = "row-resize";
  document.body.style.userSelect = "none";
};

onMounted(() => {
  const savedWidth = localStorage.getItem("sidebarWidth");
  if (savedWidth) {
    sidebarWidth.value = parseInt(savedWidth, 10);
  }
  const savedHeight = localStorage.getItem("requestPanelHeight");
  if (savedHeight) {
    requestPanelHeight.value = parseFloat(savedHeight);
  }
  document.addEventListener("mousemove", handleMouseMove);
  document.addEventListener("mouseup", handleMouseUp);
});

onUnmounted(() => {
  document.removeEventListener("mousemove", handleMouseMove);
  document.removeEventListener("mouseup", handleMouseUp);
});
</script>

<template>
  <div class="app-container">
    <!-- 自定义标题栏 -->
    <TitleBar />

    <!-- 顶部区域 -->
    <div class="top-area">
      <MenuBar
        ref="menuBarRef"
        :workspaces="workspaces"
        :current-workspace="currentWorkspace"
        :environments="environments"
        :active-environment="activeEnvironment"
        @switch-workspace="onSwitchWorkspace"
        @switch-environment="onSwitchEnvironment"
        @open-settings="openSettings"
        @open-ai-settings="openAiSettings"
        @open-git-backup="openGitBackup"
      />
    </div>

    <!-- 主内容区 -->
    <div class="main-area">
      <!-- 左侧导航 -->
      <Sidebar
        ref="sidebarRef"
        :workspace="currentWorkspace"
        :style="{ width: sidebarWidth + 'px' }"
        @select-api="selectApi"
        @select-collection="selectCollection"
        @delete-collection="onDeleteCollection"
        @switch-workspace="onSwitchWorkspace"
        @create-workspace="openCreateWorkspace"
        @rename-api="onRenameApi"
        @delete-apis="onDeleteApis"
        @nav-change="onNavChange"
        @select-environment="onSelectEnvironment"
        @environment-updated="loadEnvironments"
        @workspace-deleted="onWorkspaceDeleted"
        @select-saved-response="onSelectSavedResponse"
        @select-history="onSelectHistory"
        @select-workspace="onSelectWorkspace"
        @workspace-updated="onWorkspaceUpdated"
        @branch-switched="onBranchSwitched"
        @select-chat-session="onSelectChatSession"
        @new-chat-session="onNewChatSession"
        @session-created="onSessionCreated"
        @select-orchestration="onSelectOrchestration"
        @workspace-imported="onWorkspaceImported"
        @select-ws-config="onSelectWsConfig"
        @create-ws-config="onCreateWsConfig"
      />

      <!-- 可拖拽分隔线 -->
      <div
        class="sidebar-resizer"
        :class="{ dragging: isDragging }"
        @mousedown="startDrag"
      ></div>

      <!-- 主内容列：TabsBar + 面板（位于 Sidebar 右侧，不跨到集合列表上方） -->
      <div class="main-column">
        <TabsBar
          v-if="
            (showRequestResponse || showCollectionSettings) &&
            !showSavedResponseDoc
          "
          :tabs="displayTabs"
          :active-tab="activeTab"
          :workspace="currentWorkspace"
          @update:active-tab="activeTab = $event"
          @close-tab="closeTab"
          @close-all-tabs="closeAllTabs"
          @close-other-tabs="closeOtherTabs"
          @select-collection="selectCollection"
          @select-api="selectApi"
        />

        <!-- 保存响应 MD 文档面板（优先级高于 showRequestResponse） -->
        <div class="content-area" v-if="showSavedResponseDoc">
          <SavedResponseDocPanel
            :saved-response="selectedSavedResponse"
            :workspace-id="currentWorkspace?.id || ''"
            @close="closeSavedResponseDoc"
          />
        </div>

        <!-- WebSocket 导航时显示详情面板 -->
        <div class="content-area" v-else-if="showWebSocketPanel">
          <WebSocketDetailPanel
            :workspace-id="currentWorkspace?.id || ''"
            :ws-config="selectedWsConfig"
          />
        </div>

        <!-- 中间内容区 -->
        <div class="content-area" v-else-if="showRequestResponse">
          <!-- 请求区 -->
          <div
            class="request-area"
            :style="{
              height:
                currentRequestTab === 'scripts' ||
                currentRequestTab === 'docs' ||
                currentRequestTab === 'stress'
                  ? '100%'
                  : requestPanelHeight + '%',
            }"
          >
            <RequestPanel
              :request="currentRequest"
              :has-active-tab="tabs.length > 0"
              :variables="availableVariables"
              :request-tab="currentRequestTab"
              :workspace-id="currentWorkspace?.id || ''"
              :api-id="displayTabs[activeTab]?.id || ''"
              @update:request="updateRequest($event)"
              @send="sendRequest"
              @save="saveRequest"
              @update-tab="onUpdateRequestTab"
            />
          </div>

          <!-- 请求/响应分割线 -->
          <div
            class="panel-resizer"
            :class="{ resizing: isResizing }"
            @mousedown="startResize"
            v-if="
              currentRequestTab !== 'scripts' &&
              currentRequestTab !== 'docs' &&
              currentRequestTab !== 'stress'
            "
          ></div>

          <!-- 响应区 - 脚本/文档/压测tab时不显示 -->
          <div
            class="response-area"
            v-if="
              showRequestResponse &&
              currentNavKey === 'collection' &&
              currentRequestTab !== 'stress'
            "
            :style="{ height: 100 - requestPanelHeight + '%' }"
          >
            <ResponsePanel
              :response="response"
              :loading="loading"
              :test-results="testResults"
              :sse-events="sseEvents"
              @save-response="onSaveResponse"
            />
          </div>
        </div>

        <!-- 集合设置面板 -->
        <div class="content-area" v-else-if="showCollectionSettings">
          <CollectionSettingsPanel
            :collection="selectedCollection"
            :workspace-id="currentWorkspace?.id || ''"
            :variables="availableVariables"
            @save="onCollectionSettingsSaved"
          />
        </div>

        <!-- 历史详情面板 -->
        <div class="content-area" v-else-if="showHistoryDetail">
          <HistoryDetailPanel :entry="selectedHistoryEntry" />
        </div>

        <!-- 工作区设置面板 -->
        <div class="content-area" v-else-if="showWorkspaceInfo">
          <WorkspaceSettingsPanel
            :workspace="selectedWorkspace"
            :workspace-id="selectedWorkspace?.id || ''"
          />
        </div>

        <!-- 环境信息面板 -->
        <div class="content-area" v-else-if="showEnvironmentInfo">
          <EnvironmentPanel
            :active-environment="selectedEnvironment"
            :workspace-id="currentWorkspace?.id || ''"
            @save-variables="saveEnvVariables"
          />
        </div>

        <!-- Chat 面板 -->
        <div class="content-area" v-else-if="showChatPanel">
          <ChatPanel
            :workspace-id="currentWorkspace?.id || ''"
            :session-id="chatSessionId"
          />
        </div>

        <!-- 编排面板 -->
        <div
          class="content-area"
          v-else-if="showOrchestrationPanel && selectedOrchestration"
        >
          <OrchestrationEditor
            :workspace-id="currentWorkspace?.id || ''"
            :orchestration-id="selectedOrchestration?.id || ''"
            @steps-changed="onOrchestrationStepsChanged"
          />
        </div>

        <!-- 编排空状态 -->
        <div
          class="content-area"
          v-else-if="showOrchestrationPanel && !selectedOrchestration"
        >
          <div class="orchestration-placeholder">
            <div class="placeholder-hint">{{ t("empty.selectApiHint") }}</div>
          </div>
        </div>

        <!-- 空状态提示 -->
        <div class="empty-content" v-else>
          <div class="empty-message">
            {{
              currentWorkspace
                ? t("empty.selectApiHint")
                : t("empty.selectWorkspace")
            }}
          </div>
        </div>
      </div>
    </div>

    <!-- 控制台面板 -->
    <ConsolePanel
      :visible="showConsolePanel"
      :logs="consoleLogs"
      @close="closeConsolePanel"
      @clear="clearConsoleLogs"
    />

    <!-- 底部状态栏 -->
    <StatusBar
      :workspace-name="currentWorkspace?.name"
      @open-cookie-panel="openCookiePanel"
      @open-console-panel="openConsolePanel"
    />

    <!-- Cookie 管理面板 -->
    <CookiePanel
      :visible="showCookiePanel"
      :cookies="cookies"
      :workspace-id="currentWorkspace?.id || ''"
      @close="closeCookiePanel"
      @refresh="loadCookies"
    />

    <!-- 工作区对话框 -->
    <WorkspaceDialog
      :visible="showWorkspaceDialog"
      :mode="workspaceDialogMode"
      @close="closeWorkspaceDialog"
      @created="onWorkspaceCreated"
      @updated="onWorkspaceUpdated"
    />

    <!-- 保存响应对话框 -->
    <SaveResponseDialog
      :show="showSaveResponseDialog"
      :default-name="saveResponseDefaultName"
      @save="handleSaveResponse"
      @cancel="showSaveResponseDialog = false"
    />

    <!-- 统一设置中心（合并原 Settings/AI/Git 三个面板） -->
    <SettingsCenter
      :visible="showSettingsPanel"
      :initial-category="settingsCategory"
      @close="closeSettings"
    />

    <!-- Toast 提示 -->
    <Toast />
  </div>
</template>

<style>
/* 全局样式 - 主题由 themes.css 控制 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html,
body,
#app {
  height: 100%;
  overflow: hidden;
}

body {
  font-family: var(--font-ui);
  font-size: var(--font-size-base);
  /* 使用 CSS 变量，由主题系统控制 */
  color: var(--text-primary);
  background: var(--background-app);
  transition:
    background-color 0.3s ease,
    color 0.3s ease;
}
</style>

<style scoped src="./App.css"></style>
