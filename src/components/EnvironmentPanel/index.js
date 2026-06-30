import { ref, reactive, watch, onUnmounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "../../composables/useToast.js";

export function useEnvPanelSetup(props, emit) {
  const { t } = useI18n();

  const tabs = [
    { key: "variables", name: t("tabs.variables") },
    { key: "headers", name: t("tabs.headers") },
    { key: "scripts", name: t("tabs.scripts") },
  ];

  const activeTab = ref("variables");

  const localSettings = reactive({
    name: props.activeEnvironment?.name || "",
    variables: [],
    commonHeaders: [],
    preScript: "",
    postScript: "",
  });

  let saveTimer = null;
  const saving = ref(false);
  let skipNextSave = false;
  let initialized = false;

  const initSettings = async () => {
    localSettings.name = props.activeEnvironment?.name || "";

    if (
      props.activeEnvironment?.variables &&
      props.activeEnvironment.variables.length > 0
    ) {
      localSettings.variables = props.activeEnvironment.variables.map((v) => ({
        key: v.key || "",
        value: v.value || "",
        enabled: v.enabled ?? true,
        description: v.description || "",
      }));
    } else {
      localSettings.variables = [];
    }

    if (
      props.activeEnvironment?.common_headers &&
      props.activeEnvironment.common_headers.length > 0
    ) {
      localSettings.commonHeaders = props.activeEnvironment.common_headers.map(
        (h) => ({
          key: h.key || "",
          value: h.value || "",
          enabled: h.enabled ?? true,
          description: h.description || "",
        }),
      );
    } else {
      localSettings.commonHeaders = [];
    }

    const envId = props.activeEnvironment?.id;
    if (props.workspaceId && envId) {
      try {
        const preScript = await invoke("get_script", {
          workspaceId: props.workspaceId,
          targetType: "environment",
          targetId: envId,
          scriptKind: "pre",
        });
        const postScript = await invoke("get_script", {
          workspaceId: props.workspaceId,
          targetType: "environment",
          targetId: envId,
          scriptKind: "post",
        });
        localSettings.preScript = preScript || "";
        localSettings.postScript = postScript || "";
      } catch (e) {
        console.error("加载环境脚本失败:", e);
        localSettings.preScript = "";
        localSettings.postScript = "";
      }
    }
  };

  watch(
    () => props.activeEnvironment,
    () => {
      initialized = false;
      initSettings();
      setTimeout(() => {
        initialized = true;
      }, 100);
    },
    { immediate: true },
  );

  const debouncedSave = () => {
    if (!initialized) return;
    if (skipNextSave) {
      skipNextSave = false;
      return;
    }
    if (saveTimer) {
      clearTimeout(saveTimer);
    }
    saveTimer = setTimeout(() => {
      saveSettings();
    }, 500);
  };

  watch(
    () => localSettings.variables,
    () => {
      debouncedSave();
    },
    { deep: true },
  );

  watch(
    () => localSettings.commonHeaders,
    () => {
      debouncedSave();
    },
    { deep: true },
  );

  onUnmounted(() => {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveSettings();
    }
  });

  const addVariable = () => {
    skipNextSave = true;
    localSettings.variables.push({
      key: "",
      value: "",
      enabled: true,
      description: "",
    });
  };

  const removeVariable = (index) => {
    localSettings.variables.splice(index, 1);
  };

  const addHeader = () => {
    skipNextSave = true;
    localSettings.commonHeaders.push({
      key: "",
      value: "",
      enabled: true,
      description: "",
    });
  };

  const removeHeader = (index) => {
    localSettings.commonHeaders.splice(index, 1);
  };

  const handleScriptUpdate = (updated) => {
    localSettings.preScript = updated.preScript || "";
    localSettings.postScript = updated.postScript || "";
  };

  const saveScripts = async (scriptKind) => {
    if (!props.workspaceId || !props.activeEnvironment?.id) return;
    try {
      await invoke("save_script", {
        workspaceId: props.workspaceId,
        targetType: "environment",
        targetId: props.activeEnvironment.id,
        scriptKind: scriptKind,
        content:
          scriptKind === "pre"
            ? localSettings.preScript
            : localSettings.postScript,
      });
      showToast(t("toast.scriptSaved"), "success");
    } catch (e) {
      console.error("保存环境脚本失败:", e);
      showToast(t("toast.scriptSaveFailed"), "error");
    }
  };

  const saveSettings = async () => {
    if (saving.value) return;

    saving.value = true;
    try {
      const validVariables = localSettings.variables
        .filter((v) => v.key.trim())
        .map((v) => ({
          key: v.key,
          value: v.value,
          enabled: v.enabled,
          description: v.description?.trim() || null,
        }));

      const validHeaders = localSettings.commonHeaders
        .filter((h) => h.key.trim())
        .map((h) => ({
          key: h.key,
          value: h.value,
          enabled: h.enabled,
          description: h.description?.trim() || null,
        }));

      const environment = {
        id: props.activeEnvironment.id,
        name: props.activeEnvironment.name,
        variables: validVariables,
        common_headers: validHeaders.length > 0 ? validHeaders : null,
      };

      await invoke("save_environment", {
        workspaceId: props.workspaceId,
        environment,
      });

      emit("saveVariables", {
        variables: validVariables,
        common_headers: validHeaders.length > 0 ? validHeaders : null,
      });
    } catch (e) {
      console.error("保存环境设置失败:", e);
    } finally {
      saving.value = false;
    }
  };

  const handleInputChange = () => {
    debouncedSave();
  };

  return {
    t,
    activeTab,
    tabs,
    localSettings,
    saving,
    addVariable,
    removeVariable,
    addHeader,
    removeHeader,
    handleScriptUpdate,
    saveScripts,
    saveSettings,
    handleInputChange,
    variables: computed(() => props.activeEnvironment?.variables || []),
  };
}
