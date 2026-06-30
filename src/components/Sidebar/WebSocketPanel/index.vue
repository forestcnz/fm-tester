<script setup>
import { useI18n } from "vue-i18n";
import { ref, watch, computed, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "../../../composables/useToast.js";
import { useDialogEscape } from "../../../composables/useDialogStack.js";
import Icon from "../../Icon/index.vue";
import "./style.css";

const { t } = useI18n();

const props = defineProps({
  workspace: Object,
});

const emit = defineEmits(["selectWsConfig", "createWsConfig"]);

// WebSocket 配置列表
const wsConfigs = ref([]);
const selectedConfigId = ref(null);
const searchQuery = ref("");

// Inline 编辑状态
const editingId = ref(null);
const editingName = ref("");
const inlineEditInput = ref(null);
const setInlineEditInput = (el) => {
  inlineEditInput.value = el;
};
const isSavingEdit = ref(false);

// 右键菜单状态
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  config: null,
});

// 加载 WebSocket 配置列表
const loadConfigs = async () => {
  if (!props.workspace?.id) return;
  try {
    const configs = await invoke("get_ws_configs", {
      workspaceId: props.workspace.id,
    });
    wsConfigs.value = configs.map((c) => ({
      id: c.id,
      name: c.name,
      url: c.url,
      headers: c.headers || [],
      params: c.params || [],
      createdAt: c.createdAt,
      updatedAt: c.updatedAt,
    }));
  } catch (e) {
    console.log("加载 WebSocket 配置失败:", e);
  }
};

// 选择配置
const selectConfig = async (config) => {
  if (editingId.value) return;
  selectedConfigId.value = config.id;
  // 重新拉取最新数据
  await loadConfigs();
  const latestConfig = wsConfigs.value.find((c) => c.id === config.id);
  if (latestConfig) {
    emit("selectWsConfig", latestConfig);
  }
};

// 开始新建配置（inline 编辑模式）
const startCreateConfig = () => {
  if (!props.workspace?.id) return;
  const tempId = `temp-${Date.now()}`;
  editingId.value = tempId;
  editingName.value = t("buttons.newWebSocket");
  nextTick(() => {
    if (inlineEditInput.value) {
      inlineEditInput.value.focus();
      inlineEditInput.value.select();
    }
  });
};

// 开始重命名（inline 编辑模式）
const startRenameConfig = (config) => {
  editingId.value = config.id;
  editingName.value = config.name;
  nextTick(() => {
    if (inlineEditInput.value) {
      inlineEditInput.value.focus();
      inlineEditInput.value.select();
    }
  });
};

// 完成 inline 编辑（保存）
const finishInlineEdit = async () => {
  if (isSavingEdit.value) return;
  isSavingEdit.value = true;

  const name = editingName.value.trim();
  if (!name) {
    cancelInlineEdit();
    isSavingEdit.value = false;
    return;
  }

  try {
    if (editingId.value.startsWith("temp-")) {
      const id = await invoke("save_ws_config", {
        workspaceId: props.workspace.id,
        id: null,
        name: name,
        url: "",
        headers: [],
        params: [],
      });
      showToast(t("toast.apiSaved"), "success");
      await loadConfigs();
      const newConfig = wsConfigs.value.find((c) => c.id === id);
      if (newConfig) {
        selectedConfigId.value = newConfig.id;
        emit("selectWsConfig", newConfig);
      }
    } else {
      const config = wsConfigs.value.find((c) => c.id === editingId.value);
      if (config) {
        await invoke("save_ws_config", {
          workspaceId: props.workspace.id,
          id: editingId.value,
          name: name,
          url: config.url || "",
          headers: config.headers || [],
          params: config.params || [],
        });
        showToast(t("toast.apiSaved"), "success");
        await loadConfigs();
      }
    }
  } catch (e) {
    console.error("保存失败:", e);
    showToast(t("toast.wsSaveFailed"), "error");
  }

  cancelInlineEdit();
  isSavingEdit.value = false;
};

// 取消 inline 编辑
const cancelInlineEdit = () => {
  editingId.value = null;
  editingName.value = "";
};

// 处理编辑输入框键盘事件
const handleEditKeydown = (e) => {
  if (e.key === "Enter") {
    e.preventDefault();
    finishInlineEdit();
  } else if (e.key === "Escape") {
    e.preventDefault();
    cancelInlineEdit();
  }
};

// 处理编辑输入框失焦
const handleEditBlur = () => {
  if (isSavingEdit.value) return;
  finishInlineEdit();
};

// 删除配置
const deleteConfig = async (id) => {
  if (!props.workspace?.id) return;
  try {
    await invoke("delete_ws_config", {
      workspaceId: props.workspace.id,
      id,
    });
    await loadConfigs();
    if (selectedConfigId.value === id) {
      selectedConfigId.value = null;
      emit("selectWsConfig", null);
    }
  } catch (e) {
    console.error("删除配置失败:", e);
  }
};

// 打开右键菜单
const openContextMenu = (e, config) => {
  e.preventDefault();
  e.stopPropagation();
  contextMenu.value = {
    visible: true,
    x: e.clientX,
    y: e.clientY,
    config: config,
  };
};

