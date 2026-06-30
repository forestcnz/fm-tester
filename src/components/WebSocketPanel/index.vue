<script setup>
import { ref, computed, nextTick, onMounted, onUnmounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import VariableHighlight from "../VariableHighlight/index.vue";
import "./style.css";

const { t } = useI18n();

const props = defineProps({
  workspaceId: {
    type: String,
    default: "",
  },
  variables: {
    type: Array,
    default: () => [],
  },
  initialUrl: {
    type: String,
    default: "",
  },
  initialHeaders: {
    type: Array,
    default: () => [],
  },
  initialParams: {
    type: Array,
    default: () => [],
  },
});

const emit = defineEmits(["close", "update:config"]);

// WebSocket 配置
const wsUrl = ref(props.initialUrl || "");
const wsHeaders = ref([]);
const wsParams = ref([]);
const wsConfigTab = ref("params");

// WebSocket 状态
const wsState = ref({
  status: "disconnected",
  connectedAt: null,
  error: null,
});

// 消息
const wsMessages = ref([]);

// 发送消息
const wsMessageInput = ref("");
const wsMessageType = ref("text");

// 事件监听器
let unlistenWsState = null;
let unlistenWsMessage = null;

// 状态样式
const wsStatusClass = computed(() => {
  switch (wsState.value.status) {
    case "connecting":
      return "ws-connecting";
    case "connected":
      return "ws-connected";
    case "disconnected":
      return "ws-disconnected";
    case "error":
      return "ws-error";
    default:
      return "";
  }
});

// 状态文本
const wsStatusText = computed(() => {
  const status = wsState.value.status;
  switch (status) {
    case "connecting":
      return t("websocket.connecting");
    case "connected":
      return t("websocket.connected");
    case "disconnected":
      return t("websocket.disconnected");
    case "error":
      return t("websocket.error");
    default:
      return status;
  }
});

// 连接时间格式化
const wsConnectedTime = computed(() => {
  if (!wsState.value.connectedAt) return "";
  const date = new Date(wsState.value.connectedAt);
  return date.toLocaleString();
});

// 设置事件监听
const setupListeners = async () => {
  // 先移除旧的监听器
  await removeListeners();

  unlistenWsState = await listen("ws-state", (event) => {
    const state = event.payload;
    wsState.value = {
      status: state.status,
      connectedAt: state.connected_at,
      error: state.error,
    };
  });

  unlistenWsMessage = await listen("ws-message", (event) => {
    const message = event.payload;
    wsMessages.value.push({
      id: message.id,
      direction: message.direction,
      content: message.content,
      type: message.type,
      timestamp: message.timestamp,
    });

    // 滚动到底部
    scrollToBottom();
  });
};

// 移除事件监听
const removeListeners = async () => {
  if (unlistenWsState) {
    await unlistenWsState();
    unlistenWsState = null;
  }
  if (unlistenWsMessage) {
    await unlistenWsMessage();
    unlistenWsMessage = null;
  }
};

// 连接 WebSocket
const connect = async () => {
  if (!wsUrl.value.trim()) {
    return;
  }

  let url = wsUrl.value.trim();
  if (!url.startsWith("ws://") && !url.startsWith("wss://")) {
    return;
  }

  if (!props.workspaceId) {
    return;
  }

  // 初始化
  wsMessages.value = [];
  wsState.value = { status: "connecting", connectedAt: null, error: null };

  await setupListeners();

  try {
    await invoke("connect_websocket", {
      url: url,
      headers: wsHeaders.value.filter((h) => h.enabled && h.key.trim()),
      params: wsParams.value.filter((p) => p.enabled && p.key.trim()),
      workspaceId: props.workspaceId,
      wsId: null,
    });
  } catch (error) {
    wsState.value = {
      status: "error",
      connectedAt: null,
      error: error,
    };
  }
};

// 断开连接
const disconnect = async () => {
  try {
    await invoke("disconnect_websocket");
  } catch (error) {
    console.error("断开失败:", error);
  }
};

// 发送消息
const sendMessage = async () => {
  if (!wsMessageInput.value.trim()) {
    return;
  }

  if (wsState.value.status !== "connected") {
    return;
  }

  try {
    await invoke("send_ws_message", {
      content: wsMessageInput.value,
      messageType: wsMessageType.value,
    });
    wsMessageInput.value = "";
  } catch (error) {
    console.error("发送失败:", error);
  }
};

// 清空消息
const clearMessages = () => {
  wsMessages.value = [];
};

// 添加 Header
const addHeader = () => {
  wsHeaders.value.push({ key: "", value: "", enabled: true });
};

// 删除 Header
const removeHeader = (index) => {
  wsHeaders.value.splice(index, 1);
};

// 添加 Param
const addParam = () => {
  wsParams.value.push({ key: "", value: "", enabled: true });
};

// 删除 Param
const removeParam = (index) => {
  wsParams.value.splice(index, 1);
};

// 格式化消息时间
const formatTime = (timestamp) => {
  const date = new Date(timestamp);
  return date.toLocaleTimeString();
};

// 消息容器
const messagesContainer = ref(null);

// 滚动到底部
const scrollToBottom = () => {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
    }
  });
};

