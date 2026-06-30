import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { showToast } from "../../composables/useToast";
import { useDialogEscape } from "../../composables/useDialogStack.js";

// 导出 composable 函数
export function useWorkspaceDialogSetup(props, emit) {
  const { t } = useI18n();

  // 表单数据
  const name = ref("");
  const description = ref("");
  const error = ref("");
  const loading = ref(false);

  // 工作区列表
  const workspaces = ref([]);
  const selectedWorkspaceId = ref(null);

  // 加载工作区列表
  const loadWorkspaces = async () => {
    try {
      workspaces.value = await invoke("get_workspaces");
    } catch (e) {
      console.error("加载工作区列表失败:", e);
    }
  };

  // 创建工作区
  const createWorkspace = async () => {
    if (!name.value.trim()) {
      error.value = "请输入工作区名称";
      return;
    }

    loading.value = true;
    error.value = "";

    try {
      const workspace = await invoke("create_workspace", {
        name: name.value,
        description: description.value,
      });

      showToast(t("toast.workspaceCreated"), "success");
      emit("created", workspace);
      close(); // 创建成功后关闭对话框
    } catch (e) {
      error.value = e;
    } finally {
      loading.value = false;
    }
  };

  // 选中工作区项
  const handleSelectWorkspaceItem = (id) => {
    selectedWorkspaceId.value = id;
  };

  // 选择工作区（确认按钮）
  const selectWorkspace = async () => {
    if (!selectedWorkspaceId.value) {
      error.value = "请选择一个工作区";
      return;
    }

    loading.value = true;
    error.value = "";

    try {
      const workspace = await invoke("switch_workspace", {
        id: selectedWorkspaceId.value,
      });
      emit("selected", workspace);
    } catch (e) {
      error.value = e;
    } finally {
      loading.value = false;
    }
  };

  // 重置表单
  const resetForm = () => {
    name.value = "";
    description.value = "";
    error.value = "";
  };

  // 关闭对话框
  const close = () => {
    resetForm();
    emit("close");
  };

  // ESC 键关闭
  useDialogEscape(() => props.visible, close);

  // 监听显示状态
  watch(
    () => props.visible,
    (visible) => {
      if (visible && props.mode === "select") {
        loadWorkspaces();
      }
    },
  );

  return {
    name,
    description,
    error,
    loading,
    workspaces,
    selectedWorkspaceId,
    loadWorkspaces,
    createWorkspace,
    handleSelectWorkspaceItem,
    selectWorkspace,
    resetForm,
    close,
  };
}
