import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { showToast } from "../../composables/useToast";
import { useDialogEscape } from "../../composables/useDialogStack.js";

export function useImportDialogSetup(props, emit) {
  const { t } = useI18n();

  const content = ref("");
  const format = ref("auto");
  const rootName = ref("");
  const targetCollectionId = ref(null);
  const error = ref("");
  const loading = ref(false);
  const selectedFile = ref(null);

  const detectFormat = (fileContent, filename) => {
    try {
      const json = JSON.parse(fileContent);
      if (
        json.info &&
        (json.info._postman_id || json.info.schema?.includes("postman"))
      ) {
        return "postman";
      }
      if (json.openapi || json.swagger) {
        return "openapi";
      }
    } catch {
      /* ignore */
    }
    if (filename.endsWith(".yaml") || filename.endsWith(".yml")) {
      return "openapi";
    }
    return "auto";
  };

  const selectFile = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: false,
        filters: [
          { name: "API Collection", extensions: ["json", "yaml", "yml"] },
        ],
      });
      if (selected) {
        selectedFile.value = selected;
        const { readFile } = await import("@tauri-apps/plugin-fs");
        const fileData = await readFile(selected);
        const decoder = new TextDecoder();
        const fileContent = decoder.decode(fileData);
        content.value = fileContent;

        format.value = detectFormat(fileContent, selected);
      }
    } catch (e) {
      console.error("选择文件失败:", e);
      showToast(`${t("toast.selectFileFailed")}: ${e}`, "error");
    }
  };

  const importCollection = async () => {
    if (!content.value.trim()) {
      error.value = "请选择文件";
      return;
    }

    loading.value = true;
    error.value = "";

    try {
      const params = {
        workspaceId: props.workspaceId,
        content: content.value,
        targetCollectionId: targetCollectionId.value || null,
        rootName: rootName.value.trim() || null,
      };

      let result;
      if (format.value === "postman") {
        result = await invoke("import_postman", params);
      } else {
        result = await invoke("import_openapi", {
          ...params,
          format: format.value,
        });
      }
      showToast(t("toast.importSuccess"), "success");
      emit("imported", result);
      close();
    } catch (e) {
      error.value =
        typeof e === "string" ? e : e.message || t("toast.importFailed");
    } finally {
      loading.value = false;
    }
  };

  const resetForm = () => {
    content.value = "";
    format.value = "auto";
    rootName.value = "";
    targetCollectionId.value = null;
    error.value = "";
    selectedFile.value = null;
  };

  const close = () => {
    resetForm();
    emit("close");
  };

  // ESC 键关闭
  useDialogEscape(() => props.visible, close);

  watch(
    () => props.visible,
    (visible) => {
      if (visible) {
        targetCollectionId.value = props.targetCollectionId || null;
      }
    },
  );

  return {
    rootName,
    targetCollectionId,
    error,
    loading,
    selectedFile,
    selectFile,
    importOpenapi: importCollection,
    close,
  };
}
