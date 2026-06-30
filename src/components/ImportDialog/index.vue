<script setup>
import { useI18n } from "vue-i18n";
import { computed } from "vue";
import { useImportDialogSetup } from "./index.js";
import Icon from "../Icon/index.vue";

const { t } = useI18n();

const props = defineProps({
  visible: Boolean,
  workspaceId: String,
  targetCollectionId: String,
  collections: Array,
});

const emit = defineEmits(["close", "imported"]);

const {
  rootName,
  targetCollectionId,
  error,
  loading,
  selectedFile,
  selectFile,
  importOpenapi,
  close,
} = useImportDialogSetup(props, emit);

const findCollectionOption = (collections, options = [], depth = 0) => {
  if (!collections || depth > 0) return options;
  for (const col of collections) {
    if (col.type === "collection") {
      options.push({ id: col.id, label: col.name, depth });
    }
  }
  return options;
};

const collectionOptions = computed(() => {
  const options = [{ id: null, label: t("import.rootLevel"), depth: -1 }];
  return findCollectionOption(props.collections, options, 0);
});
</script>

<template>
  <div v-if="visible" class="dialog-overlay" @click.self="close">
    <div class="dialog import-dialog">
      <div class="dialog-header">
        <span class="dialog-title">{{ t("dialogs.importCollection") }}</span>
        <span class="dialog-close" @click="close">×</span>
      </div>

      <div class="dialog-body">
        <div class="form-group file-group">
          <button class="btn-select-file" @click="selectFile">
            <Icon name="folder" :size="16" />
            <span v-if="selectedFile" class="file-name">{{
              selectedFile
            }}</span>
            <span v-else>{{ t("buttons.selectFile") }}</span>
          </button>
        </div>

        <div class="form-group">
          <label>{{ t("import.rootName") }}</label>
          <input
            v-model="rootName"
            type="text"
            :placeholder="t('import.rootNamePlaceholder')"
            class="form-input"
          />
        </div>

        <div class="form-group">
          <label>{{ t("import.targetCollection") }}</label>
          <select v-model="targetCollectionId" class="form-select">
            <option
              v-for="opt in collectionOptions"
              :key="opt.id"
              :value="opt.id"
            >
              {{ opt.label }}
            </option>
          </select>
        </div>

        <div v-if="error" class="error-msg">{{ error }}</div>
      </div>

      <div class="dialog-footer">
        <button
          class="btn-primary"
          :disabled="loading || !selectedFile"
          @click="importOpenapi"
        >
          {{ loading ? t("common.loading") : t("buttons.import") }}
        </button>
        <button class="btn-secondary" @click="close">
          {{ t("common.cancel") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
