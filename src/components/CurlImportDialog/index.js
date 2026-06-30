import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "../../composables/useToast";
import { useI18n } from "vue-i18n";
import { useDialogEscape } from "../../composables/useDialogStack.js";

export function useCurlImportDialogSetup(props, emit) {
  const { t } = useI18n();

  const curlCommand = ref("");
  const parsedResult = ref(null);
  const apiName = ref("");
  const targetCollectionId = ref(null);
  const error = ref("");
  const loading = ref(false);
  const parsing = ref(false);

  const parseCurlCommand = async (command) => {
    if (!command.trim()) {
      parsedResult.value = null;
      apiName.value = "";
      return;
    }

    parsing.value = true;

    try {
      const result = await invoke("parse_curl", {
        curlCommand: command.trim(),
      });

      parsedResult.value = result;
      const urlPath =
        result.url.split("/").pop() || result.url.split("?")[0] || "New API";
      apiName.value = urlPath.replace(/[{}]/g, "").substring(0, 30);
    } catch {
      parsedResult.value = null;
      apiName.value = "";
    } finally {
      parsing.value = false;
    }
  };

  const readClipboardAndParse = async () => {
    parsing.value = true;

    try {
      const clipboardText = await navigator.clipboard.readText();

      if (!clipboardText || !clipboardText.trim()) {
        parsing.value = false;
        return;
      }

      try {
        const result = await invoke("parse_curl", {
          curlCommand: clipboardText.trim(),
        });

        parsedResult.value = result;
        curlCommand.value = clipboardText.trim();
        const urlPath =
          result.url.split("/").pop() || result.url.split("?")[0] || "New API";
        apiName.value = urlPath.replace(/[{}]/g, "").substring(0, 30);
      } catch {
        parsedResult.value = null;
        apiName.value = "";
      }
    } catch {
      /* ignore */
    } finally {
      parsing.value = false;
    }
  };

  const importCurl = async () => {
    if (!parsedResult.value) {
      error.value = t("curlImport.parseFailed");
      return;
    }

    if (!apiName.value.trim()) {
      error.value = t("placeholder.name");
      return;
    }

    loading.value = true;
    error.value = "";

    try {
      const newApi = await invoke("create_api", {
        workspaceId: props.workspaceId,
        name: apiName.value.trim(),
        method: parsedResult.value.method,
        url: parsedResult.value.url,
        parentId: targetCollectionId.value,
        headers: parsedResult.value.headers,
        body: parsedResult.value.body,
        bodyType: parsedResult.value.body_type,
        formFields: parsedResult.value.form_fields,
      });

      showToast(t("toast.curlImportSuccess"), "success");
      emit("imported", newApi);
      close();
    } catch (e) {
      error.value =
        typeof e === "string" ? e : e.message || t("toast.curlImportFailed");
    } finally {
      loading.value = false;
    }
  };

  const resetForm = () => {
    curlCommand.value = "";
    parsedResult.value = null;
    apiName.value = "";
    targetCollectionId.value = null;
    error.value = "";
    loading.value = false;
    parsing.value = false;
  };

  const close = () => {
    resetForm();
    emit("close");
  };

  // ESC 键关闭
  useDialogEscape(() => props.visible, close);

  watch(
    () => curlCommand.value,
    (newVal) => {
      if (newVal.trim() && !parsedResult.value) {
        parseCurlCommand(newVal);
      }
    },
  );

  watch(
    () => props.visible,
    async (visible) => {
      if (visible) {
        targetCollectionId.value = props.initialCollectionId || null;
        await readClipboardAndParse();
      }
    },
  );

  return {
    curlCommand,
    parsedResult,
    apiName,
    targetCollectionId,
    error,
    loading,
    parsing,
    importCurl,
    close,
  };
}
