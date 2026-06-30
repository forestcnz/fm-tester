<script setup>
import { useSidebarSetup } from "./index.js";
import IconNav from "./IconNav/index.vue";
import CollectionPanel from "./CollectionPanel/index.vue";
import EnvironmentPanel from "./EnvironmentPanel/index.vue";
import WorkspacePanel from "./WorkspacePanel/index.vue";
import HistoryPanel from "./HistoryPanel/index.vue";
import ChatHistoryPanel from "./ChatHistoryPanel/index.vue";
import OrchestrationPanel from "./OrchestrationPanel/index.vue";
import WebSocketPanel from "./WebSocketPanel/index.vue";

const props = defineProps({
  workspace: Object,
});

const emit = defineEmits([
  "selectApi",
  "selectCollection",
  "deleteCollection",
  "selectEnvironment",
  "createWorkspace",
  "renameApi",
  "deleteApis",
  "navChange",
  "environmentUpdated",
  "workspaceDeleted",
  "selectSavedResponse",
  "selectHistory",
  "selectWorkspace",
  "workspaceUpdated",
  "branchSwitched",
  "selectChatSession",
  "newChatSession",
  "sessionCreated",
  "selectOrchestration",
  "workspaceImported",
  "selectWsConfig",
  "createWsConfig",
]);

const {
  activeNavKey,
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
} = useSidebarSetup(props, emit);

defineExpose({
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
});
</script>

<template>
  <div class="sidebar">
    <!-- 图标导航 -->
    <IconNav :active-key="activeNavKey" @nav-change="handleNavChange" />

    <!-- 面板内容区域 -->
    <div class="panel-container">
      <!-- 集合面板 -->
      <CollectionPanel
        v-if="activeNavKey === 'collection'"
        ref="collectionPanelRef"
        :workspace="props.workspace"
        @select-api="handleSelectApi"
        @select-collection="handleSelectCollection"
        @delete-apis="handleDeleteApis"
        @delete-collection="handleDeleteCollection"
        @rename-api="handleRenameApi"
        @select-saved-response="handleSelectSavedResponse"
      />

      <!-- WebSocket 面板 -->
      <WebSocketPanel
        v-if="activeNavKey === 'websocket'"
        ref="websocketPanelRef"
        :workspace="props.workspace"
        @select-ws-config="handleSelectWsConfig"
        @create-ws-config="handleCreateWsConfig"
      />

      <!-- 编排面板 -->
      <OrchestrationPanel
        v-if="activeNavKey === 'orchestration'"
        ref="orchestrationPanelRef"
        :workspace="props.workspace"
        @select-orchestration="handleSelectOrchestration"
      />

      <!-- 环境面板 -->
      <EnvironmentPanel
        v-if="activeNavKey === 'environment'"
        ref="environmentPanelRef"
        :workspace="props.workspace"
        @select-environment="handleSelectEnvironment"
        @environment-updated="handleEnvironmentUpdated"
      />

      <!-- 工作区面板 -->
      <WorkspacePanel
        v-if="activeNavKey === 'workspace'"
        ref="workspacePanelRef"
        :workspace="props.workspace"
        @select-workspace="handleSelectWorkspace"
        @create-workspace="handleCreateWorkspace"
        @workspace-deleted="handleWorkspaceDeleted"
        @workspace-updated="handleWorkspaceUpdated"
        @branch-switched="handleBranchSwitched"
        @workspace-imported="handleWorkspaceImported"
      />

      <!-- 历史面板 -->
      <HistoryPanel
        v-if="activeNavKey === 'history'"
        ref="historyPanelRef"
        :workspace="props.workspace"
        @select-history="handleSelectHistory"
      />

      <!-- 聊天会话面板 -->
      <ChatHistoryPanel
        v-if="activeNavKey === 'chat'"
        ref="chatHistoryPanelRef"
        :workspace="props.workspace"
        @select-session="handleSelectChatSession"
        @new-session="handleNewChatSession"
        @session-created="handleSessionCreated"
      />
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
