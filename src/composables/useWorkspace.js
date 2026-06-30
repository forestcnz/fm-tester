import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { showToast } from "./useToast";

/**
 * 工作区管理 composable
 */
export function useWorkspace() {
  const { t } = useI18n();
  const currentWorkspace = ref(null);
  const workspaces = ref([]);
  const showWorkspaceDialog = ref(false);
  const workspaceDialogMode = ref("create");

  const loadWorkspaces = async () => {
    try {
      const list = await invoke("get_workspaces");
      workspaces.value = list || [];
    } catch (e) {
      console.error("加载工作区列表失败:", e);
    }
  };

  const loadLastWorkspace = async () => {
    try {
      await loadWorkspaces();
      const workspace = await invoke("get_last_workspace");
      if (workspace) {
        currentWorkspace.value = workspace;
      }
      return workspace;
    } catch (e) {
      console.error("加载工作区失败:", e);
      return null;
    }
  };

  const openCreateWorkspace = () => {
    workspaceDialogMode.value = "create";
    showWorkspaceDialog.value = true;
  };

  const closeWorkspaceDialog = () => {
    showWorkspaceDialog.value = false;
  };

  const onWorkspaceCreated = async (workspace) => {
    await loadWorkspaces(); // 重新加载工作区列表
    // 返回工作区信息，由 App.js 决定是否切换
    return workspace;
  };

  const onWorkspaceDeleted = async (deletedId) => {
    workspaces.value = workspaces.value.filter((w) => w.id !== deletedId);

    // 如果删除的是当前选中的工作区，直接清空
    if (currentWorkspace.value?.id === deletedId) {
      currentWorkspace.value = null;
    }
  };

  const onSwitchWorkspace = async (workspace) => {
    currentWorkspace.value = workspace;
    if (workspace?.id) {
      try {
        await invoke("set_last_workspace", { workspaceId: workspace.id });
        showToast(t("toast.workspaceSwitched"), "success");
      } catch (e) {
        console.error("保存工作区失败:", e);
        showToast(t("toast.workspaceSwitchFailed"), "error");
      }
    }
  };

  return {
    currentWorkspace,
    workspaces,
    showWorkspaceDialog,
    workspaceDialogMode,
    loadWorkspaces,
    loadLastWorkspace,
    openCreateWorkspace,
    closeWorkspaceDialog,
    onWorkspaceCreated,
    onWorkspaceDeleted,
    onSwitchWorkspace,
  };
}
