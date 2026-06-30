<script setup>
import { useI18n } from "vue-i18n";
import { nextTick, ref, watch } from "vue";
import { useOrchestrationPanelSetup } from "./index.js";
import Icon from "../../Icon/index.vue";

const { t } = useI18n();

const props = defineProps({
  workspace: Object,
});

const emit = defineEmits(["selectOrchestration"]);

const inlineEditInput = ref(null);

const {
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
  finishInlineEdit,
  handleEditKeydown,
  handleContextAction,
  dragState,
  onMouseDown,
} = useOrchestrationPanelSetup(props, emit);

watch(editingItem, (val) => {
  if (val) {
    nextTick(() => {
      if (inlineEditInput.value) {
        inlineEditInput.value.focus();
        inlineEditInput.value.select();
      }
    });
  }
});

const handleEditBlur = () => {
  if (isSavingEdit.value) return;
  finishInlineEdit();
};

defineExpose({
  loadOrchestrations,
});
</script>

<template>
  <div class="orchestration-panel">
    <div class="panel-header">
      <span class="panel-title">{{ t("nav.orchestration") }}</span>
      <div class="panel-actions">
        <button class="add-btn" @click="handleContextAction('new')">
          <Icon name="add" :size="14" />
        </button>
      </div>
    </div>

    <div v-if="!props.workspace" class="empty-panel">
      {{ t("empty.selectWorkspace") }}
    </div>

    <div
      v-else
      class="orchestration-list"
      @contextmenu.prevent="(e) => openContextMenu(e, null)"
    >
      <div
        v-if="orchestrations.length === 0 && !editingItem"
        class="empty-panel"
      >
        {{ t("orchestration.noOrchestrations") }}
      </div>

      <template v-for="orch in orchestrations" :key="orch.id">
        <div
          v-if="editingItem && !editingItem.isNew && editingItem.id === orch.id"
          class="orchestration-item editing"
          :style="{ paddingLeft: '16px' }"
          @mousedown.stop
          @click.stop
        >
          <span class="orch-icon"
            ><Icon name="orchestration" :size="14"
          /></span>
          <input
            :ref="(el) => (inlineEditInput = el)"
            v-model="editingName"
            class="inline-edit-input"
            :placeholder="t('placeholder.name')"
            @keydown="handleEditKeydown"
            @blur="handleEditBlur"
            @mousedown.stop
            @click.stop
          />
        </div>
        <div
          v-else
          class="orchestration-item"
          :class="{
            selected: selectedOrchestration === orch.id,
            'context-target':
              contextMenu.visible && contextMenu.item?.id === orch.id,
            dragging: dragState.draggingId === orch.id,
            'dragover-before':
              dragState.dragOverId === orch.id &&
              dragState.dragOverPosition === 'before',
            'dragover-after':
              dragState.dragOverId === orch.id &&
              dragState.dragOverPosition === 'after',
          }"
          :data-orch-id="orch.id"
          :style="{ paddingLeft: '16px' }"
          @click="selectOrchestrationItem(orch)"
          @contextmenu.prevent="(e) => openContextMenu(e, orch)"
          @mousedown="(e) => onMouseDown(e, orch)"
        >
          <span class="orch-icon"
            ><Icon name="orchestration" :size="14"
          /></span>
          <span class="orch-name">{{ orch.name }}</span>
          <span class="orch-count"
            >{{ orch.step_count }} {{ t("orchestration.steps") }}</span
          >
        </div>
      </template>

      <div
        v-if="editingItem && editingItem.isNew"
        class="orchestration-item editing"
        :style="{ paddingLeft: '16px' }"
        @mousedown.stop
        @click.stop
      >
        <span class="orch-icon"><Icon name="orchestration" :size="14" /></span>
        <input
          :ref="(el) => (inlineEditInput = el)"
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

    <div
      v-if="contextMenu.visible"
      class="context-menu-overlay"
      @click="closeContextMenu"
    ></div>

    <div
      v-if="contextMenu.visible"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <template v-if="!contextMenu.item">
        <div class="menu-item" @click="handleContextAction('new')">
          <span class="menu-icon"><Icon name="add" :size="14" /></span>
          <span>{{ t("orchestration.newOrchestration") }}</span>
        </div>
      </template>
      <template v-else>
        <div class="menu-item" @click="handleContextAction('rename')">
          <span class="menu-icon"><Icon name="edit" :size="14" /></span>
          <span>{{ t("common.rename") }}</span>
        </div>
        <div class="menu-divider"></div>
        <div class="menu-item delete" @click="handleContextAction('delete')">
          <span class="menu-icon"><Icon name="delete" :size="14" /></span>
          <span>{{ t("common.delete") }}</span>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
