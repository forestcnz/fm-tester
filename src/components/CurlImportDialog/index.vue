<script setup>
import { useI18n } from "vue-i18n";
import { computed } from "vue";
import { useCurlImportDialogSetup } from "./index.js";

const { t } = useI18n();

const props = defineProps({
  visible: Boolean,
  workspaceId: String,
  initialCollectionId: String,
  collections: Array,
});

const emit = defineEmits(["close", "imported"]);

const {
  curlCommand,
  parsedResult,
  apiName,
  targetCollectionId,
  error,
  loading,
  parsing,
  importCurl,
  close,
} = useCurlImportDialogSetup(props, emit);

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
    <div class="dialog curl-import-dialog">
      <div class="dialog-header">
        <span class="dialog-title">{{ t("dialogs.importCurl") }}</span>
        <span class="dialog-close" @click="close">×</span>
      </div>

      <div class="dialog-body">
        <div v-if="parsing" class="loading-state">
          <span class="loading-text">{{ t("curlImport.clipboardRead") }}</span>
        </div>

        <template v-else>
          <div v-if="!parsedResult" class="form-group">
            <label>{{ t("curlImport.curlCommand") }}</label>
            <textarea
              v-model="curlCommand"
              class="curl-textarea"
              :placeholder="t('curlImport.pasteHint')"
              rows="4"
            ></textarea>
          </div>

          <div v-if="parsedResult" class="parse-result">
            <div class="result-row">
              <span
                class="method-tag"
                :class="parsedResult.method.toLowerCase()"
                >{{ parsedResult.method }}</span
              >
              <span class="url-text" :title="parsedResult.url">{{
                parsedResult.url
              }}</span>
            </div>
          </div>

          <div class="form-group">
            <label>{{ t("curlImport.apiName") }}</label>
            <input
              v-model="apiName"
              type="text"
              class="form-input"
              :placeholder="t('placeholder.name')"
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
        </template>
      </div>

      <div class="dialog-footer">
        <button
          class="btn-primary"
          :disabled="loading || parsing || !apiName.trim() || !parsedResult"
          @click="importCurl"
        >
          {{ loading ? t("curlImport.importing") : t("buttons.import") }}
        </button>
        <button class="btn-secondary" @click="close">
          {{ t("common.cancel") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped src="./style.css"></style>
