import { ref, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { useDialogEscape } from "../../composables/useDialogStack.js";
import { showToast } from "../../composables/useToast";

export function useAiSettingsSetup(props, emit) {
  const { t } = useI18n();
  const loading = ref(false);

  // AI 设置
  const aiApiEndpoint = ref("https://api.openai.com/v1");
  const aiApiKey = ref("");
  const apiKeyPlaceholder = ref(""); // placeholder 文本
  const hasApiKey = ref(false); // 是否已配置 API Key
  const aiModel = ref("");
  const aiModels = ref([]);
  const loadingModels = ref(false);
  const showModelDropdown = ref(false);
  const aiTimeout = ref(600); // 默认 10 分钟
  const customHeaders = ref([]);

  // 加载设置
  const loadSettings = async () => {
    try {
      loading.value = true;
      const settings = await invoke("get_settings");
      // 加载 AI 设置
      if (settings.ai) {
        aiApiEndpoint.value =
          settings.ai.api_endpoint || "https://api.openai.com/v1";
        // encrypted_api_key: 包含 **** 表示已配置部分显示，空字符串表示未配置
        const encryptedKey = settings.ai.encrypted_api_key || "";
        hasApiKey.value = encryptedKey.includes("****");
        if (hasApiKey.value) {
          // 已配置，显示部分 API Key
          aiApiKey.value = encryptedKey;
          apiKeyPlaceholder.value = "";
        } else {
          aiApiKey.value = "";
          apiKeyPlaceholder.value = "请输入 API Key";
        }
        aiModel.value = settings.ai.model || "";
        aiTimeout.value = settings.ai.timeout || 600;
        // 加载自定义请求头
        if (
          settings.ai.custom_headers &&
          settings.ai.custom_headers.length > 0
        ) {
          customHeaders.value = settings.ai.custom_headers.map((h) => ({
            key: h.key || "",
            value: h.value || "",
            enabled: h.enabled ?? true,
            description: h.description || "",
          }));
        } else {
          customHeaders.value = [];
        }
      }
    } catch (e) {
      console.error("Failed to load AI settings:", e);
    } finally {
      loading.value = false;
    }
  };

  // 获取模型列表（使用当前输入的配置）
  const fetchModels = async () => {
    closeModelDropdown();

    const currentApiKey = aiApiKey.value.trim();
    // 如果包含 ****，说明用户没有修改，使用保存的 key
    const apiKeyToSend = currentApiKey.includes("****")
      ? null
      : currentApiKey || null;

    try {
      loadingModels.value = true;
      const models = await invoke("get_ai_models", {
        apiEndpoint: aiApiEndpoint.value,
        apiKey: apiKeyToSend,
        customHeaders: getEnabledHeaders(),
      });
      aiModels.value = models || [];
      showToast(t("settings.aiFetchModelsSuccess"), "success");
    } catch (e) {
      console.error("Failed to fetch AI models:", e);
      aiModels.value = [];
      showToast(t("settings.aiFetchModelsFailed"), "error");
    } finally {
      loadingModels.value = false;
    }
  };

  // 获取启用的请求头
  const getEnabledHeaders = () => {
    return customHeaders.value
      .filter((h) => h.enabled && h.key.trim())
      .map((h) => ({
        key: h.key,
        value: h.value,
        enabled: h.enabled,
        description: h.description?.trim() || null,
      }));
  };

  // 选择模型
  const selectModel = (model) => {
    aiModel.value = model;
    showModelDropdown.value = false;
  };

  // 切换下拉框
  const toggleModelDropdown = () => {
    showModelDropdown.value = !showModelDropdown.value;
  };

  // 关闭下拉框
  const closeModelDropdown = () => {
    showModelDropdown.value = false;
  };

  // 点击外部关闭下拉框
  const handleClickOutside = (event) => {
    const container = document.querySelector(".model-input-container");
    if (container && !container.contains(event.target)) {
      showModelDropdown.value = false;
    }
  };

  // 添加请求头
  const addHeader = () => {
    customHeaders.value.push({
      key: "",
      value: "",
      enabled: true,
      description: "",
    });
  };

  // 移除请求头
  const removeHeader = (index) => {
    customHeaders.value.splice(index, 1);
  };

  // 保存设置
  const saveSettings = async () => {
    try {
      loading.value = true;

      // API Key 保存逻辑：
      // - 用户输入了新值（不包含 ****）→ 传新值让后端加密保存
      // - 用户没修改（包含 ****）→ 传 null（后端保持原值）
      // - 用户清空了 → 传空字符串（后端清空）
      const currentApiKey = aiApiKey.value.trim();
      const apiKeyToSave = currentApiKey.includes("****")
        ? null
        : currentApiKey || null;

      await invoke("update_settings", {
        timeout: 60, // 保持默认值
        aiApiEndpoint: aiApiEndpoint.value,
        aiApiKey: apiKeyToSave,
        aiModel: aiModel.value,
        aiTimeout: parseInt(aiTimeout.value) || 600,
        aiCustomHeaders: customHeaders.value
          .filter((h) => h.key.trim())
          .map((h) => ({
            key: h.key,
            value: h.value,
            enabled: h.enabled,
            description: h.description?.trim() || null,
          })),
      });

      // 保存后重新加载设置
      await loadSettings();

      emit("saved");
      emit("close");
    } catch (e) {
      console.error("Failed to save AI settings:", e);
    } finally {
      loading.value = false;
    }
  };

  // 关闭面板
  const close = () => {
    emit("close");
  };

  // ESC 键关闭
  useDialogEscape(() => props.visible, close);

  // 每次打开面板时重新加载设置
  watch(
    () => props.visible,
    (visible) => {
      if (visible) {
        loadSettings();
      }
    },
  );

  onMounted(() => {
    if (props.visible) {
      loadSettings();
    }
    document.addEventListener("click", handleClickOutside);
  });

  onUnmounted(() => {
    document.removeEventListener("click", handleClickOutside);
  });

  return {
    t,
    aiApiEndpoint,
    aiApiKey,
    apiKeyPlaceholder,
    hasApiKey,
    aiModel,
    aiModels,
    loadingModels,
    showModelDropdown,
    aiTimeout,
    customHeaders,
    fetchModels,
    selectModel,
    toggleModelDropdown,
    closeModelDropdown,
    addHeader,
    removeHeader,
    loading,
    saveSettings,
    close,
  };
}
