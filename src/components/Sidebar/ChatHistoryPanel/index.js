import { ref, onMounted, onUnmounted, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useI18n } from "vue-i18n";
import { useDialogEscape } from "../../../composables/useDialogStack.js";

export function useChatHistorySetup(props, emit) {
  const { t } = useI18n();
  const sessions = ref([]);
  const activeSessionId = ref(null);
  const loading = ref(false);

  // 重命名状态
  const renamingSessionId = ref(null);
  const renamingTitle = ref("");

  // 右键菜单状态
  const showContextMenu = ref(false);
  const contextMenuPosition = ref({ x: 0, y: 0 });
  const contextMenuSession = ref(null);

  // 加载聊天会话列表
  const loadSessions = async () => {
    if (!props.workspace?.id) {
      sessions.value = [];
      return;
    }

    try {
      loading.value = true;
      const result = await invoke("get_chat_sessions", {
        workspaceId: props.workspace.id,
      });
      sessions.value = result || [];
    } catch (e) {
      console.error("Failed to load chat sessions:", e);
      sessions.value = [];
    } finally {
      loading.value = false;
    }
  };

  // 选择会话
  const selectSession = (session) => {
    activeSessionId.value = session.id;
    emit("select-session", session);
    closeContextMenu();
  };

  // 创建新会话
  const createNewSession = () => {
    activeSessionId.value = null;
    emit("new-session");
    closeContextMenu();
  };

  // 删除会话
  const deleteSession = async (sessionId) => {
    if (!props.workspace?.id) return;

    try {
      await invoke("delete_chat_session", {
        workspaceId: props.workspace.id,
        sessionId: sessionId,
      });

      // 如果删除的是当前活动的会话，清空选择
      if (activeSessionId.value === sessionId) {
        activeSessionId.value = null;
        emit("new-session");
      }

      // 重新加载会话列表
      await loadSessions();
    } catch (e) {
      console.error("Failed to delete chat session:", e);
    }
  };

  // 开始重命名
  const startRename = (session) => {
    renamingSessionId.value = session.id;
    renamingTitle.value = session.title || "";
    closeContextMenu();
    // 下一tick后聚焦输入框
    nextTick(() => {
      const input = document.querySelector(".rename-input");
      if (input) {
        input.focus();
      }
    });
  };

  // 取消重命名
  const cancelRename = () => {
    renamingSessionId.value = null;
    renamingTitle.value = "";
  };

  // 确认重命名
  const confirmRename = async (session) => {
    if (!props.workspace?.id || !renamingTitle.value.trim()) {
      cancelRename();
      return;
    }

    try {
      await invoke("rename_chat_session", {
        workspaceId: props.workspace.id,
        sessionId: session.id,
        newTitle: renamingTitle.value.trim(),
      });

      // 重新加载会话列表
      await loadSessions();
    } catch (e) {
      console.error("Failed to rename chat session:", e);
    } finally {
      cancelRename();
    }
  };

  // 处理右键菜单
  const handleContextMenu = (session, event) => {
    event.stopPropagation();
    contextMenuSession.value = session;
    contextMenuPosition.value = {
      x: event.clientX,
      y: event.clientY,
    };
    showContextMenu.value = true;
  };

  // 关闭右键菜单
  const closeContextMenu = () => {
    showContextMenu.value = false;
    contextMenuSession.value = null;
  };

  useDialogEscape(showContextMenu, closeContextMenu);

  // 从右键菜单重命名
  const handleRenameFromMenu = () => {
    if (contextMenuSession.value) {
      startRename(contextMenuSession.value);
    }
  };

  // 从右键菜单删除
  const handleDeleteFromMenu = async () => {
    if (contextMenuSession.value) {
      await deleteSession(contextMenuSession.value.id);
      closeContextMenu();
    }
  };

  // 监听工作区变化
  watch(
    () => props.workspace?.id,
    () => {
      loadSessions();
    },
  );

  // 事件监听器
  let sessionSavedUnlisten = null;
  let titleUpdatedUnlisten = null;

  onMounted(async () => {
    await loadSessions();

    // 监听会话保存事件，刷新列表并选中新会话
    sessionSavedUnlisten = await listen("chat-session-saved", async (event) => {
      await loadSessions();
      // 设置活动会话ID
      activeSessionId.value = event.payload;
      emit("session-created", event.payload);
    });

    // 监听会话标题自动更新事件，刷新列表显示
    titleUpdatedUnlisten = await listen(
      "chat-session-title-updated",
      async () => {
        await loadSessions();
      },
    );

    // 监听全局点击关闭右键菜单
    document.addEventListener("click", closeContextMenu);
  });

  onUnmounted(() => {
    if (sessionSavedUnlisten) {
      sessionSavedUnlisten();
    }
    if (titleUpdatedUnlisten) {
      titleUpdatedUnlisten();
    }
    document.removeEventListener("click", closeContextMenu);
  });

  return {
    t,
    sessions,
    activeSessionId,
    loading,
    renamingSessionId,
    renamingTitle,
    showContextMenu,
    contextMenuPosition,
    contextMenuSession,
    selectSession,
    createNewSession,
    loadSessions,
    startRename,
    cancelRename,
    confirmRename,
    handleContextMenu,
    closeContextMenu,
    handleRenameFromMenu,
    handleDeleteFromMenu,
  };
}
