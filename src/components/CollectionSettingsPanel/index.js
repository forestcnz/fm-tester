import { ref, reactive, watch, onUnmounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "../../composables/useToast.js";
import { COMMON_HEADERS } from "../HeaderAutocomplete/index.js";

// 导出 composable 函数
export function useCollectionSettingsSetup(props, emit) {
  const { t } = useI18n();

  const tabs = [
    { key: "headers", name: t("tabs.headers") },
    { key: "variables", name: t("tabs.variables") },
    { key: "scripts", name: t("tabs.scripts") },
  ];

  const activeTab = ref("headers");

  // 本地设置状态
  const localSettings = reactive({
    name: props.collection?.name || "",
    commonHeaders: [],
    collectionVariables: [],
    preScript: "",
    postScript: "",
  });

  // 防抖保存定时器
  let saveTimer = null;

  // 保存中状态
  const saving = ref(false);

  // 跳过下一次保存的标记（添加空项后使用）
  let skipNextSave = false;

  // 初始化数据
  const initSettings = async () => {
    localSettings.name = props.collection?.name || "";

    // 通用请求头
    if (
      props.collection?.common_headers &&
      props.collection.common_headers.length > 0
    ) {
      localSettings.commonHeaders = props.collection.common_headers.map(
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

    // 集合变量
    if (
      props.collection?.collection_variables &&
      props.collection.collection_variables.length > 0
    ) {
      localSettings.collectionVariables =
        props.collection.collection_variables.map((v) => ({
          key: v.key || "",
          value: v.value || "",
          enabled: v.enabled ?? true,
          description: v.description || "",
        }));
    } else {
      localSettings.collectionVariables = [];
    }

    // 从后端加载脚本
    if (props.workspaceId && props.collection?.id) {
      try {
        const preScript = await invoke("get_script", {
          workspaceId: props.workspaceId,
          targetType: "collection",
          targetId: props.collection.id,
          scriptKind: "pre",
        });
        const postScript = await invoke("get_script", {
          workspaceId: props.workspaceId,
          targetType: "collection",
          targetId: props.collection.id,
          scriptKind: "post",
        });
        localSettings.preScript = preScript || "";
        localSettings.postScript = postScript || "";
      } catch (e) {
        // 其他异常情况（如网络错误、数据库连接错误等）
        console.warn("加载集合脚本异常:", e);
        localSettings.preScript = "";
        localSettings.postScript = "";
      }
    }
  };

  // 初始化标记（避免初始化时触发保存）
  let initialized = false;

  // 监听 collection 变化
  watch(
    () => props.collection,
    () => {
      initialized = false;
      initSettings();
      // 延迟标记为已初始化，避免初始化触发保存
      setTimeout(() => {
        initialized = true;
      }, 100);
    },
    { immediate: true },
  );

  // 防抖保存（500ms 后保存）
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

  // 监听请求头变化
  watch(
    () => localSettings.commonHeaders,
    () => {
      debouncedSave();
    },
    { deep: true },
  );

  // 监听变量变化
  watch(
    () => localSettings.collectionVariables,
    () => {
      debouncedSave();
    },
    { deep: true },
  );

  // 组件卸载时清理定时器并强制保存
  onUnmounted(() => {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveSettings(); // 立即保存未完成的变更
    }
  });

  // 添加请求头
  const addHeader = () => {
    skipNextSave = true; // 添加空项后跳过下一次保存触发
    localSettings.commonHeaders.push({
      key: "",
      value: "",
      enabled: true,
      description: "",
    });
  };

  // 移除请求头
  const removeHeader = (index) => {
    localSettings.commonHeaders.splice(index, 1);
  };

  // 添加变量
  const addVariable = () => {
    skipNextSave = true; // 添加空项后跳过下一次保存触发
    localSettings.collectionVariables.push({
      key: "",
      value: "",
      enabled: true,
      description: "",
    });
  };

  // 移除变量
  const removeVariable = (index) => {
    localSettings.collectionVariables.splice(index, 1);
  };

  // Header 自动补全相关
  const showHeaderKeyAutocomplete = ref(false);
  const showHeaderValueAutocomplete = ref(false);
  const headerAutocompletePosition = ref({ top: 0, left: 0 });
  const headerAutocompleteWidth = ref("280px");
  const headerSelectedIndex = ref(0);
  const headerKeyInput = ref("");
  const headerValueInput = ref("");
  const activeHeaderIndex = ref(-1);

  const filteredHeaderKeys = computed(() => {
    const input = headerKeyInput.value.toLowerCase();
    if (!input) {
      return COMMON_HEADERS.slice(0, 10);
    }
    return COMMON_HEADERS.filter((h) =>
      h.key.toLowerCase().includes(input),
    ).slice(0, 10);
  });

  const filteredHeaderValues = computed(() => {
    const header = COMMON_HEADERS.find(
      (h) => h.key.toLowerCase() === headerKeyInput.value.toLowerCase(),
    );
    if (!header || !header.values.length) return [];

    const input = headerValueInput.value.toLowerCase();
    if (!input) return header.values;

    return header.values.filter((v) => v.toLowerCase().includes(input));
  });

  const handleHeaderKeyInput = (value, index, event) => {
    headerKeyInput.value = value;
    activeHeaderIndex.value = index;
    headerSelectedIndex.value = 0;

    if (value.length > 0) {
      const rect = event.target.getBoundingClientRect();
      headerAutocompletePosition.value = {
        top: rect.bottom + window.scrollY,
        left: rect.left + window.scrollX,
      };
      headerAutocompleteWidth.value = `${rect.width}px`;
      showHeaderKeyAutocomplete.value = true;
      showHeaderValueAutocomplete.value = false;
    } else {
      showHeaderKeyAutocomplete.value = false;
    }
  };

  const handleHeaderValueInput = (key, value, index, event) => {
    headerKeyInput.value = key;
    headerValueInput.value = value;
    activeHeaderIndex.value = index;
    headerSelectedIndex.value = 0;

    // 如果包含 {{，不显示 header 值补全（让变量补全生效）
    if (value.includes("{{")) {
      showHeaderValueAutocomplete.value = false;
      return;
    }

    const header = COMMON_HEADERS.find(
      (h) => h.key.toLowerCase() === key.toLowerCase(),
    );

    if (header && header.values.length > 0) {
      // 获取位置：优先使用 event.target，否则尝试从 headerValueRefs 获取
      let rect = null;
      if (event && event.target) {
        rect = event.target.getBoundingClientRect();
      } else if (headerValueRefs[index]) {
        const inputEl = headerValueRefs[index]?.getInputRef?.();
        if (inputEl) {
          rect = inputEl.getBoundingClientRect();
        }
      }

      if (rect) {
        headerAutocompletePosition.value = {
          top: rect.bottom + window.scrollY,
          left: rect.left + window.scrollX,
        };
        headerAutocompleteWidth.value = `${rect.width}px`;
        showHeaderValueAutocomplete.value = true;
        showHeaderKeyAutocomplete.value = false;
      }
    } else {
      showHeaderValueAutocomplete.value = false;
    }
  };

  const selectHeaderKey = (item) => {
    if (
      activeHeaderIndex.value >= 0 &&
      localSettings.commonHeaders[activeHeaderIndex.value]
    ) {
      localSettings.commonHeaders[activeHeaderIndex.value].key = item.key;
    }
    hideHeaderAutocomplete();
  };

  const selectHeaderValue = (value) => {
    if (
      activeHeaderIndex.value >= 0 &&
      localSettings.commonHeaders[activeHeaderIndex.value]
    ) {
      localSettings.commonHeaders[activeHeaderIndex.value].value = value;
    }
    hideHeaderAutocomplete();
  };

  const hideHeaderAutocomplete = () => {
    showHeaderKeyAutocomplete.value = false;
    showHeaderValueAutocomplete.value = false;
    headerSelectedIndex.value = 0;
  };

  const handleHeaderKeyNavigation = (e) => {
    if (!showHeaderKeyAutocomplete.value && !showHeaderValueAutocomplete.value)
      return false;

    const list = showHeaderKeyAutocomplete.value
      ? filteredHeaderKeys.value
      : showHeaderValueAutocomplete.value
        ? filteredHeaderValues.value
        : [];

    if (!list.length) return false;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      headerSelectedIndex.value = Math.min(
        headerSelectedIndex.value + 1,
        list.length - 1,
      );
      return true;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      headerSelectedIndex.value = Math.max(headerSelectedIndex.value - 1, 0);
      return true;
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (
        showHeaderKeyAutocomplete.value &&
        filteredHeaderKeys.value[headerSelectedIndex.value]
      ) {
        selectHeaderKey(filteredHeaderKeys.value[headerSelectedIndex.value]);
      } else if (
        showHeaderValueAutocomplete.value &&
        filteredHeaderValues.value[headerSelectedIndex.value]
      ) {
        selectHeaderValue(
          filteredHeaderValues.value[headerSelectedIndex.value],
        );
      }
      return true;
    } else if (e.key === "Escape") {
      hideHeaderAutocomplete();
      return true;
    } else if (e.key === "Tab") {
      hideHeaderAutocomplete();
      return false;
    }
    return false;
  };

  // Header value 输入框 refs（用于获取位置）
  const headerValueRefs = {};

  // 处理 VariableHighlight 的 input 事件（值变化）
  const handleHeaderValueChange = (key, value, index) => {
    headerKeyInput.value = key;
    headerValueInput.value = value;
    activeHeaderIndex.value = index;

    // 如果包含 {{，不显示 header 值补全
    if (value.includes("{{")) {
      showHeaderValueAutocomplete.value = false;
      return;
    }

    // 检查是否有预设值
    const header = COMMON_HEADERS.find(
      (h) => h.key.toLowerCase() === key.toLowerCase(),
    );

    if (header && header.values.length > 0 && headerValueRefs[index]) {
      // 延迟获取位置，确保 DOM 已更新
      setTimeout(() => {
        const inputEl = headerValueRefs[index]?.getInputRef?.();
        if (inputEl) {
          const rect = inputEl.getBoundingClientRect();
          headerAutocompletePosition.value = {
            top: rect.bottom + window.scrollY,
            left: rect.left + window.scrollX,
          };
          headerAutocompleteWidth.value = `${rect.width}px`;
          showHeaderValueAutocomplete.value = true;
          showHeaderKeyAutocomplete.value = false;
        }
      }, 50);
    } else {
      showHeaderValueAutocomplete.value = false;
    }
  };

  // 处理脚本更新
  const handleScriptUpdate = (updated) => {
    localSettings.preScript = updated.preScript || "";
    localSettings.postScript = updated.postScript || "";
  };

  // 保存脚本
  const saveScripts = async (scriptKind) => {
    if (!props.workspaceId || !props.collection?.id) return;
    try {
      await invoke("save_script", {
        workspaceId: props.workspaceId,
        targetType: "collection",
        targetId: props.collection.id,
        scriptKind: scriptKind,
        content:
          scriptKind === "pre"
            ? localSettings.preScript
            : localSettings.postScript,
      });
      showToast(t("toast.scriptSaved"), "success");
    } catch (e) {
      console.error("保存集合脚本失败:", e);
      showToast(t("toast.scriptSaveFailed"), "error");
    }
  };

  // 保存设置
  const saveSettings = async () => {
    if (saving.value) return; // 防止重复保存

    // 检查是否有空项（key 为空），如果有则不保存
    const hasEmptyHeaders = localSettings.commonHeaders.some(
      (h) => !h.key.trim(),
    );
    const hasEmptyVariables = localSettings.collectionVariables.some(
      (v) => !v.key.trim(),
    );

    if (hasEmptyHeaders || hasEmptyVariables) {
      // 有空项时不保存，等用户填写后再保存
      return;
    }

    saving.value = true;
    try {
      // 过滤非空的请求头
      const validHeaders = localSettings.commonHeaders
        .filter((h) => h.key.trim())
        .map((h) => ({
          key: h.key,
          value: h.value,
          enabled: h.enabled,
          description: h.description?.trim() || null,
        }));

      // 过滤非空的变量
      const validVariables = localSettings.collectionVariables
        .filter((v) => v.key.trim())
        .map((v) => ({
          key: v.key,
          value: v.value,
          enabled: v.enabled,
          description: v.description?.trim() || null,
        }));

      // 保存集合设置（请求头、变量）
      await invoke("update_collection_settings", {
        workspaceId: props.workspaceId,
        id: props.collection.id,
        commonHeaders: validHeaders.length > 0 ? validHeaders : null,
        collectionVariables: validVariables.length > 0 ? validVariables : null,
      });

      emit("save");
    } catch (e) {
      console.error("保存集合设置失败:", e);
    } finally {
      saving.value = false;
    }
  };

  return {
    t,
    activeTab,
    tabs,
    localSettings,
    saving,
    addHeader,
    removeHeader,
    addVariable,
    removeVariable,
    handleScriptUpdate,
    saveScripts,
    saveSettings,
    // Header 自动补全
    showHeaderKeyAutocomplete,
    showHeaderValueAutocomplete,
    headerAutocompletePosition,
    headerAutocompleteWidth,
    headerSelectedIndex,
    filteredHeaderKeys,
    filteredHeaderValues,
    handleHeaderKeyInput,
    handleHeaderValueInput,
    handleHeaderValueChange,
    selectHeaderKey,
    selectHeaderValue,
    hideHeaderAutocomplete,
    handleHeaderKeyNavigation,
    headerValueRefs,
    // 变量列表
    variables: computed(() => props.variables),
  };
}
