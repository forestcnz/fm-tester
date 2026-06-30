import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

/**
 * 环境变量管理 composable
 * @param {Ref} currentWorkspace - 当前工作区引用
 * @param {Ref} currentNavKey - 当前导航项引用
 */
export function useEnvironment(currentWorkspace, currentNavKey) {
  const environments = ref([]);
  const activeEnvironmentId = ref(null);
  const activeEnvironment = ref(null);
  const selectedEnvironment = ref(null);
  const selectedEnvVariables = ref([]);
  const activeVariables = ref([]);
  const availableVariables = ref([]);

  const loadEnvironments = async () => {
    if (!currentWorkspace.value?.id) {
      environments.value = [];
      activeEnvironmentId.value = null;
      activeEnvironment.value = null;
      selectedEnvironment.value = null;
      selectedEnvVariables.value = [];
      activeVariables.value = [];
      return;
    }
    try {
      // 并行加载环境配置和激活变量
      const [config, variablesMap] = await Promise.all([
        invoke("get_environments", {
          workspaceId: currentWorkspace.value?.id,
        }),
        invoke("get_active_variables", {
          workspaceId: currentWorkspace.value?.id,
        }),
      ]);

      environments.value = config.environments || [];
      activeEnvironmentId.value = config.active_environment_id || null;
      if (activeEnvironmentId.value) {
        activeEnvironment.value =
          environments.value.find((e) => e.id === activeEnvironmentId.value) ||
          null;
      } else {
        activeEnvironment.value = null;
      }
      selectedEnvironment.value = null;
      selectedEnvVariables.value = [];
      activeVariables.value = Object.entries(variablesMap).map(
        ([key, value]) => ({ key, value }),
      );
    } catch (e) {
      console.error("加载环境失败:", e);
    }
  };

  // 监听 currentWorkspace 变化，自动刷新环境
  watch(currentWorkspace, async (ws) => {
    if (ws?.path) {
      await loadEnvironments();
    }
  });

  const loadActiveVariables = async () => {
    if (!currentWorkspace.value?.id) {
      activeVariables.value = [];
      return;
    }
    try {
      const variablesMap = await invoke("get_active_variables", {
        workspaceId: currentWorkspace.value?.id,
      });
      activeVariables.value = Object.entries(variablesMap).map(
        ([key, value]) => ({ key, value }),
      );
    } catch (e) {
      console.error("加载激活变量失败:", e);
    }
  };

  const loadAvailableVariables = async (itemId, itemType) => {
    if (!currentWorkspace.value?.id || !itemId) {
      availableVariables.value = [];
      return;
    }
    try {
      const variables = await invoke("get_available_variables", {
        workspaceId: currentWorkspace.value?.id,
        environmentId: activeEnvironmentId.value || null,
        itemId,
        itemType,
      });
      availableVariables.value = variables.map((v) => ({
        key: v.key,
        value: v.value,
        source: v.source,
        description: v.description,
      }));
    } catch (e) {
      console.error("加载可用变量失败:", e);
      availableVariables.value = [];
    }
  };

  const switchEnvironment = async (environmentId) => {
    if (!currentWorkspace.value?.id) return;
    try {
      await invoke("switch_environment", {
        workspaceId: currentWorkspace.value?.id,
        environmentId,
      });
      activeEnvironmentId.value = environmentId;
      activeEnvironment.value =
        environments.value.find((e) => e.id === environmentId) || null;
    } catch (e) {
      console.error("切换环境失败:", e);
    }
  };

  const selectEnvironment = (environmentId) => {
    selectedEnvironment.value =
      environments.value.find((e) => e.id === environmentId) || null;
    selectedEnvVariables.value =
      selectedEnvironment.value?.variables?.length > 0
        ? [...selectedEnvironment.value.variables]
        : [{ key: "", value: "", enabled: true }];
  };

  const saveEnvironment = async (environment) => {
    if (!currentWorkspace.value?.id) return;
    try {
      await invoke("save_environment", {
        workspaceId: currentWorkspace.value?.id,
        environment,
      });
      await loadEnvironments();
    } catch (e) {
      console.error("保存环境失败:", e);
    }
  };

  const deleteEnvironment = async (environmentId) => {
    if (!currentWorkspace.value?.id) return;
    try {
      await invoke("delete_environment", {
        workspaceId: currentWorkspace.value?.id,
        environmentId,
      });
      await loadEnvironments();
    } catch (e) {
      console.error("删除环境失败:", e);
    }
  };

  // 更新环境本地状态（由 EnvironmentPanel 组件调用，保存已在组件中完成）
  const saveEnvVariables = async (data) => {
    if (!selectedEnvironment.value) return;

    const { variables, common_headers } = data;

    // 更新 environments 列表中对应环境
    const envIndex = environments.value.findIndex(
      (e) => e.id === selectedEnvironment.value.id,
    );
    if (envIndex !== -1) {
      environments.value[envIndex].variables = variables || [];
      environments.value[envIndex].common_headers = common_headers;
    }
    // 更新 selectedEnvironment
    selectedEnvironment.value = {
      ...selectedEnvironment.value,
      variables: variables || [],
      common_headers: common_headers,
    };
    // 如果当前环境是激活环境，也更新 activeEnvironment
    if (activeEnvironment.value?.id === selectedEnvironment.value.id) {
      activeEnvironment.value = {
        ...activeEnvironment.value,
        variables: variables || [],
        common_headers: common_headers,
      };
    }
    // 更新激活变量
    await loadActiveVariables();
  };

  const onSwitchEnvironment = async (envId) => {
    await switchEnvironment(envId);
  };

  const onSelectEnvironment = (envId) => {
    selectEnvironment(envId);
  };

  const showEnvironmentInfo = computed(() => {
    return currentNavKey.value === "environment" && currentWorkspace.value;
  });

  return {
    environments,
    activeEnvironmentId,
    activeEnvironment,
    selectedEnvironment,
    selectedEnvVariables,
    activeVariables,
    availableVariables,
    loadEnvironments,
    loadActiveVariables,
    loadAvailableVariables,
    switchEnvironment,
    selectEnvironment,
    saveEnvironment,
    deleteEnvironment,
    saveEnvVariables,
    onSwitchEnvironment,
    onSelectEnvironment,
    showEnvironmentInfo,
  };
}