// 关闭右键菜单
const closeContextMenu = () => {
  contextMenu.value.visible = false;
};

useDialogEscape(() => contextMenu.value.visible, closeContextMenu);

// 处理右键菜单操作
const handleContextAction = async (action) => {
  closeContextMenu();
  if (action === "new") {
    startCreateConfig();
  } else if (action === "rename" && contextMenu.value.config) {
    startRenameConfig(contextMenu.value.config);
  } else if (action === "delete" && contextMenu.value.config) {
    await deleteConfig(contextMenu.value.config.id);
  }
};

// 过滤后的配置列表
const filteredConfigs = computed(() => {
  if (!searchQuery.value.trim()) return wsConfigs.value;
  const query = searchQuery.value.toLowerCase();
  return wsConfigs.value.filter(
    (c) =>
      c.name.toLowerCase().includes(query) ||
      c.url.toLowerCase().includes(query),
  );
});

// 监听 workspace 变化
watch(
  () => props.workspace?.id,
  (newId, oldId) => {
    if (newId && newId !== oldId) {
      selectedConfigId.value = null;
      cancelInlineEdit();
      loadConfigs();
    }
  },
  { immediate: true },
);

// 暴露方法
defineExpose({
  loadConfigs,
  setSelectedConfigId: (id) => {
    selectedConfigId.value = id;
  },
});
</script>

<template>
  <div class="ws-panel" @contextmenu.prevent>
    <!-- 面板头部 -->
    <div class="panel-header">
      <span class="panel-title">{{ t("nav.websocket") }}</span>
      <button
        class="action-btn"
        @click="startCreateConfig"
        :title="t('buttons.new')"
      >
        <Icon name="plus" :size="16" />
      </button>
    </div>

    <!-- 搜索框 -->
    <div class="search-box">
      <input
        v-model="searchQuery"
        type="text"
        :placeholder="t('placeholder.search')"
        class="search-input"
      />
    </div>

    <!-- 提示：需要先选择工作区 -->
    <div v-if="!props.workspace" class="empty-panel">
      {{ t("empty.selectWorkspace") }}
    </div>

    <!-- 配置列表 -->
    <div
      v-else
      class="config-list"
      @contextmenu.prevent="(e) => openContextMenu(e, null)"
    >
      <div
        v-if="filteredConfigs.length === 0 && !editingId"
        class="empty-panel"
      >
        {{
          wsConfigs.length === 0
            ? t("empty.noWsConfigs")
            : t("empty.noSearchResult")
        }}
      </div>

      <!-- 配置列表项 -->
      <div
        v-for="config in filteredConfigs"
        :key="config.id"
        class="config-item"
        :class="{
          selected: selectedConfigId === config.id,
          editing: editingId === config.id,
        }"
        @click="selectConfig(config)"
        @contextmenu.prevent.stop="(e) => openContextMenu(e, config)"
      >
        <!-- 编辑模式 -->
        <template v-if="editingId === config.id">
          <span class="ws-tag">WS</span>
          <input
            :ref="setInlineEditInput"
            v-model="editingName"
            class="inline-edit-input"
            :placeholder="t('placeholder.name')"
            @keydown="handleEditKeydown"
            @blur="handleEditBlur"
            @mousedown.stop
            @click.stop
          />
        </template>
        <!-- 正常显示 -->
        <template v-else>
          <span class="ws-tag">WS</span>
          <span class="config-name">{{ config.name }}</span>
          <button class="delete-btn" @click.stop="deleteConfig(config.id)">
            <Icon name="delete" :size="14" />
          </button>
        </template>
      </div>

      <!-- 新建时的临时项（显示在最后） -->
      <div
        v-if="editingId && editingId.startsWith('temp-')"
        class="config-item editing"
      >
        <span class="ws-tag">WS</span>
        <input
          :ref="setInlineEditInput"
          v-model="editingName"
          class="inline-edit-input"
          :placeholder="t('placeholder.name')"
          @keydown="handleEditKeydown"
          @blur="handleEditBlur"
          @mousedown.stop
          @click.stop
        />
      </div>
    </div>

    <!-- 右键菜单遮罩 -->
    <div
      v-if="contextMenu.visible"
      class="ws-menu-overlay"
      @click="closeContextMenu"
      @contextmenu.prevent
    ></div>

    <!-- 右键菜单 -->
    <div
      v-if="contextMenu.visible"
      class="ws-context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
      @contextmenu.prevent
    >
      <!-- 空白区域：只显示新建 -->
      <div
        v-if="!contextMenu.config"
        class="menu-item"
        @click="handleContextAction('new')"
      >
        <Icon name="websocket" :size="14" />
        <span>{{ t("buttons.newWebSocket") }}</span>
      </div>

      <!-- 配置项：显示重命名、删除 -->
      <template v-if="contextMenu.config">
        <div class="menu-item" @click="handleContextAction('rename')">
          <Icon name="edit" :size="14" />
          <span>{{ t("common.rename") }}</span>
        </div>
        <div class="menu-item delete" @click="handleContextAction('delete')">
          <Icon name="delete" :size="14" />
          <span>{{ t("common.delete") }}</span>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
