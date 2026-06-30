import { ref, watch, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { showToast } from "../../../composables/useToast";
import { useDialogEscape } from "../../../composables/useDialogStack.js";

export function useOrchestrationPanelSetup(props, emit) {
  const { t } = useI18n();

  const orchestrations = ref([]);
  const selectedOrchestration = ref(null);
  const editingItem = ref(null);
  const editingName = ref("");
  const isSavingEdit = ref(false);

  const contextMenu = ref({
    visible: false,
    x: 0,
    y: 0,
    item: null,
  });

  // 拖拽排序状态
  const DRAG_THRESHOLD = 4;
  const dragState = ref({
    draggingId: null,
    dragOverId: null,
    dragOverPosition: null,
  });
  const isDragging = ref(false);
  const dragStartY = ref(0);
  const dragStartId = ref(null);
  const dragListenersAdded = ref(false); // 跟踪监听器是否已添加

  const onMouseDown = (e, orch) => {
    // 如果有正在编辑的项，先完成编辑
    if (editingItem.value && editingItem.value.id !== orch.id) {
      finishInlineEdit();
    }

    if (e.button !== 0) return; // 只响应左键
    dragStartY.value = e.clientY;
    dragStartId.value = orch.id;
    isDragging.value = false;
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
    dragListenersAdded.value = true;
  };

  const onMouseMove = (e) => {
    if (!dragStartId.value) return;

    const deltaY = Math.abs(e.clientY - dragStartY.value);
    if (!isDragging.value && deltaY < DRAG_THRESHOLD) return;

    if (!isDragging.value) {
      isDragging.value = true;
      dragState.value = {
        draggingId: dragStartId.value,
        dragOverId: null,
        dragOverPosition: null,
      };
    }

    const orchEls = document.querySelectorAll(".orchestration-item");
    for (const el of orchEls) {
      const rect = el.getBoundingClientRect();
      const id = el.dataset.orchId;
      if (id && id !== dragState.value.draggingId) {
        const midY = rect.top + rect.height / 2;
        if (e.clientY >= rect.top && e.clientY <= rect.bottom) {
          dragState.value.dragOverId = id;
          dragState.value.dragOverPosition =
            e.clientY < midY ? "before" : "after";
          return;
        }
      }
    }
    dragState.value.dragOverId = null;
    dragState.value.dragOverPosition = null;
  };

  const onMouseUp = async () => {
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
    dragListenersAdded.value = false;

    if (isDragging.value && dragState.value.dragOverId) {
      const dragIndex = orchestrations.value.findIndex(
        (o) => o.id === dragState.value.draggingId,
      );
      const overIndex = orchestrations.value.findIndex(
        (o) => o.id === dragState.value.dragOverId,
      );
      if (dragIndex !== -1 && overIndex !== -1 && dragIndex !== overIndex) {
        const newOrder = [...orchestrations.value.map((o) => o.id)];
        const [removed] = newOrder.splice(dragIndex, 1);
        const insertIndex =
          dragState.value.dragOverPosition === "before"
            ? dragIndex < overIndex
              ? overIndex - 1
              : overIndex
            : dragIndex < overIndex
              ? overIndex
              : overIndex + 1;
        newOrder.splice(insertIndex, 0, removed);
        await reorderOrchestrations(newOrder);
      }
    }

    dragState.value = {
      draggingId: null,
      dragOverId: null,
      dragOverPosition: null,
    };
    isDragging.value = false;
    dragStartId.value = null;
  };

  const reorderOrchestrations = async (orchestrationIds) => {
    if (!props.workspace?.id) return;
    try {
      await invoke("reorder_orchestrations_cmd", {
        workspaceId: props.workspace.id,
        orchestrationIds,
      });
      await loadOrchestrations();
    } catch (e) {
      console.error("编排排序失败:", e);
      showToast(t("toast.saveFailed"), "error");
    }
  };

  // 组件卸载时强制清理事件监听器，防止内存泄漏
  onUnmounted(() => {
    if (dragListenersAdded.value) {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
      dragListenersAdded.value = false;
    }
  });

  const loadOrchestrations = async () => {
    if (!props.workspace?.id) return;
    try {
      const data = await invoke("get_orchestrations", {
        workspaceId: props.workspace.id,
      });
      orchestrations.value = data?.orchestrations || [];
    } catch (e) {
      console.error("加载编排失败:", e);
      orchestrations.value = [];
    }
  };

  const selectOrchestrationItem = (orch) => {
    selectedOrchestration.value = orch.id;
    emit("selectOrchestration", orch);
  };

  const openContextMenu = (event, item) => {
    event.preventDefault();
    event.stopPropagation();
    contextMenu.value = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      item: item,
    };
  };

  const closeContextMenu = () => {
    contextMenu.value.visible = false;
  };

  useDialogEscape(() => contextMenu.value.visible, closeContextMenu);

  const startInlineEdit = (isNew = true, item = null) => {
    if (isNew) {
      editingItem.value = {
        isNew: true,
        id: `temp-${Date.now()}`,
      };
      editingName.value = t("orchestration.newOrchestration");
    } else {
      editingItem.value = {
        isNew: false,
        id: item.id,
      };
      editingName.value = item.name;
    }
  };

  const finishInlineEdit = async () => {
    if (!editingItem.value) return;
    if (isSavingEdit.value) return;
    isSavingEdit.value = true;

    const name = editingName.value.trim();
    if (!name) {
      cancelInlineEdit();
      isSavingEdit.value = false;
      return;
    }

    if (!props.workspace?.id) {
      cancelInlineEdit();
      isSavingEdit.value = false;
      return;
    }

    try {
      if (editingItem.value.isNew) {
        const newOrch = await invoke("create_orchestration_cmd", {
          workspaceId: props.workspace.id,
          name,
          description: null,
        });
        await loadOrchestrations();
        cancelInlineEdit();
        selectOrchestrationItem(newOrch);
        showToast(t("toast.orchestrationCreated"), "success");
      } else {
        await invoke("update_orchestration_cmd", {
          workspaceId: props.workspace.id,
          orchestrationId: editingItem.value.id,
          name,
          description: null,
        });
        await loadOrchestrations();
        cancelInlineEdit();
        showToast(t("toast.orchestrationRenamed"), "success");
      }
    } catch (e) {
      console.error("保存失败:", e);
      showToast(t("toast.orchestrationRenameFailed"), "error");
      cancelInlineEdit();
    } finally {
      isSavingEdit.value = false;
    }
  };

  const cancelInlineEdit = () => {
    editingItem.value = null;
    editingName.value = "";
  };

  const handleEditKeydown = (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      finishInlineEdit();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelInlineEdit();
    }
  };

  const handleContextAction = async (action) => {
    const { item } = contextMenu.value;

    if (action === "new") {
      startInlineEdit(true);
    } else if (action === "rename") {
      startInlineEdit(false, item);
    } else if (action === "delete") {
      await deleteOrchestration(item);
    }

    closeContextMenu();
  };

  const deleteOrchestration = async (orch) => {
    if (!props.workspace?.id) return;
    try {
      await invoke("delete_orchestration_cmd", {
        workspaceId: props.workspace.id,
        orchestrationId: orch.id,
      });
      await loadOrchestrations();
      selectedOrchestration.value = null;
      emit("selectOrchestration", null);
      showToast(t("toast.orchestrationDeleted"), "success");
    } catch (e) {
      console.error("删除编排失败:", e);
      showToast(t("toast.orchestrationDeleteFailed"), "error");
    }
  };

  watch(
    () => props.workspace,
    async (ws, oldWs) => {
      // 工作区变化时重置选中状态
      if (ws?.id !== oldWs?.id) {
        selectedOrchestration.value = null;
      }
      if (ws) {
        await loadOrchestrations();
      }
    },
    { immediate: true },
  );

  return {
    orchestrations,
    selectedOrchestration,
    editingItem,
    editingName,
    isSavingEdit,
    contextMenu,
    loadOrchestrations,
    selectOrchestrationItem,
    openContextMenu,
    closeContextMenu,
    startInlineEdit,
    finishInlineEdit,
    cancelInlineEdit,
    handleEditKeydown,
    handleContextAction,
    deleteOrchestration,
    dragState,
    onMouseDown,
  };
}