// 组件挂载时设置监听器
onMounted(async () => {
  await setupListeners();
});

// 组件卸载时清理
onUnmounted(async () => {
  await disconnect();
  await removeListeners();
});

// 监听 initialUrl 变化
watch(
  () => props.initialUrl,
  (url) => {
    if (url !== undefined && url !== wsUrl.value) {
      wsUrl.value = url;
    }
  },
);

// 监听 initialHeaders 变化
watch(
  () => props.initialHeaders,
  (headers) => {
    if (headers) {
      wsHeaders.value = headers.map((h) => ({
        key: h.key || "",
        value: h.value || "",
        enabled: h.enabled !== false,
      }));
    } else {
      wsHeaders.value = [];
    }
  },
  { immediate: true },
);

// 监听 initialParams 变化
watch(
  () => props.initialParams,
  (params) => {
    if (params) {
      wsParams.value = params.map((p) => ({
        key: p.key || "",
        value: p.value || "",
        enabled: p.enabled !== false,
      }));
    } else {
      wsParams.value = [];
    }
  },
  { immediate: true },
);

// 手动保存配置
const saveConfig = () => {
  emit("update:config", {
    url: wsUrl.value,
    headers: wsHeaders.value,
    params: wsParams.value,
  });
};
</script>

<template>
  <div class="websocket-panel">
    <!-- URL 输入区 -->
    <div class="ws-url-bar">
      <div class="ws-method-tag">WS</div>
      <VariableHighlight
        mode="input"
        :text="wsUrl"
        :variables="variables"
        @input="(val) => (wsUrl = val)"
        class="url-mode"
        :placeholder="t('placeholder.url')"
      />
      <button
        class="ws-connect-btn"
        :class="{ connected: wsState.status === 'connected' }"
        @click="wsState.status === 'connected' ? disconnect() : connect()"
      >
        {{
          wsState.status === "connected"
            ? t("buttons.disconnect")
            : t("buttons.connect")
        }}
      </button>
      <button class="ws-save-btn" @click="saveConfig">
        {{ t("buttons.save") }}
      </button>
    </div>

    <!-- 状态栏 -->
    <div class="ws-status-bar">
      <div class="ws-status-info">
        <span class="ws-status-indicator" :class="wsStatusClass"></span>
        <span class="ws-status-text">{{ wsStatusText }}</span>
        <span v-if="wsState.error" class="ws-error-detail">
          {{
            typeof wsState.error === "string"
              ? wsState.error
              : wsState.error.message || JSON.stringify(wsState.error)
          }}
        </span>
        <span v-if="wsConnectedTime" class="ws-connected-time">
          {{ t("websocket.connectedAt") }}: {{ wsConnectedTime }}
        </span>
      </div>
    </div>

    <!-- 配置区 -->
    <div class="ws-config-area">
      <!-- 配置标签页 -->
      <div class="ws-config-tabs">
        <div
          class="ws-config-tab"
          :class="{ active: wsConfigTab === 'params' }"
          @click="wsConfigTab = 'params'"
        >
          {{ t("tabs.params") }}
        </div>
        <div
          class="ws-config-tab"
          :class="{ active: wsConfigTab === 'headers' }"
          @click="wsConfigTab = 'headers'"
        >
          {{ t("tabs.headers") }}
        </div>
      </div>

      <!-- Params -->
      <div v-show="wsConfigTab === 'params'" class="ws-config-content">
        <div class="ws-config-toolbar">
          <button class="ws-add-btn" @click="addParam">
            {{ t("buttons.addParam") }}
          </button>
        </div>
        <div class="ws-config-list">
          <div class="ws-config-table-header">
            <span class="ws-col-check"></span>
            <span class="ws-col-key">{{ t("table.paramName") }}</span>
            <span class="ws-col-value">{{ t("table.paramValue") }}</span>
            <span class="ws-col-action"></span>
          </div>
          <div v-if="wsParams.length === 0" class="ws-config-empty">
            {{ t("empty.noParams") }}
          </div>
          <div
            v-for="(param, index) in wsParams"
            :key="index"
            class="ws-config-row"
          >
            <span class="ws-col-check">
              <input type="checkbox" v-model="param.enabled" />
            </span>
            <span class="ws-col-key">
              <input
                type="text"
                v-model="param.key"
                :placeholder="t('placeholder.paramName')"
              />
            </span>
            <span class="ws-col-value">
              <input
                type="text"
                v-model="param.value"
                :placeholder="t('placeholder.paramValue')"
              />
            </span>
            <span class="ws-col-action">
              <button class="ws-remove-btn" @click="removeParam(index)">
                ×
              </button>
            </span>
          </div>
        </div>
      </div>

      <!-- Headers -->
      <div v-show="wsConfigTab === 'headers'" class="ws-config-content">
        <div class="ws-config-toolbar">
          <button class="ws-add-btn" @click="addHeader">
            {{ t("buttons.addHeader") }}
          </button>
        </div>
        <div class="ws-config-list">
          <div class="ws-config-table-header">
            <span class="ws-col-check"></span>
            <span class="ws-col-key">{{ t("table.headerName") }}</span>
            <span class="ws-col-value">{{ t("table.headerValue") }}</span>
            <span class="ws-col-action"></span>
          </div>
          <div v-if="wsHeaders.length === 0" class="ws-config-empty">
            {{ t("empty.noHeaders") }}
          </div>
          <div
            v-for="(header, index) in wsHeaders"
            :key="index"
            class="ws-config-row"
          >
            <span class="ws-col-check">
              <input type="checkbox" v-model="header.enabled" />
            </span>
            <span class="ws-col-key">
              <input
                type="text"
                v-model="header.key"
                :placeholder="t('placeholder.headerName')"
              />
            </span>
            <span class="ws-col-value">
              <input
                type="text"
                v-model="header.value"
                :placeholder="t('placeholder.headerValue')"
              />
            </span>
            <span class="ws-col-action">
              <button class="ws-remove-btn" @click="removeHeader(index)">
                ×
              </button>
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- 主内容区 -->
    <div class="ws-main-area">
      <!-- 消息区 -->
      <div class="ws-messages-area">
        <div class="ws-messages-header">
          <span>{{ t("websocket.messages") }} ({{ wsMessages.length }})</span>
          <button class="ws-clear-btn" @click="clearMessages">
            {{ t("buttons.clear") }}
          </button>
        </div>
        <div ref="messagesContainer" class="ws-messages-list">
          <div v-if="wsMessages.length === 0" class="ws-messages-empty">
            {{ t("empty.noWsMessages") }}
          </div>
          <div
            v-for="msg in wsMessages"
            :key="msg.id"
            class="ws-message-item"
            :class="{
              sent: msg.direction === 'sent',
              received: msg.direction === 'received',
            }"
          >
            <div class="ws-message-meta">
              <span class="ws-message-time">{{
                formatTime(msg.timestamp)
              }}</span>
              <span class="ws-message-direction">
                {{
                  msg.direction === "sent"
                    ? t("websocket.sent")
                    : t("websocket.received")
                }}
              </span>
              <span class="ws-message-type">{{ msg.type }}</span>
            </div>
            <div class="ws-message-content">{{ msg.content }}</div>
          </div>
        </div>

        <!-- 发送区 -->
        <div class="ws-send-area">
          <select v-model="wsMessageType" class="ws-type-select">
            <option value="text">Text</option>
            <option value="binary">Binary (Base64)</option>
          </select>
          <input
            type="text"
            v-model="wsMessageInput"
            :placeholder="t('placeholder.wsMessage')"
            class="ws-send-input"
            @keyup.enter="sendMessage"
          />
          <button
            class="ws-send-btn"
            @click="sendMessage"
            :disabled="wsState.status !== 'connected'"
          >
            {{ t("buttons.send") }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
