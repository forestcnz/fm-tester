import { ref, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useDialogEscape } from "../../../composables/useDialogStack.js";

export function useEnvironmentPanelSetup(props, emit) {
  // 环境数据
  const environments = ref([]);
  const activeEnvironmentId = ref(null);
  const showEnvDialog = ref(false);
  const editingEnv = ref(null);
  const editingEnvName = ref("");
  const editingEnvVariables = ref([]);

  // 环境右键菜单
  const envContextMenu = ref({
    visible: false,
    x: 0,
    y: 0,
    env: null,
    type: "", // 'root' 或 'env'
  });

  // 拖拽排序状态
  const dragState = ref({
    draggingId: null,
    dragOverId: null,
    dragPosition: null, // 'before' | 'after'
  });
  let isDragging = false;
  let dragStartY = 0;
  let dragStartId = null;
  let dragListenersAdded = false; // 跟踪监听器是否已添加
  const DRAG_THRESHOLD = 4;

  // 加载环境列表
  const loadEnvironments = async () => {
    if (!props.workspace?.id) return;
    try {
      const config = await invoke("get_environments", {
        workspaceId: props.workspace.id,
      });
      environments.value = config.environments || [];
      activeEnvironmentId.value = config.active_environment_id || null;
    } catch (e) {
      console.error("加载环境失败:", e);
      environments.value = [];
    }
  };

  // 监听 workspace 变化，自动加载环境
  watch(
    () => props.workspace,
    async (ws) => {
      if (ws) {
        await loadEnvironments();
      }
    },
    { immediate: true },
  );

  // 选择环境（仅选中效果，通知父组件更新显示）
  const selectEnvironment = (envId) => {
    activeEnvironmentId.value = envId;
    emit("selectEnvironment", envId);
  };

  // 打开新建环境对话框
  const openCreateEnvDialog = () => {
    editingEnv.value = null;
    editingEnvName.value = "";
    editingEnvVariables.value = [{ key: "", value: "", enabled: true }];
    showEnvDialog.value = true;
  };

  // 打开编辑环境对话框
  const openEditEnvDialog = (env) => {
    editingEnv.value = env;
    editingEnvName.value = env.name;
    editingEnvVariables.value =
      env.variables.length > 0
        ? [...env.variables]
        : [{ key: "", value: "", enabled: true }];
    showEnvDialog.value = true;
  };

  // 添加变量行
  const addEnvVariable = () => {
    editingEnvVariables.value.push({ key: "", value: "", enabled: true });
  };

  // 删除变量行
  const removeEnvVariable = (index) => {
    if (editingEnvVariables.value.length > 1) {
      editingEnvVariables.value.splice(index, 1);
    }
  };

  // 保存环境
  const handleSaveEnv = async () => {
    if (!props.workspace?.id) return;
    if (!editingEnvName.value.trim()) return;

    // 编辑时保留原有变量和 headers，新建时为空
    let variables = [];
    let common_headers = null;
    if (editingEnv.value) {
      variables = editingEnv.value.variables || [];
      common_headers = editingEnv.value.common_headers || null;
    }

    const envId = editingEnv.value?.id || `env_${Date.now()}`;
    const environment = {
      id: envId,
      name: editingEnvName.value.trim(),
      variables,
      common_headers,
    };

    try {
      await invoke("save_environment", {
        workspaceId: props.workspace.id,
        environment,
      });
      await loadEnvironments();
      showEnvDialog.value = false;
      // 通知父组件刷新环境列表
      emit("environmentUpdated");
    } catch (e) {
      console.error("保存环境失败:", e);
    }
  };

  // 删除环境
  const deleteEnvironment = async (envId) => {
    if (!props.workspace?.id) return;
    try {
      await invoke("delete_environment", {
        workspaceId: props.workspace.id,
        environmentId: envId,
      });
      await loadEnvironments();
      // 通知父组件刷新环境列表
      emit("environmentUpdated");
    } catch (e) {
      console.error("删除环境失败:", e);
    }
  };

  // 打开环境右键菜单
  const openEnvContextMenu = (event, env, type) => {
    event.preventDefault();
    event.stopPropagation();

    envContextMenu.value = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      env: env,
      type: type,
    };
  };

  // 关闭环境右键菜单
  const closeEnvContextMenu = () => {
    envContextMenu.value.visible = false;
  };

  // 环境右键菜单操作
  const handleEnvContextAction = async (action) => {
    const { env } = envContextMenu.value;

    if (action === "new-env") {
      openCreateEnvDialog();
    } else if (action === "edit-env") {
      if (env) {
        openEditEnvDialog(env);
      }
    } else if (action === "delete-env") {
      if (env) {
        await deleteEnvironment(env.id);
      }
    }

    closeEnvContextMenu();
  };

  // ============ 拖拽排序 ============

  const getItemEl = (id) => {
    return document.querySelector(`[data-item-id="${id}"]`);
  };

  const onMouseDown = (e, env) => {
    if (e.button !== 0) return;
    dragStartY = e.clientY;
    dragStartId = env.id;
    isDragging = false;
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
    dragListenersAdded = true;
  };

  const onMouseMove = (e) => {
    if (!dragStartId) {
      cleanupDrag();
      return;
    }

    const deltaY = Math.abs(e.clientY - dragStartY);
    if (!isDragging && deltaY < DRAG_THRESHOLD) return;

    if (!isDragging) {
      isDragging = true;
      dragState.value = {
        draggingId: dragStartId,
        dragOverId: null,
        dragPosition: null,
      };
    }

    const target = findItemAtY(e.clientY);
    if (target) {
      dragState.value = {
        ...dragState.value,
        dragOverId: target.id,
        dragPosition: target.position,
      };
    } else {
      dragState.value = {
        ...dragState.value,
        dragOverId: null,
        dragPosition: null,
      };
    }
  };

  const onMouseUp = async (_e) => {
    if (
      isDragging &&
      dragState.value.dragOverId &&
      dragState.value.draggingId !== dragState.value.dragOverId
    ) {
      await performReorder();
    }
    cleanupDrag();
  };

  const performReorder = async () => {
    const { draggingId, dragOverId, dragPosition } = dragState.value;
    if (
      !draggingId ||
      !dragOverId ||
      !dragPosition ||
      draggingId === dragOverId
    )
      return;

    // 找到目标索引
    const targetIndex = environments.value.findIndex(
      (env) => env.id === dragOverId,
    );
    if (targetIndex === -1) return;

    const newIndex = dragPosition === "before" ? targetIndex : targetIndex + 1;

    try {
      await invoke("reorder_environments", {
        workspaceId: props.workspace.id,
        environmentId: draggingId,
        newIndex,
      });
      await loadEnvironments();
    } catch (e) {
      console.error("排序失败:", e);
    }
  };

  const cleanupDrag = () => {
    isDragging = false;
    dragStartId = null;
    dragStartY = 0;
    dragState.value = {
      draggingId: null,
      dragOverId: null,
      dragPosition: null,
    };
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
    dragListenersAdded = false;
  };

  const findItemAtY = (clientY) => {
    for (const env of environments.value) {
      if (env.id === dragState.value.draggingId) continue;

      const el = getItemEl(env.id);
      if (!el) continue;

      const rect = el.getBoundingClientRect();
      if (clientY >= rect.top && clientY <= rect.bottom) {
        const midY = rect.top + rect.height / 2;
        const position = clientY < midY ? "before" : "after";
        return { id: env.id, position };
      }
    }
    return null;
  };

  // 全局点击关闭右键菜单
  const handleGlobalClick = () => {
    closeEnvContextMenu();
  };

  // ESC 键关闭弹窗
  useDialogEscape(showEnvDialog, () => {
    showEnvDialog.value = false;
  });
  useDialogEscape(() => envContextMenu.value.visible, closeEnvContextMenu);

  onMounted(() => {
    document.addEventListener("click", handleGlobalClick);
  });

  onUnmounted(() => {
    document.removeEventListener("click", handleGlobalClick);
    // 强制清理拖拽监听器，防止内存泄漏
    if (dragListenersAdded) {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
      dragListenersAdded = false;
    }
  });

  return {
    // 状态
    environments,
    activeEnvironmentId,
    showEnvDialog,
    editingEnv,
    editingEnvName,
    editingEnvVariables,
    envContextMenu,
    dragState,
    // 函数
    loadEnvironments,
    selectEnvironment,
    openCreateEnvDialog,
    openEditEnvDialog,
    addEnvVariable,
    removeEnvVariable,
    handleSaveEnv,
    deleteEnvironment,
    openEnvContextMenu,
    closeEnvContextMenu,
    handleEnvContextAction,
    onMouseDown,
  };
}
