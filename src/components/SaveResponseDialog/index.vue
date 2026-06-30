<script setup>
import { useI18n } from "vue-i18n";
import { useSaveResponseDialogSetup } from "./index.js";

const { t } = useI18n();

const props = defineProps({
  show: Boolean,
  defaultName: String,
});

const emit = defineEmits(["save", "cancel"]);

const { name, handleSave, handleCancel } = useSaveResponseDialogSetup(
  props,
  emit,
);
</script>

<template>
  <div v-if="show" class="dialog-overlay" @click.self="handleCancel">
    <div class="dialog-content">
      <h3>{{ t("dialogs.saveResponse") }}</h3>
      <input
        v-model="name"
        :placeholder="t('placeholder.responseName')"
        class="name-input"
        @keyup.enter="handleSave"
      />
      <div class="dialog-buttons">
        <button class="cancel-btn" @click="handleCancel">
          {{ t("common.cancel") }}
        </button>
        <button class="save-btn" @click="handleSave">
          {{ t("common.save") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style src="./style.css"></style>
