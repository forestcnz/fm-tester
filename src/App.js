import { ref, reactive, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useWorkspace } from "./composables/useWorkspace.js";
import { useEnvironment } from "./composables/useEnvironment.js";
import { useTabs } from "./composables/useTabs.js";
import { useRequest } from "./composables/useRequest.js";
import { useResponse } from "./composables/useResponse.js";
import { useSettings } from "./composables/useSettings.js";
import { useOrchestrationExecution } from "./composables/useOrchestrationExecution.js";
import { useTheme } from "./composables/useTheme.js";
import { useI18n } from "vue-i18n";
import { showToast } from "./composables/useToast.js";

// 导出 composable 函数
export function useAppSetup() {
  const { t } = useI18n();

  // 侧边栏引用（需要在组件中设置）
  const sidebarRef = ref(null);

  // 清理函数数组
  const cleanupFunctions = [];

  // 当前导航项
  const currentNavKey = ref("collection");

  // UI 面板显示状态（提前声明，避免 watch 引用未声明的 ref 造成 TDZ 隐患）
  const showChatPanel = ref(false);
  const chatSessionId = ref(null);
  const showOrchestrationPanel = ref(false);
  const selectedOrchestration = ref(null);
  const showWebSocketPanel = ref(false);
  const selectedWsConfig = ref(null);

  // 请求子标签页状态
  const requestTabs = ref({});

  // 当前请求子标签页
  const currentRequestTab = ref("params");

  // 标签页列表
  const tabs = ref([]);
  const activeTab = ref(0);

  // 当前请求状态
  const currentRequest = reactive({
    method: "GET",
    url: "",
    params: [],
    headers: [],
    body: "",
    bodyType: "raw",
    formData: [],
    binaryFile: null,
    timeout: null,
  });

  // 响应数据
  const response = ref(null);
  const loading = ref(false);

  // 初始化各个模块
  const workspace = useWorkspace();
  const settings = useSettings();
  const orchestrationExecution = useOrchestrationExecution();
  const theme = useTheme(); // 初始化主题系统

  // 标签页管理模块
  const tabsModule = useTabs(
    workspace.currentWorkspace,
    currentNavKey,
    sidebarRef,
    currentRequest,
    response,
    loading,
    requestTabs,
    tabs,
    activeTab,
    currentRequestTab,
  );

  // 请求管理模块
  const requestModule = useRequest(
    workspace.currentWorkspace,
    tabs,
    activeTab,
    sidebarRef,
    requestTabs,
    currentRequestTab,
    tabsModule.updateCurrentRequest,
    tabsModule.saveOpenTabs,
    currentRequest,
    response,
    loading,
  );

  // 响应管理模块
  const responseModule = useResponse(
    workspace.currentWorkspace,
    tabs,
    activeTab,
    currentNavKey,
    sidebarRef,
    response,
    currentRequest,
    tabsModule.updateCurrentRequest,
    requestModule.testResults,
    requestModule.sseEvents,
  );

  // 环境管理模块
  const environment = useEnvironment(workspace.currentWorkspace, currentNavKey);

  // 设置 activeTab watcher
  requestModule.setupActiveTabWatcher();

  // 监听当前标签页变化，加载可用变量
  // 使用 getter 函数监听 tabs 数组内容变化（push 不改变引用）
  watch(
    [() => [...tabs.value], activeTab],
    () => {
      const currentTab = tabs.value[activeTab.value];
      if (
        currentTab &&
        currentNavKey.value === "collection" &&
        workspace.currentWorkspace.value?.id
      ) {
        environment.loadAvailableVariables(currentTab.id, currentTab.tabType);
      } else {
        environment.availableVariables.value = [];
      }
    },
    { immediate: true },
  );

  // 监听环境变化，重新加载可用变量
  watch(
    () => environment.activeEnvironmentId.value,
    () => {
      const currentTab = tabs.value[activeTab.value];
      if (
        currentTab &&
        currentNavKey.value === "collection" &&
        workspace.currentWorkspace.value?.id
      ) {
        environment.loadAvailableVariables(currentTab.id, currentTab.tabType);
      }
    },
  );

  // 监听导航变化，更新 Chat 面板状态
  watch(currentNavKey, () => {
    showChatPanel.value = currentNavKey.value === "chat";
  });

  // 生命周期钩子
  // 保存事件监听器引用以便清理
  let contextmenuHandler = null;

  onMounted(async () => {
    // 初始化主题
    theme.initTheme();

    // 添加 contextmenu 事件监听器并保存引用
    contextmenuHandler = (e) => {
      e.preventDefault();
    };
    document.addEventListener("contextmenu", contextmenuHandler);

    // 并行启动：事件监听器设置和加载工作区
    const [, lastWorkspace] = await Promise.all([
      requestModule.setupHttpLogListener(),
      workspace.loadLastWorkspace(),
    ]);

    if (lastWorkspace?.id) {
      // 并行加载环境、Cookies、标签页
      await Promise.all([
        environment.loadEnvironments(),
        responseModule.loadCookies(),
        tabsModule.loadOpenTabs(lastWorkspace.id),
      ]);
    }

    // 监听定时任务触发事件
    const unlistenScheduledRun = await listen(
      "orchestration-scheduled-run",
      async (event) => {
        const orchestrationId = event.payload;

        // 全局自动执行编排（不管在哪个页面）
        try {
          const orchestration = await invoke("get_orchestration", {
            workspaceId: workspace.currentWorkspace.value?.id,
            orchestrationId,
          });
          showToast(
            t("toast.scheduledTriggered", { name: orchestration.name }),
            "info",
          );

          // 自动执行编排
          await orchestrationExecution.executeOrchestration(
            workspace.currentWorkspace.value?.id,
            orchestrationId,
          );
        } catch (e) {
          console.error("定时任务执行失败:", e);
          showToast(t("toast.scheduledTriggerFailed"), "error");
        }
      },
    );

    // 保存 unlisten 函数
    cleanupFunctions.push(unlistenScheduledRun);

    // 延迟恢复定时任务（不阻塞启动）
    if (lastWorkspace?.id) {
      invoke("restore_scheduled_tasks_cmd", {
        workspaceId: lastWorkspace.id,
      }).catch((e) => console.error("恢复定时任务失败:", e));
    }
  });

  onUnmounted(() => {
    // 清理 HTTP 日志监听器
    requestModule.cleanupHttpLogListener();

    // 清理 contextmenu 事件监听器
    if (contextmenuHandler) {
      document.removeEventListener("contextmenu", contextmenuHandler);
      contextmenuHandler = null;
    }

    // 执行所有保存的清理函数（如 Tauri 事件监听器）
    cleanupFunctions.forEach((fn) => fn());
    cleanupFunctions.length = 0;
  });

  // 导航切换处理
  const onNavChange = async (navKey) => {
    currentNavKey.value = navKey;
    // 切换导航时关闭 WebSocket 面板（websocket 导航项在侧边栏有独立面板）
    if (showWebSocketPanel.value && navKey !== "websocket") {
      showWebSocketPanel.value = false;
      selectedWsConfig.value = null;
      if (currentRequest.method === "WebSocket") {
        currentRequest.method = "GET";
      }
    }
    if (navKey !== "history") {
      responseModule.selectedHistoryEntry.value = null;
    }
    if (navKey !== "collection") {
      responseModule.closeSavedResponseDoc();
    }
    if (navKey === "environment") {
      await environment.loadEnvironments();
    }
    showOrchestrationPanel.value = navKey === "orchestration";
    showWebSocketPanel.value = navKey === "websocket";
  };

  // WebSocket 配置选择
  const handleSelectWsConfig = (config) => {
    selectedWsConfig.value = config;
  };

  const handleCreateWsConfig = () => {
    // 新建配置后，selectedWsConfig 会由 Sidebar 事件设置
  };

  const handleSelectOrchestration = (orch) => {
    selectedOrchestration.value = orch;
  };

  const handleOrchestrationStepsChanged = async () => {
    await sidebarRef.value?.loadOrchestrations();
  };

  // 处理选择聊天会话
  const handleSelectChatSession = (session) => {
    chatSessionId.value = session.id;
  };

  // 处理新建聊天会话
  const handleNewChatSession = () => {
    chatSessionId.value = null;
  };

  // 处理会话创建完成（设置sessionId以便显示）
  const handleSessionCreated = (sessionId) => {
    chatSessionId.value = sessionId;
  };

  // 工作区切换后的额外处理
  const handleWorkspaceSwitch = async (ws) => {
    await workspace.onSwitchWorkspace(ws);
    tabs.value = [];
    activeTab.value = 0;
    tabsModule.collectionTabsData.value = {};
    // 重置编排状态
    selectedOrchestration.value = null;
    await environment.loadEnvironments();
    await responseModule.loadCookies();
    if (ws?.id) {
      await sidebarRef.value?.loadCollections();
      await sidebarRef.value?.loadEnvironments();
      await tabsModule.loadOpenTabs(ws.id);
    }
  };

  // 工作区创建后的处理
  const handleWorkspaceCreated = async (ws) => {
    await workspace.onWorkspaceCreated(ws);
    await sidebarRef.value?.loadWorkspaces(); // 刷新侧边栏工作区列表
    // 新建工作区后自动切换并刷新界面
    if (ws) {
      await handleWorkspaceSwitch(ws);
    }
  };

  // 工作区删除后的处理
  const handleWorkspaceDeleted = async (deletedId) => {
    const wasCurrentWorkspace =
      workspace.currentWorkspace.value?.id === deletedId;
    await workspace.onWorkspaceDeleted(deletedId);

    // 如果删除的是当前选中的工作区，清空所有数据
    if (wasCurrentWorkspace) {
      tabs.value = [];
      activeTab.value = 0;
      tabsModule.collectionTabsData.value = {};
      // 重置编排状态
      selectedOrchestration.value = null;
      await environment.loadEnvironments();
      await responseModule.loadCookies();
    }

    await sidebarRef.value?.loadWorkspaces();
  };

  // 更新选中的工作区数据（同步/更新后刷新）
  const handleWorkspaceUpdated = async (ws) => {
    if (!ws) return;
    await workspace.loadWorkspaces();
    if (responseModule.selectedWorkspace.value?.id === ws.id) {
      responseModule.selectedWorkspace.value = ws;
    }
    if (workspace.currentWorkspace.value?.id === ws.id) {
      workspace.currentWorkspace.value = ws;
    }
  };

  // 分支切换后刷新环境（仅当前工作区）
  const handleBranchSwitched = async (ws) => {
    // 更新当前工作区的分支信息
    workspace.currentWorkspace.value = ws;
    // 刷新环境变量
    await environment.loadEnvironments();
    await sidebarRef.value?.loadEnvironments();
  };

  // 工作区导入后的处理
  const handleWorkspaceImported = async (ws) => {
    await workspace.loadWorkspaces();
    await sidebarRef.value?.loadWorkspaces();
    if (ws) {
      await handleWorkspaceSwitch(ws);
    }
  };

  // 返回所有需要的状态和方法
  return {
    // 工作区
    currentWorkspace: workspace.currentWorkspace,
    workspaces: workspace.workspaces,
    showWorkspaceDialog: workspace.showWorkspaceDialog,
    workspaceDialogMode: workspace.workspaceDialogMode,
    sidebarRef,
    loadWorkspaces: workspace.loadWorkspaces,
    openCreateWorkspace: workspace.openCreateWorkspace,
    closeWorkspaceDialog: workspace.closeWorkspaceDialog,
    onWorkspaceCreated: handleWorkspaceCreated,
    onWorkspaceDeleted: handleWorkspaceDeleted,
    onSwitchWorkspace: handleWorkspaceSwitch,
    showWorkspaceInfo: responseModule.showWorkspaceInfo,
    selectedWorkspace: responseModule.selectedWorkspace,
    onSelectWorkspace: responseModule.onSelectWorkspace,
    onWorkspaceUpdated: handleWorkspaceUpdated,
    onBranchSwitched: handleBranchSwitched,
    onWorkspaceImported: handleWorkspaceImported,

    // 标签页
    tabs,
    displayTabs: tabsModule.displayTabs,
    activeTab,
    collectionTabsData: tabsModule.collectionTabsData,
    selectedCollection: tabsModule.selectedCollection,
    selectCollection: tabsModule.selectCollection,
    showCollectionSettings: tabsModule.showCollectionSettings,
    onCollectionSettingsSaved: tabsModule.onCollectionSettingsSaved,
    closeTab: tabsModule.closeTab,
    closeAllTabs: tabsModule.closeAllTabs,
    closeOtherTabs: tabsModule.closeOtherTabs,
    onDeleteApis: tabsModule.onDeleteApis,
    onDeleteCollection: tabsModule.onDeleteCollection,

    // 请求
    currentRequest,
    currentRequestTab,
    response,
    loading,
    testResults: requestModule.testResults,
    sseEvents: requestModule.sseEvents,
    selectApi: async (apiOrId) => {
      responseModule.closeSavedResponseDoc();
      return requestModule.selectApi(apiOrId);
    },
    sendRequest: requestModule.sendRequest,
    saveRequest: requestModule.saveRequest,
    updateRequest: requestModule.updateRequest,
    onRenameApi: requestModule.onRenameApi,
    onUpdateRequestTab: tabsModule.onUpdateRequestTab,
    showRequestResponse: tabsModule.showRequestResponse,

    // 环境
    environments: environment.environments,
    activeEnvironmentId: environment.activeEnvironmentId,
    activeEnvironment: environment.activeEnvironment,
    selectedEnvironment: environment.selectedEnvironment,
    activeVariables: environment.activeVariables,
    availableVariables: environment.availableVariables,
    loadEnvironments: environment.loadEnvironments,
    loadActiveVariables: environment.loadActiveVariables,
    switchEnvironment: environment.switchEnvironment,
    selectEnvironment: environment.selectEnvironment,
    saveEnvironment: environment.saveEnvironment,
    deleteEnvironment: environment.deleteEnvironment,
    saveEnvVariables: environment.saveEnvVariables,
    onSwitchEnvironment: environment.onSwitchEnvironment,
    onSelectEnvironment: environment.onSelectEnvironment,
    showEnvironmentInfo: environment.showEnvironmentInfo,

    // Cookie
    cookies: responseModule.cookies,
    showCookiePanel: responseModule.showCookiePanel,
    loadCookies: responseModule.loadCookies,
    openCookiePanel: responseModule.openCookiePanel,
    closeCookiePanel: responseModule.closeCookiePanel,

    // Console
    showConsolePanel: requestModule.showConsolePanel,
    consoleLogs: requestModule.consoleLogs,
    openConsolePanel: requestModule.openConsolePanel,
    closeConsolePanel: requestModule.closeConsolePanel,
    clearConsoleLogs: requestModule.clearConsoleLogs,

    // 保存响应
    showSaveResponseDialog: responseModule.showSaveResponseDialog,
    saveResponseDefaultName: responseModule.saveResponseDefaultName,
    onSaveResponse: responseModule.onSaveResponse,
    handleSaveResponse: responseModule.handleSaveResponse,
    onSelectSavedResponse: responseModule.onSelectSavedResponse,
    showSavedResponseDoc: responseModule.showSavedResponseDoc,
    selectedSavedResponse: responseModule.selectedSavedResponse,
    closeSavedResponseDoc: responseModule.closeSavedResponseDoc,

    // 历史
    selectedHistoryEntry: responseModule.selectedHistoryEntry,
    onSelectHistory: responseModule.onSelectHistory,
    showHistoryDetail: responseModule.showHistoryDetail,

    // 导航
    currentNavKey,
    onNavChange,

    // Chat
    showChatPanel,
    chatSessionId,
    onSelectChatSession: handleSelectChatSession,
    onNewChatSession: handleNewChatSession,
    onSessionCreated: handleSessionCreated,

    // Orchestration
    showOrchestrationPanel,
    selectedOrchestration,
    onSelectOrchestration: handleSelectOrchestration,
    onOrchestrationStepsChanged: handleOrchestrationStepsChanged,

    // 设置
    showSettingsPanel: settings.showSettingsPanel,
    settingsCategory: settings.settingsCategory,
    openSettings: settings.openSettings,
    closeSettings: settings.closeSettings,
    showAiSettingsPanel: settings.showAiSettingsPanel,
    openAiSettings: settings.openAiSettings,
    closeAiSettings: settings.closeAiSettings,
    showGitBackupPanel: settings.showGitBackupPanel,
    openGitBackup: settings.openGitBackup,
    closeGitBackup: settings.closeGitBackup,

    // WebSocket 导航
    showWebSocketPanel,
    selectedWsConfig,
    onSelectWsConfig: handleSelectWsConfig,
    onCreateWsConfig: handleCreateWsConfig,
  };
}
