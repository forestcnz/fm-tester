<script setup>
import { useChatHistorySetup } from "./index.js";
import "./style.css";

const props = defineProps({
  workspace: Object,
});

const emit = defineEmits(["select-session", "new-session", "session-created"]);

const {
  t,
  sessions,
  activeSessionId,
  loading,
  renamingSessionId,
  renamingTitle,
  showContextMenu,
  contextMenuPosition,
  selectSession,
  createNewSession,
  cancelRename,
  confirmRename,
  handleContextMenu,
  handleRenameFromMenu,
  handleDeleteFromMenu,
} = useChatHistorySetup(props, emit);
</script>

<template>
  <div class="chat-history-panel paper-theme">
    <!-- 纸质风格头部 -->
    <div class="panel-header">
      <div class="header-title-group">
        <div class="panel-title">{{ t("nav.chat") }}</div>
        <div class="panel-subtitle">Conversations</div>
      </div>
      <button
        class="paper-new-btn"
        @click="createNewSession"
        :title="t('buttons.new')"
      >
        <span class="btn-icon">+</span>
      </button>
    </div>

    <!-- 线条装饰 -->
    <div class="header-line"></div>

    <!-- 会话列表 -->
    <div class="session-list" v-if="!loading && sessions.length > 0">
      <div
        v-for="session in sessions"
        :key="session.id"
        class="session-item"
        :class="{ active: activeSessionId === session.id }"
        @click="selectSession(session)"
        @contextmenu.prevent="handleContextMenu(session, $event)"
      >
        <div class="session-card">
          <!-- 纸质卡片内容 -->
          <div class="session-info">
            <!-- 重命名模式 -->
            <template v-if="renamingSessionId === session.id">
              <div class="rename-wrapper">
                <input
                  v-model="renamingTitle"
                  class="paper-rename-input"
                  :placeholder="
                    t('chat.sessionTitle', { date: session.created_at })
                  "
                  @keyup.enter="confirmRename(session)"
                  @keyup.escape="cancelRename"
                  @blur="confirmRename(session)"
                  ref="renameInput"
                />
              </div>
            </template>
            <!-- 正常显示模式 -->
            <template v-else>
              <div class="session-title">
                <span class="title-dot"></span>
                {{
                  session.title ||
                  t("chat.sessionTitle", { date: session.created_at })
                }}
              </div>
              <div class="session-meta">
                <span class="session-date">{{ session.created_at }}</span>
                <span
                  class="session-badge"
                  v-if="activeSessionId === session.id"
                >
                  Active
                </span>
              </div>
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div class="empty-sessions" v-else-if="!loading">
      <div class="empty-paper-card">
        <div class="empty-icon">📝</div>
        <div class="empty-text">{{ t("chat.noSessions") }}</div>
        <button class="paper-create-btn" @click="createNewSession">
          {{ t("chat.newSession") }}
        </button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div class="loading-sessions" v-else>
      <div class="paper-loading-indicator">
        <span class="loading-dot"></span>
        <span class="loading-dot"></span>
        <span class="loading-dot"></span>
      </div>
    </div>
  </div>

  <!-- 右键菜单（纸质风格） -->
  <Teleport to="body">
    <div
      v-if="showContextMenu"
      class="paper-context-menu"
      :style="{
        top: contextMenuPosition.y + 'px',
        left: contextMenuPosition.x + 'px',
      }"
      @click.stop
    >
      <div class="context-menu-item" @click.stop="handleRenameFromMenu">
        <span class="menu-icon">✎</span>
        <span>{{ t("common.rename") }}</span>
      </div>
      <div class="context-divider"></div>
      <div class="context-menu-item danger" @click.stop="handleDeleteFromMenu">
        <span class="menu-icon">✕</span>
        <span>{{ t("common.delete") }}</span>
      </div>
    </div>
  </Teleport>
</template>
