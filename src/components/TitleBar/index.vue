<script setup>
import { ref, onMounted, computed } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTheme } from "../../composables/useTheme";

const appWindow = getCurrentWindow();
const isMaximized = ref(false);
const { isDarkTheme, setTheme } = useTheme();

const isDark = computed(() => isDarkTheme());

const toggleTheme = () => {
  setTheme(isDark.value ? "paper" : "dark");
};

const minimizeWindow = async () => {
  try {
    await appWindow.minimize();
  } catch (err) {
    console.error("最小化失败:", err);
  }
};

const maximizeWindow = async () => {
  try {
    await appWindow.toggleMaximize();
    isMaximized.value = await appWindow.isMaximized();
  } catch (err) {
    console.error("最大化失败:", err);
  }
};

const closeWindow = async () => {
  try {
    await appWindow.close();
  } catch (err) {
    console.error("关闭失败:", err);
  }
};

onMounted(async () => {
  try {
    isMaximized.value = await appWindow.isMaximized();
  } catch (err) {
    console.error("获取窗口状态失败:", err);
  }
});
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <!-- 品牌 -->
    <div class="brand" data-tauri-drag-region>
      <span class="brand-mark">
        <img src="/app-icon.png" alt="FM Tester" />
      </span>
      <span class="brand-name">FM <em>Tester</em></span>
    </div>

    <span class="tb-spacer" data-tauri-drag-region></span>

    <!-- 主题快捷切换 -->
    <button
      class="tb-btn"
      :title="isDark ? '切换到浅色' : '切换到深色'"
      @click.stop="toggleTheme"
    >
      <svg
        v-if="isDark"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="12" cy="12" r="4" />
        <path
          d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"
        />
      </svg>
      <svg
        v-else
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
      </svg>
    </button>

    <!-- 窗口控制 -->
    <div class="title-controls">
      <button
        class="control-btn minimize"
        @click.stop="minimizeWindow"
        title="最小化"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
          <rect x="2" y="5" width="8" height="2" />
        </svg>
      </button>
      <button
        class="control-btn maximize"
        @click.stop="maximizeWindow"
        title="最大化"
      >
        <svg
          v-if="!isMaximized"
          width="12"
          height="12"
          viewBox="0 0 12 12"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <rect x="2" y="2" width="8" height="8" />
        </svg>
        <svg
          v-else
          width="12"
          height="12"
          viewBox="0 0 12 12"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <rect x="3" y="1" width="8" height="8" rx="1" />
          <rect
            x="1"
            y="3"
            width="8"
            height="8"
            rx="1"
            fill="var(--bg-panel)"
          />
        </svg>
      </button>
      <button
        class="control-btn close"
        @click.stop="closeWindow"
        title="关闭"
      >
        <svg
          width="12"
          height="12"
          viewBox="0 0 12 12"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <line x1="2" y1="2" x2="10" y2="10" />
          <line x1="10" y1="2" x2="2" y2="10" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  height: 38px;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 0 0 0 14px;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--line);
  flex-shrink: 0;
  user-select: none;
}

.brand {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-left: 6px;
}

.brand-mark {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  overflow: hidden;
  box-shadow: var(--shadow-sm);
  flex-shrink: 0;
}

.brand-mark img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.brand-name {
  font-family: var(--font-serif);
  font-weight: 600;
  font-size: 16px;
  letter-spacing: -0.01em;
  color: var(--fg);
}

.brand-name em {
  font-style: italic;
  font-weight: 500;
  color: var(--primary);
}

.brand-sub {
  font-size: 10.5px;
  color: var(--fg-3);
  letter-spacing: 0.14em;
  text-transform: uppercase;
  padding-left: 10px;
  margin-left: 4px;
  border-left: 1px solid var(--line);
}

.tb-spacer {
  flex: 1;
}

.tb-btn {
  width: 30px;
  height: 26px;
  display: grid;
  place-items: center;
  border-radius: 7px;
  color: var(--fg-2);
  cursor: pointer;
  border: none;
  background: none;
  transition:
    background var(--dur-base) var(--ease),
    color var(--dur-base) var(--ease);
}

.tb-btn:hover {
  background: var(--bg-sunk);
  color: var(--fg);
}

.tb-btn svg {
  width: 16px;
  height: 16px;
}

.title-controls {
  display: flex;
  align-items: center;
  height: 100%;
}

.control-btn {
  width: 46px;
  height: 100%;
  border: none;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--fg-2);
  transition:
    background 0.15s,
    color 0.15s;
}

.control-btn:hover {
  background: var(--bg-sunk);
  color: var(--fg);
}

.control-btn.close:hover {
  background: var(--err);
  color: #fff;
}

.control-btn svg {
  display: block;
}
</style>
