<script setup>
import { useChatSetup } from "./index.js";
import "./style.css";

const props = defineProps({
  workspaceId: {
    type: String,
    default: "",
  },
  sessionId: {
    type: String,
    default: null,
  },
});

const {
  t,
  messages,
  inputMessage,
  loading,
  sending,
  streamingDone,
  hasWorkspace,
  fmContext,
  sendMessage,
  stopSending,
  renderMarkdown,
  formatTime,
} = useChatSetup(props);
</script>

<template>
  <div class="chat-panel paper-theme">
    <!-- 消息区域 -->
    <div class="chat-messages">
      <div v-if="!hasWorkspace" class="no-workspace-hint">
        <div class="empty-paper-card">
          <div class="empty-icon">📄</div>
          <div class="empty-text">{{ t("chat.noWorkspaceHint") }}</div>
        </div>
      </div>

      <div v-else-if="messages.length === 0" class="empty-chat">
        <div class="empty-paper-card">
          <div class="empty-icon">💬</div>
          <div class="empty-text">{{ t("chat.emptyHint") }}</div>
          <div class="empty-subtext">Type below to start conversation</div>
        </div>
      </div>

      <template v-else>
        <div
          v-for="(msg, index) in messages"
          :key="index"
          class="message"
          :class="msg.role"
        >
          <div class="msg-ava">{{ msg.role === "user" ? "U" : "AI" }}</div>
          <!-- 消息气泡 -->
          <div class="paper-card">
            <!-- 消息标签 -->
            <div class="paper-label">
              <span class="label-dot"></span>
              <span class="label-text">
                {{ msg.role === "user" ? t("chat.you") : t("chat.ai") }}
              </span>
              <span v-if="msg.timestamp" class="label-time">
                {{ formatTime(msg.timestamp) }}
              </span>
            </div>

            <!-- 思考过程（进行中显示标题+内容实时显示） -->
            <div
              v-if="
                msg.role === 'assistant' &&
                msg.reasoning &&
                !streamingDone[index]
              "
              class="reasoning-paper"
            >
              <div class="reasoning-header">
                <span class="reasoning-title">thinking</span>
                <span class="reasoning-dots">
                  <span class="dot"></span>
                  <span class="dot"></span>
                  <span class="dot"></span>
                </span>
              </div>
              <div class="reasoning-content">
                {{ msg.reasoning }}
              </div>
            </div>

            <!-- 思考内容（完成后显示，不显示标题） -->
            <div
              v-if="
                msg.role === 'assistant' &&
                msg.reasoning &&
                streamingDone[index]
              "
              class="reasoning-content-final"
            >
              {{ msg.reasoning }}
            </div>

            <!-- 工具调用状态（@fm Agent 模式） -->
            <div
              v-if="msg.role === 'assistant' && msg.tools && msg.tools.length"
              class="tool-calls"
            >
              <div class="tool-calls-title">🔧 {{ t("chat.toolCalling") }}</div>
              <div
                v-for="(tool, ti) in msg.tools"
                :key="ti"
                class="tool-call-item"
              >
                {{ tool.name }}
              </div>
            </div>

            <!-- 消息内容 -->
            <div
              class="message-content"
              :class="{
                'loading-content':
                  msg.role === 'assistant' && !msg.content && sending,
              }"
            >
              <!-- 加载动画 -->
              <template
                v-if="msg.role === 'assistant' && !msg.content && sending"
              >
                <div class="paper-loading">
                  <span class="loading-line"></span>
                  <span class="loading-line"></span>
                  <span class="loading-line"></span>
                </div>
              </template>

              <!-- 流式输出 -->
              <template
                v-else-if="msg.role === 'assistant' && !streamingDone[index]"
              >
                <div class="streaming-text">{{ msg.content }}</div>
              </template>

              <!-- Markdown渲染 -->
              <template v-else-if="msg.role === 'assistant'">
                <!-- eslint-disable-next-line vue/no-v-html -->
                <div class="markdown-content" v-html="renderMarkdown(msg.content)"></div>
              </template>

              <!-- 用户消息 -->
              <template v-else>
                <div class="user-text">{{ msg.content }}</div>
              </template>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- 纸质风格输入区域 -->
    <div class="chat-input-area">
      <div class="input-paper">
        <div v-if="fmContext" class="fm-badge">
          ✦ {{ t("chat.fmContextOn") }}
        </div>
        <textarea
          v-model="inputMessage"
          :disabled="sending || loading || !hasWorkspace"
          :placeholder="
            hasWorkspace
              ? t('chat.inputPlaceholder')
              : t('chat.noWorkspaceHint')
          "
          class="paper-input"
          rows="3"
          @keydown.enter="sendMessage"
        ></textarea>

        <div class="input-footer">
          <span class="input-hint">{{ t("chat.sendHint") }}</span>
          <div class="input-actions">
            <button v-if="sending" class="paper-stop-btn" @click="stopSending">
              <span class="btn-icon">■</span>
              <span class="btn-text">{{ t("chat.stop") }}</span>
            </button>
            <button
              v-else
              class="paper-send-btn"
              :disabled="!inputMessage.trim() || loading || !hasWorkspace"
              @click="sendMessage"
            >
              <span class="btn-icon">→</span>
              <span class="btn-text">{{ t("buttons.send") }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
