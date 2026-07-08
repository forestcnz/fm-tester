<script setup>
import { useI18n } from "vue-i18n";
import { ref, computed, nextTick, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { showToast } from "../../composables/useToast.js";
import "./style.css";

const { t } = useI18n();

const props = defineProps({
  workspaceId: String,
  wsConfig: Object, // 当前选中的 WebSocket 配置
});

// WebSocket 配置
const wsUrl = ref("");
const wsConfigName = ref("");
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

// 从 URL 解析查询参数
const parseUrlParams = (url) => {
  if (!url) return [];
  try {
    const urlObj = new URL(url);
    const params = [];
    urlObj.searchParams.forEach((value, key) => {
      params.push({ key, value, enabled: true });
    });
    return params;
  } catch {
    const queryIndex = url.indexOf("?");
    if (queryIndex < 0) return [];
    const queryStr = url.slice(queryIndex + 1);
    if (!queryStr) return [];
    const params = [];
    queryStr.split("&").forEach((pair) => {
      const [key, value = ""] = pair.split("=");
      if (key) {
        params.push({
          key: decodeURIComponent(key),
          value: decodeURIComponent(value),
          enabled: true,
        });
      }
    });
    return params;
  }
};

// 从 params 构建带参数的 URL
const buildUrlWithParams = (baseUrl, params) => {
  if (!baseUrl) return "";
  const queryIndex = baseUrl.indexOf("?");
  const cleanUrl = queryIndex < 0 ? baseUrl : baseUrl.slice(0, queryIndex);
  const enabledParams = params.filter((p) => p.enabled && p.key);
  if (enabledParams.length === 0) return cleanUrl;
  const queryStr = enabledParams
    .map(
      (p) =>
        `${encodeURIComponent(p.key)}=${encodeURIComponent(p.value || "")}`,
    )
    .join("&");
  return `${cleanUrl}?${queryStr}`;
};

// URL 与 params 双向同步标志（防止循环更新）
let isUpdatingFromUrl = false;
let isUpdatingFromParams = false;

// URL 输入变化 → 同步到 params
const onUrlInput = () => {
  if (isUpdatingFromParams) return;
  isUpdatingFromUrl = true;
  wsParams.value = parseUrlParams(wsUrl.value);
  nextTick(() => {
    isUpdatingFromUrl = false;
  });
};

// 监听 wsConfig 变化，更新表单
watch(
  () => props.wsConfig,
  (config) => {
    isUpdatingFromUrl = true;
    if (config) {
      wsUrl.value = config.url || "";
      wsConfigName.value = config.name || "";
      wsHeaders.value = (config.headers || []).map((h) => ({
        key: h.key || "",
        value: h.value || "",
        enabled: h.enabled !== false,
      }));
      const savedParams = (config.params || []).map((p) => ({
        key: p.key || "",
        value: p.value || "",
        enabled: p.enabled !== false,
      }));
      // 已保存的参数优先；为空时从 URL 的 query 自动解析
      wsParams.value =
        savedParams.length > 0 ? savedParams : parseUrlParams(config.url || "");
    } else {
      wsUrl.value = "";
      wsConfigName.value = "";
      wsHeaders.value = [];
      wsParams.value = [];
    }
    // 重置连接状态与消息，避免上一个接口的错误信息残留
    wsState.value = { status: "disconnected", connectedAt: null, error: null };
    wsMessages.value = [];
    nextTick(() => {
      isUpdatingFromUrl = false;
    });
  },
  { immediate: true },
);

// params 变化 → 同步回 URL
watch(
  wsParams,
  () => {
    if (isUpdatingFromUrl) return;
    isUpdatingFromParams = true;
    wsUrl.value = buildUrlWithParams(wsUrl.value, wsParams.value);
    nextTick(() => {
      isUpdatingFromParams = false;
    });
  },
  { deep: true },
);

// 保存当前配置
const saveCurrentConfig = async () => {
  if (!props.workspaceId || !props.wsConfig?.id) return;
  try {
    await invoke("save_ws_config", {
      workspaceId: props.workspaceId,
      id: props.wsConfig.id,
      name: wsConfigName.value || "WebSocket",
      url: wsUrl.value,
      headers: wsHeaders.value.filter((h) => h.enabled && h.key.trim()),
      params: wsParams.value.filter((p) => p.enabled && p.key.trim()),
    });
    showToast(t("toast.apiSaved"), "success");
  } catch (e) {
    console.error("保存配置失败:", e);
    showToast(t("toast.wsSaveFailed"), "error");
  }
};

// 设置事件监听
const setupListeners = async () => {
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
  if (!wsUrl.value.trim() || !props.workspaceId) return;

  const rawUrl = wsUrl.value.trim();
  if (!rawUrl.startsWith("ws://") && !rawUrl.startsWith("wss://")) return;

  // 去掉 query 部分，query 由后端按 params 拼接，避免与 URL 中已有的参数重复
  const queryIdx = rawUrl.indexOf("?");
  const cleanUrl = queryIdx < 0 ? rawUrl : rawUrl.slice(0, queryIdx);

  wsMessages.value = [];
  wsState.value = { status: "connecting", connectedAt: null, error: null };

  await setupListeners();

  try {
    await invoke("connect_websocket", {
      url: cleanUrl,
      headers: wsHeaders.value.filter((h) => h.enabled && h.key.trim()),
      params: wsParams.value.filter((p) => p.enabled && p.key.trim()),
      workspaceId: props.workspaceId,
      wsId: null,
    });
  } catch (error) {
    wsState.value = { status: "error", connectedAt: null, error };
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
  if (!wsMessageInput.value.trim() || wsState.value.status !== "connected")
    return;
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
const formatTime = (timestamp) => new Date(timestamp).toLocaleTimeString();

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

// 组件挂载
onMounted(async () => {
  await setupListeners();
});

// 组件卸载
onUnmounted(async () => {
  await disconnect();
  await removeListeners();
});
</script>

<template>
  <div class="ws-detail-panel">
    <!-- 无配置时的提示 -->
    <div v-if="!props.wsConfig" class="ws-empty">
      <div class="empty-hint">{{ t("websocket.sidebarHint") }}</div>
    </div>

    <!-- 配置详情 -->
    <template v-else>
      <!-- URL 输入区 -->
      <div class="ws-url-bar">
        <div class="ws-method-tag">WS</div>
        <input
          type="text"
          v-model="wsUrl"
          :placeholder="t('placeholder.url')"
          class="ws-url-input"
          @input="onUrlInput"
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
        <button class="ws-save-btn" @click="saveCurrentConfig">
          {{ t("common.save") }}
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

      <!-- 参数/请求头配置区 -->
      <div class="ws-config-area">
        <div class="ws-config-tabs">
          <button
            class="ws-config-tab"
            :class="{ active: wsConfigTab === 'params' }"
            @click="wsConfigTab = 'params'"
          >
            {{ t("tabs.params")
            }}<span v-if="wsParams.length" class="ws-tab-cnt">{{
              wsParams.length
            }}</span>
          </button>
          <button
            class="ws-config-tab"
            :class="{ active: wsConfigTab === 'headers' }"
            @click="wsConfigTab = 'headers'"
          >
            {{ t("tabs.headers")
            }}<span v-if="wsHeaders.length" class="ws-tab-cnt">{{
              wsHeaders.length
            }}</span>
          </button>
        </div>
        <div v-show="wsConfigTab === 'params'" class="ws-config-content">
          <div class="ws-config-toolbar">
            <button class="ws-add-btn" @click="addParam">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
              >
                <path d="M12 5v14M5 12h14" />
              </svg>
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
        <div v-show="wsConfigTab === 'headers'" class="ws-config-content">
          <div class="ws-config-toolbar">
            <button class="ws-add-btn" @click="addHeader">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
              >
                <path d="M12 5v14M5 12h14" />
              </svg>
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
              <span class="ws-message-direction">{{
                msg.direction === "sent"
                  ? t("websocket.sent")
                  : t("websocket.received")
              }}</span>
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
    </template>
  </div>
</template>

<style scoped src="./style.css"></style>
