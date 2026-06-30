import { ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { useDialogEscape } from "../../../composables/useDialogStack.js";
import { useWorkspaceIO } from "../../../composables/useWorkspaceIO";
import { showToast } from "../../../composables/useToast.js";

export function useWorkspacePanelSetup(props, emit) {
  const { t } = useI18n();
  const { exportWorkspace, exporting } = useWorkspaceIO();

  const workspaces = ref([]);
  const currentWorkspace = ref(null);
  const showImportDialog = ref(false);
  const restoreDialog = ref({ visible: false, targetWorkspace: null });
  const backingUp = ref(false);

  const renamingId = ref(null);
  const renameValue = ref("");

  watch(
    () => props.workspace,
    (ws) => {
      currentWorkspace.value = ws;
    },
    { immediate: true },
  );

  const wsContextMenu = ref({
    visible: false,
    x: 0,
    y: 0,
    ws: null,
  });

  const blankContextMenu = ref({
    visible: false,
    x: 0,
    y: 0,
  });

  const dragState = ref({
    draggingId: null,
    dragOverId: null,
    dragPosition: null,
  });
  let isDragging = false;
  let dragStartY = 0;
  let dragStartId = null;
  let dragListenersAdded = false;
  const DRAG_THRESHOLD = 4;

  const loadWorkspaces = async () => {
    try {
      workspaces.value = (await invoke("get_workspaces")) || [];
    } catch (e) {
      console.error("加载工作区失败:", e);
      workspaces.value = [];
    }
  };

  onMounted(async () => {
    await loadWorkspaces();
  });

  const selectWorkspace = (workspace) => {
    currentWorkspace.value = workspace;
    emit("selectWorkspace", workspace);
  };

  const createWorkspace = () => {
    emit("createWorkspace");
  };

  const openImportDialog = () => {
    showImportDialog.value = true;
  };

  const deleteWorkspace = async (ws) => {
    try {
      await invoke("delete_workspace", { id: ws.id });
      await loadWorkspaces();
      emit("workspaceDeleted", ws.id);
    } catch (e) {
      console.error("删除工作区失败:", e);
    }
  };

  const startRename = (ws) => {
    renamingId.value = ws.id;
    renameValue.value = ws.name;
    nextTick(() => {
      const el = document.querySelector(".ws-rename-input");
      el?.focus();
      el?.select();
    });
  };

  const cancelRename = () => {
    renamingId.value = null;
    renameValue.value = "";
  };

  const confirmRename = async (ws) => {
    const newName = renameValue.value.trim();
    if (!newName || newName === ws.name) {
      cancelRename();
      return;
    }
    try {
      await invoke("update_workspace", {
        id: ws.id,
        name: newName,
        description: ws.description,
      });
      await loadWorkspaces();
      const refreshed = workspaces.value.find((w) => w.id === ws.id);
      if (refreshed) emit("workspaceUpdated", refreshed);
      cancelRename();
    } catch (e) {
      console.error("重命名工作区失败:", e);
      cancelRename();
    }
  };

  const openWsContextMenu = (event, ws) => {
    event.preventDefault();
    event.stopPropagation();

    wsContextMenu.value = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      ws: ws,
    };
  };

  const closeWsContextMenu = () => {
    wsContextMenu.value.visible = false;
  };

  const openBlankContextMenu = (event) => {
    event.preventDefault();
    event.stopPropagation();
    blankContextMenu.value = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
    };
  };

  const closeBlankContextMenu = () => {
    blankContextMenu.value.visible = false;
  };

  const handleImported = async (workspace) => {
    showImportDialog.value = false;
    await loadWorkspaces();
    emit("workspaceImported", workspace);
  };

  // 打开 Git 恢复对话框；传入 ws 表示覆盖该工作区，传 null 表示新建
  const openRestoreDialog = (ws) => {
    restoreDialog.value = { visible: true, targetWorkspace: ws };
  };

  const handleRestoreImported = async (workspace) => {
    restoreDialog.value = { visible: false, targetWorkspace: null };
    await loadWorkspaces();
    emit("workspaceImported", workspace);
  };

  const handleWsContextAction = async (action) => {
    const { ws } = wsContextMenu.value;

    if (action === "rename-ws") {
      if (ws) startRename(ws);
    } else if (action === "delete-ws") {
      if (ws) {
        await deleteWorkspace(ws);
      }
    } else if (action === "export-ws") {
      if (ws && !exporting.value) {
        await exportWorkspace(ws.id);
      }
    } else if (action === "backup-ws") {
      if (ws && !backingUp.value) {
        try {
          backingUp.value = true;
          await invoke("backup_workspace", { workspaceId: ws.id });
          showToast(t("gitBackup.backupSuccess"), "success");
          await loadWorkspaces();
          const refreshed = workspaces.value.find((w) => w.id === ws.id);
          if (refreshed) emit("workspaceUpdated", refreshed);
        } catch (e) {
          console.error(e);
          showToast(`${t("gitBackup.backupFailed")}: ${e}`, "error");
        } finally {
          backingUp.value = false;
        }
      }
    } else if (action === "restore-ws") {
      openRestoreDialog(ws);
    }

    closeWsContextMenu();
  };

  // 空白处右键菜单处理（与具体工作区无关的操作）
  const handleBlankContextAction = async (action) => {
    if (action === "restore-ws") {
      openRestoreDialog(null);
    }
    closeBlankContextMenu();
  };

  const getItemEl = (id) => {
    return document.querySelector(`[data-item-id="${id}"]`);
  };

  const onMouseDown = (e, ws) => {
    if (e.button !== 0) return;
    dragStartY = e.clientY;
    dragStartId = ws.id;
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

    const targetIndex = workspaces.value.findIndex(
      (ws) => ws.id === dragOverId,
    );
    if (targetIndex === -1) return;

    const newIndex = dragPosition === "before" ? targetIndex : targetIndex + 1;

    try {
      await invoke("reorder_workspaces", {
        workspaceId: draggingId,
        newIndex,
      });
      await loadWorkspaces();
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
    for (const ws of workspaces.value) {
      if (ws.id === dragState.value.draggingId) continue;

      const el = getItemEl(ws.id);
      if (!el) continue;

      const rect = el.getBoundingClientRect();
      if (clientY >= rect.top && clientY <= rect.bottom) {
        const midY = rect.top + rect.height / 2;
        const position = clientY < midY ? "before" : "after";
        return { id: ws.id, position };
      }
    }
    return null;
  };

  const handleGlobalClick = () => {
    closeWsContextMenu();
    closeBlankContextMenu();
  };

  useDialogEscape(() => wsContextMenu.value.visible, closeWsContextMenu);
  useDialogEscape(() => blankContextMenu.value.visible, closeBlankContextMenu);
  useDialogEscape(showImportDialog, () => {
    showImportDialog.value = false;
  });

  onMounted(() => {
    document.addEventListener("click", handleGlobalClick);
  });

  onUnmounted(() => {
    document.removeEventListener("click", handleGlobalClick);
    if (dragListenersAdded) {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
      dragListenersAdded = false;
    }
  });

  return {
    workspaces,
    currentWorkspace,
    wsContextMenu,
    blankContextMenu,
    dragState,
    showImportDialog,
    restoreDialog,
    backingUp,
    renamingId,
    renameValue,
    startRename,
    confirmRename,
    cancelRename,
    loadWorkspaces,
    selectWorkspace,
    createWorkspace,
    openImportDialog,
    openWsContextMenu,
    closeWsContextMenu,
    handleWsContextAction,
    openBlankContextMenu,
    closeBlankContextMenu,
    handleBlankContextAction,
    onMouseDown,
    handleImported,
    handleRestoreImported,
  };
}
