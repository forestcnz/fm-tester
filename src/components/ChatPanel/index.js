import { ref, computed, nextTick, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useI18n } from "vue-i18n";
import { renderMarkdown } from "../../utils/markdown.js";

export function useChatSetup(props) {
  const { t } = useI18n();

  const messages = ref([]);
  const inputMessage = ref("");
  const loading = ref(false);
  const sending = ref(false);
  const hasWorkspace = computed(
    () => !!props.workspaceId && props.workspaceId.trim() !== "",
  );
  const streamingDone = ref({});
  const abortSending = ref(false);
  const sessionId = ref(null);

  const loadChatHistory = async () => {
    if (!hasWorkspace.value) return;

    if (!sessionId.value) {
      messages.value = [];
      streamingDone.value = {};
      reasoningExpanded.value = {};
      return;
    }

    try {
      const history = await invoke("get_chat_history", {
        workspaceId: props.workspaceId,
        sessionId: sessionId.value,
      });

      if (history && history.length > 0) {
        messages.value = history.map((m) => ({
          role: m.role,
          content: m.content,
          reasoning: m.reasoning || "",
          timestamp: m.timestamp || null,
        }));
        history.forEach((_, index) => {
          streamingDone.value[index] = true;
          reasoningExpanded.value[index] = false;
        });
      } else {
        messages.value = [];
        streamingDone.value = {};
        reasoningExpanded.value = {};
      }
    } catch (e) {
      console.error("Failed to load chat history:", e);
      messages.value = [];
      streamingDone.value = {};
      reasoningExpanded.value = {};
    }
  };

  const saveChatHistory = async () => {
    if (!hasWorkspace.value) return;

    try {
      const chatMessages = messages.value.map((m) => ({
        role: m.role,
        content: m.content,
        reasoning: m.reasoning || null,
        timestamp: m.timestamp || new Date().toISOString(),
      }));

      const id = await invoke("save_chat_history", {
        workspaceId: props.workspaceId,
        sessionId: sessionId.value,
        messages: chatMessages,
      });

      sessionId.value = id;
    } catch (e) {
      console.error("Failed to save chat history:", e);
    }
  };

  const scrollToBottom = () => {
    nextTick(() => {
      const container = document.querySelector(".chat-messages");
      if (container) {
        container.scrollTop = container.scrollHeight;
      }
    });
  };

  const sendMessage = async () => {
    if (!inputMessage.value.trim() || sending.value || !hasWorkspace.value)
      return;

    abortSending.value = false;

    // @fm 触发工作区上下文（Function Calling Agent 模式），发送时剥离该标记
    const rawInput = inputMessage.value;
    const hasFm = /@fm\b/i.test(rawInput);
    const userMessage = (
      hasFm ? rawInput.replace(/@fm\s*/i, "") : rawInput
    ).trim();
    if (!userMessage) return;
    messages.value.push({
      role: "user",
      content: userMessage,
      timestamp: new Date().toISOString(),
    });
    inputMessage.value = "";
    scrollToBottom();

    sending.value = true;

    messages.value.push({
      role: "assistant",
      content: "",
      reasoning: "",
      tools: [],
      timestamp: null,
    });

    const aiIndex = messages.value.length - 1;
    streamingDone.value[aiIndex] = false;

    scrollToBottom();

    try {
      const chatMessages = messages.value.slice(0, -1).map((m) => ({
        role: m.role,
        content: m.content,
      }));

      const result = hasFm
        ? await invoke("chat_ai_agent", {
            workspaceId: props.workspaceId,
            messages: chatMessages,
          })
        : await invoke("chat_ai", {
            messages: chatMessages,
          });

      if (abortSending.value) return;

      streamingDone.value[aiIndex] = true;
      reasoningExpanded.value[aiIndex] = false;

      messages.value[aiIndex].content = result;
      messages.value[aiIndex].timestamp = new Date().toISOString();

      await saveChatHistory();
    } catch (e) {
      if (abortSending.value) return;
      console.error("Chat error:", e);
      messages.value[messages.value.length - 1].content =
        `${t("chat.error")}: ${e}`;
      messages.value[messages.value.length - 1].timestamp =
        new Date().toISOString();
    } finally {
      sending.value = false;
      scrollToBottom();
    }
  };

  const stopSending = () => {
    abortSending.value = true;
    sending.value = false;

    if (messages.value.length > 0) {
      const lastIndex = messages.value.length - 1;
      if (messages.value[lastIndex].role === "assistant") {
        streamingDone.value[lastIndex] = true;
        if (messages.value[lastIndex].reasoning) {
          reasoningExpanded.value[lastIndex] = false;
        }
        messages.value[lastIndex].timestamp = new Date().toISOString();
      }
    }

    saveChatHistory();
  };

  const clearMessages = async () => {
    if (!hasWorkspace.value) return;

    messages.value = [];
    streamingDone.value = {};
    reasoningExpanded.value = {};

    try {
      await invoke("clear_chat_history", {
        workspaceId: props.workspaceId,
        sessionId: sessionId.value,
      });
      sessionId.value = null;
    } catch (e) {
      console.error("Failed to clear chat history:", e);
    }
  };

  const formatTime = (timestamp) => {
    if (!timestamp) return "";
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now - date;

    if (diff < 60000) return t("chat.justNow");
    if (diff < 3600000) {
      const minutes = Math.floor(diff / 60000);
      return t("chat.minutesAgo", { count: minutes });
    }
    if (diff < 86400000) {
      const hours = Math.floor(diff / 3600000);
      return t("chat.hoursAgo", { count: hours });
    }

    return date.toLocaleString("zh-CN", {
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  let streamUnlisten = null;
  let reasoningUnlisten = null;
  let toolUnlisten = null;
  const reasoningExpanded = ref({});

  // 输入 @fm 时启用工作区上下文（Agent 模式）
  const fmContext = computed(() => /@fm\b/i.test(inputMessage.value));

  watch(
    () => props.workspaceId,
    async () => {
      sessionId.value = props.sessionId;
      if (hasWorkspace.value) {
        await loadChatHistory();
      } else {
        messages.value = [];
        streamingDone.value = {};
        reasoningExpanded.value = {};
        sessionId.value = null;
      }
    },
  );

  watch(
    () => props.sessionId,
    async (newSessionId) => {
      sessionId.value = newSessionId;
      if (hasWorkspace.value) {
        await loadChatHistory();
      }
    },
  );

  onMounted(async () => {
    sessionId.value = props.sessionId;

    if (hasWorkspace.value) {
      await loadChatHistory();
    }

    streamUnlisten = await listen("ai-chat-stream", (event) => {
      if (abortSending.value) return;

      if (messages.value.length > 0) {
        const lastIndex = messages.value.length - 1;
        if (messages.value[lastIndex].role === "assistant") {
          messages.value[lastIndex] = {
            ...messages.value[lastIndex],
            content: messages.value[lastIndex].content + event.payload,
          };
          scrollToBottom();
        }
      }
    });

    reasoningUnlisten = await listen("ai-chat-reasoning", (event) => {
      if (abortSending.value) return;

      if (messages.value.length > 0) {
        const lastIndex = messages.value.length - 1;
        if (messages.value[lastIndex].role === "assistant") {
          messages.value[lastIndex] = {
            ...messages.value[lastIndex],
            reasoning:
              (messages.value[lastIndex].reasoning || "") + event.payload,
          };
          reasoningExpanded.value[lastIndex] = true;
          scrollToBottom();
        }
      }
    });

    toolUnlisten = await listen("ai-chat-tool", (event) => {
      if (messages.value.length > 0) {
        const lastIndex = messages.value.length - 1;
        if (messages.value[lastIndex].role === "assistant") {
          if (!messages.value[lastIndex].tools) {
            messages.value[lastIndex].tools = [];
          }
          messages.value[lastIndex].tools.push(event.payload);
          scrollToBottom();
        }
      }
    });
  });

  onUnmounted(() => {
    if (streamUnlisten) streamUnlisten();
    if (reasoningUnlisten) reasoningUnlisten();
    if (toolUnlisten) toolUnlisten();
  });

  const renderMarkdownContent = (content) => renderMarkdown(content);

  const toggleReasoning = (index) => {
    reasoningExpanded.value[index] = !reasoningExpanded.value[index];
  };

  return {
    t,
    messages,
    inputMessage,
    loading,
    sending,
    streamingDone,
    hasWorkspace,
    reasoningExpanded,
    fmContext,
    sendMessage,
    stopSending,
    clearMessages,
    renderMarkdown: renderMarkdownContent,
    toggleReasoning,
    formatTime,
  };
}
