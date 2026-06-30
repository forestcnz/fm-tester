import { reactive, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { showToast } from "../../composables/useToast.js";

export function useWorkspaceSettingsSetup(props, emit) {
  const { t } = useI18n();

  const localSettings = reactive({
    name: props.workspace?.name || "",
    preScript: "",
    postScript: "",
  });

  const initSettings = async () => {
    localSettings.name = props.workspace?.name || "";

    if (props.workspaceId) {
      try {
        const preScript = await invoke("get_script", {
          workspaceId: props.workspaceId,
          targetType: "workspace",
          targetId: null,
          scriptKind: "pre",
        });
        const postScript = await invoke("get_script", {
          workspaceId: props.workspaceId,
          targetType: "workspace",
          targetId: null,
          scriptKind: "post",
        });
        localSettings.preScript = preScript || "";
        localSettings.postScript = postScript || "";
      } catch (e) {
        console.error("加载工作区脚本失败:", e);
        localSettings.preScript = "";
        localSettings.postScript = "";
      }
    }
  };

  watch(
    () => props.workspace,
    () => {
      initSettings();
    },
    { immediate: true },
  );

  const handleScriptUpdate = (updated) => {
    localSettings.preScript = updated.preScript || "";
    localSettings.postScript = updated.postScript || "";
  };

  const saveSettings = async (scriptKind) => {
    if (!props.workspaceId) return;

    try {
      await invoke("save_script", {
        workspaceId: props.workspaceId,
        targetType: "workspace",
        targetId: null,
        scriptKind: scriptKind,
        content:
          scriptKind === "pre"
            ? localSettings.preScript
            : localSettings.postScript,
      });

      showToast(t("toast.scriptSaved"), "success");
      emit("save");
    } catch (e) {
      console.error("保存工作区脚本失败:", e);
      showToast(t("toast.scriptSaveFailed"), "error");
    }
  };

  return {
    localSettings,
    handleScriptUpdate,
    saveSettings,
  };
}
