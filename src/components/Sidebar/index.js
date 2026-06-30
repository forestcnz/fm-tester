import { ref, watch, onMounted } from "vue";

export const navItems = [
  { icon: "collection", nameKey: "nav.collections", key: "collection" },
  { icon: "websocket", nameKey: "nav.websocket", key: "websocket" },
  { icon: "environment", nameKey: "nav.environments", key: "environment" },
  { icon: "workspace", nameKey: "nav.workspaces", key: "workspace" },
  { icon: "orchestration", nameKey: "nav.orchestration", key: "orchestration" },
  { icon: "chat", nameKey: "nav.chat", key: "chat" },
  { icon: "history", nameKey: "nav.history", key: "history" },
];

export function useSidebarSetup(props, emit) {
  const activeNavKey = ref("collection");

  const collectionPanelRef = ref(null);
  const environmentPanelRef = ref(null);
  const workspacePanelRef = ref(null);
  const historyPanelRef = ref(null);
  const chatHistoryPanelRef = ref(null);
  const orchestrationPanelRef = ref(null);
  const websocketPanelRef = ref(null);

  // 处理导航切换
  const handleNavChange = (key) => {
    activeNavKey.value = key;
    emit("navChange", key);
  };

  // 处理子组件事件转发
  const handleSelectApi = (api) => emit("selectApi", api);
  const handleSelectCollection = (collection) =>
    emit("selectCollection", collection);
  const handleDeleteApis = (ids) => emit("deleteApis", ids);
  const handleDeleteCollection = (collectionId) =>
    emit("deleteCollection", collectionId);
  const handleRenameApi = (api) => emit("renameApi", api);
  const handleSelectEnvironment = (envId) => emit("selectEnvironment", envId);
  const handleEnvironmentUpdated = () => emit("environmentUpdated");
  const handleSelectWorkspace = (ws) => emit("selectWorkspace", ws);
  const handleCreateWorkspace = () => emit("createWorkspace");
  const handleWorkspaceDeleted = (wsId) => emit("workspaceDeleted", wsId);
  const handleWorkspaceUpdated = (ws) => emit("workspaceUpdated", ws);
  const handleBranchSwitched = (ws) => emit("branchSwitched", ws);

  // 处理已保存响应事件
  const handleSelectSavedResponse = (item) => emit("selectSavedResponse", item);

  // 处理历史选择事件
  const handleSelectHistory = (entry) => emit("selectHistory", entry);

  const handleSelectChatSession = (session) =>
    emit("selectChatSession", session);
  const handleNewChatSession = () => emit("newChatSession");
  const handleSessionCreated = (sessionId) => emit("sessionCreated", sessionId);

  const handleSelectOrchestration = (orch) => emit("selectOrchestration", orch);

  // WebSocket 配置事件
  const handleSelectWsConfig = (config) => emit("selectWsConfig", config);
  const handleCreateWsConfig = () => emit("createWsConfig");

  const handleWorkspaceImported = async (ws) => {
    emit("workspaceImported", ws);
  };

  const loadWorkspaces = async () => {
    if (workspacePanelRef.value) {
      await workspacePanelRef.value.loadWorkspaces();
    }
  };

  const loadCollections = async () => {
    if (collectionPanelRef.value) {
      await collectionPanelRef.value.loadCollections();
    }
  };

  const loadEnvironments = async () => {
    if (environmentPanelRef.value) {
      await environmentPanelRef.value.loadEnvironments();
    }
  };

  const loadHistory = async () => {
    if (historyPanelRef.value) {
      await historyPanelRef.value.loadHistoryDates();
    }
  };

  const loadChatSessions = async () => {
    if (chatHistoryPanelRef.value) {
      await chatHistoryPanelRef.value.loadSessions();
    }
  };

  const loadOrchestrations = async () => {
    if (orchestrationPanelRef.value) {
      await orchestrationPanelRef.value.loadOrchestrations();
    }
  };

  // 设置选中 API
  const setSelectedApi = (apiId) => {
    if (collectionPanelRef.value) {
      collectionPanelRef.value.setSelectedApiId(apiId);
    }
  };

  // 设置选中集合
  const setSelectedCollection = (collectionId) => {
    if (collectionPanelRef.value) {
      collectionPanelRef.value.setSelectedCollectionId(collectionId);
    }
  };

  const refreshApiSavedResponses = async (apiId) => {
    if (collectionPanelRef.value) {
      await collectionPanelRef.value.refreshApiSavedResponses(apiId);
    }
  };

  // 刷新侧边栏中指定 API 的显示信息（不重新加载整个集合树）
  const refreshApiInSidebar = (apiId, method) => {
    if (collectionPanelRef.value) {
      collectionPanelRef.value.refreshApiInSidebar(apiId, method);
    }
  };

  // 监听工作区变化
  watch(
    () => props.workspace,
    async (ws) => {
      if (ws) {
        await loadCollections();
        await loadEnvironments();
      }
    },
    { immediate: true },
  );

  // 启动时加载工作区列表
  onMounted(async () => {
    await loadWorkspaces();
  });

  watch(activeNavKey, async (key) => {
    if (key === "workspace") {
      await loadWorkspaces();
    } else if (key === "collection") {
      await loadCollections();
    } else if (key === "environment") {
      await loadEnvironments();
    } else if (key === "history") {
      await loadHistory();
    } else if (key === "chat") {
      await loadChatSessions();
    } else if (key === "orchestration") {
      await loadOrchestrations();
    }
    // websocket 导航不需要加载额外数据
  });

  return {
    activeNavKey,
    navItems,

    collectionPanelRef,
    environmentPanelRef,
    workspacePanelRef,
    historyPanelRef,
    chatHistoryPanelRef,
    orchestrationPanelRef,
    websocketPanelRef,

    handleNavChange,
    handleSelectApi,
    handleSelectCollection,
    handleDeleteApis,
    handleDeleteCollection,
    handleRenameApi,
    handleSelectEnvironment,
    handleEnvironmentUpdated,
    handleSelectWorkspace,
    handleCreateWorkspace,
    handleWorkspaceDeleted,
    handleWorkspaceUpdated,
    handleBranchSwitched,
    handleSelectSavedResponse,
    handleSelectHistory,
    handleSelectChatSession,
    handleNewChatSession,
    handleSessionCreated,
    handleSelectOrchestration,
    handleWorkspaceImported,
    handleSelectWsConfig,
    handleCreateWsConfig,

    loadWorkspaces,
    loadCollections,
    loadEnvironments,
    loadHistory,
    loadChatSessions,
    loadOrchestrations,
    setSelectedApi,
    setSelectedCollection,
    refreshApiSavedResponses,
    refreshApiInSidebar,
  };
}
