import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useWorkspaceIO } from "../../composables/useWorkspaceIO";
import { useDialogEscape } from "../../composables/useDialogStack.js";

export function useWorkspaceImportSetup(props, emit) {
  const { t } = useI18n();
  const { importWorkspace, previewImport, executeImport } =
    useWorkspaceIO();

  const selectedFile = ref(null);
  const fileContent = ref(null);
  const preview = ref(null);
  const error = ref("");
  const loading = ref(false);

  const selectFile = async () => {
    try {
      error.value = "";
      preview.value = null;

      const content = await importWorkspace();
      if (!content) return;

      fileContent.value = content;

      const filePath =
        typeof content === "string"
          ? "workspace-export.json"
          : content.path || "workspace-export.json";
      selectedFile.value = filePath;

      const previewResult = await previewImport(content);
      preview.value = previewResult;
    } catch (e) {
      console.error("选择文件失败:", e);
      error.value =
        typeof e === "string" ? e : e.message || t("workspace.importFailed");
      selectedFile.value = null;
      fileContent.value = null;
    }
  };

  const confirmImport = async () => {
    if (!fileContent.value || !preview.value) return;

    loading.value = true;
    error.value = "";

    try {
      const workspace = await executeImport(fileContent.value);
      emit("imported", workspace);
      close();
    } catch (e) {
      error.value =
        typeof e === "string" ? e : e.message || t("workspace.importFailed");
    } finally {
      loading.value = false;
    }
  };

  const resetForm = () => {
    selectedFile.value = null;
    fileContent.value = null;
    preview.value = null;
    error.value = "";
    loading.value = false;
  };

  const close = () => {
    resetForm();
    emit("close");
  };

  useDialogEscape(() => props.visible, close);

  watch(
    () => props.visible,
    (visible) => {
      if (!visible) {
        resetForm();
      }
    },
  );

  return {
    selectedFile,
    preview,
    error,
    loading,
    selectFile,
    confirmImport,
    close,
  };
}
