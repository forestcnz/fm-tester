<script setup>
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { useTabsBarSetup } from "./index.js";
import { useDialogEscape } from "../../composables/useDialogStack.js";
import Icon from "../Icon/index.vue";
import ContextMenu from "../Sidebar/ContextMenu/index.vue";

const props = defineProps({
  tabs: {
    type: Array,
    default: () => [],
  },
  activeTab: {
    type: Number,
    default: 0,
  },
  workspace: {
    type: Object,
    default: null,
  },
});

const emit = defineEmits([
  "update:activeTab",
  "closeTab",
  "closeAllTabs",
  "closeOtherTabs",
  "selectCollection",
  "selectApi",
]);

const { t } = useI18n();
const { selectTab, closeTab, closeAllTabs, closeOtherTabs } = useTabsBarSetup(
  props,
  emit,
);

// 右键菜单状态
const contextMenuVisible = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const contextMenuTargetIndex = ref(-1);
const contextMenuRef = ref(null);

// 使用 computed 确保语言切换时更新
const contextMenuItems = computed(() => [
  { label: t("tabsBar.close"), action: "close" },
  { label: t("tabsBar.closeOthers"), action: "closeOther" },
  { divider: true },
  { label: t("tabsBar.closeAll"), action: "closeAll" },
]);

const handleContextMenu = (index, event) => {
  event.preventDefault();
  event.stopPropagation();
  contextMenuTargetIndex.value = index;
  contextMenuX.value = event.clientX;
  contextMenuY.value = event.clientY;
  contextMenuVisible.value = true;
};

const handleContextMenuAction = (action) => {
  if (action === "close") {
    closeTab(contextMenuTargetIndex.value);
  } else if (action === "closeOther") {
    closeOtherTabs(contextMenuTargetIndex.value);
  } else if (action === "closeAll") {
    closeAllTabs();
  }
};

const closeContextMenu = () => {
  contextMenuVisible.value = false;
};

useDialogEscape(contextMenuVisible, closeContextMenu);

// 全局点击监听，关闭右键菜单
const handleGlobalClick = (event) => {
  if (contextMenuVisible.value && contextMenuRef.value) {
    const menuEl = contextMenuRef.value.$el || contextMenuRef.value;
    if (menuEl && !menuEl.contains(event.target)) {
      closeContextMenu();
    }
  }
};

onMounted(() => {
  document.addEventListener("click", handleGlobalClick);
});

onUnmounted(() => {
  document.removeEventListener("click", handleGlobalClick);
});
</script>

<template>
  <div class="tabs-wrapper" @click="closeContextMenu">
    <!-- 标签列表 -->
    <div class="tabs-bar">
      <div class="tabs-container">
        <div
          v-for="(tab, index) in tabs"
          :key="tab.id"
          class="tab"
          :class="{ active: activeTab === index }"
          :title="tab.fullName || tab.name"
          @click="selectTab(index)"
          @contextmenu="handleContextMenu(index, $event)"
        >
          <!-- API 标签显示方法 -->
          <span
            v-if="tab.tabType !== 'collection'"
            class="tab-method"
            :class="tab.method?.toLowerCase()"
          >
            <template v-if="tab.method === 'WebSocket'"> WS </template>
            <template v-else>
              {{ tab.method }}
            </template>
          </span>
          <!-- 集合 tab 显示文件夹图标 -->
          <span v-if="tab.tabType === 'collection'" class="tab-icon">
            <Icon name="folder" :size="14" />
          </span>
          <span class="tab-name">{{ tab.name }}</span>
          <span class="tab-close" @click.stop="closeTab(index, $event)">×</span>
        </div>
      </div>
    </div>

    <!-- 右键菜单 -->
    <ContextMenu
      ref="contextMenuRef"
      :visible="contextMenuVisible"
      :x="contextMenuX"
      :y="contextMenuY"
      :items="contextMenuItems"
      @action="handleContextMenuAction"
      @close="closeContextMenu"
    />
  </div>
</template>

<style scoped src="./style.css"></style>
