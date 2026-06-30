import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { useDialogEscape } from "../../composables/useDialogStack.js";
import { useTheme } from "../../composables/useTheme.js";
import { showToast } from "../../composables/useToast.js";

/**
 * 统一设置中心 composable
 * 合并原 SettingsPanel / AISettingsPanel / GitBackupPanel 逻辑 + 外观/行为设置
 */
export function useSettingsCenter(props, emit) {
  const { t } = useI18n();
  const { setTheme } = useTheme();

  const activeCategory = ref(props.initialCategory || "general");
  const loading = ref(false);
  let suppressWatch = false;

  // 通用
  const requestTimeout = ref(60);
  const language = ref("zh-CN");

  // 行为
  const restoreWorkspace = ref(true);
  const autoSaveOnSend = ref(false);

  // AI
  const aiEndpoint = ref("https://api.openai.com/v1");
  const aiApiKey = ref("");
  const apiKeyPlaceholder = ref("");
  const hasApiKey = ref(false);
  const aiModel = ref("");
  const aiModels = ref([]);
  const loadingModels = ref(false);
  const showModelDropdown = ref(false);
  const aiTimeout = ref(600);
  const customHeaders = ref([]);

  // Git
  const repoUrl = ref("");
  const branch = ref("master");
  const username = ref("");
  const password = ref("");
  const hasPassword = ref(false);
  const gitTesting = ref(false);
  const autoBackupEnabled = ref(false);
  const autoBackupTime = ref("03:00");
  const autoBackupWorkspaceIds = ref([]);
  const workspaces = ref([]);
  const showBackupWsDropdown = ref(false);

  // 外观
  const themeId = ref("paper");
  const monoFont = ref("IBM Plex Mono");
  const animations = ref(true);

  // ===== 加载 =====
  const loadAll = async () => {
    suppressWatch = true;
    try {
      loading.value = true;
      const s = await invoke("get_settings");
      requestTimeout.value = s.request_timeout ?? 60;
      language.value = s.language || "zh-CN";

      if (s.ai) {
        aiEndpoint.value = s.ai.api_endpoint || "https://api.openai.com/v1";
        const encryptedKey = s.ai.encrypted_api_key || "";
        hasApiKey.value = encryptedKey.includes("****");
        if (hasApiKey.value) {
          aiApiKey.value = encryptedKey;
          apiKeyPlaceholder.value = "";
        } else {
          aiApiKey.value = "";
          apiKeyPlaceholder.value = t("settings.aiKeyPlaceholder");
        }
        aiModel.value = s.ai.model || "";
        aiTimeout.value = s.ai.timeout || 600;
        if (s.ai.custom_headers && s.ai.custom_headers.length > 0) {
          customHeaders.value = s.ai.custom_headers.map((h) => ({
            key: h.key || "",
            value: h.value || "",
            enabled: h.enabled ?? true,
            description: h.description || "",
          }));
        } else {
          customHeaders.value = [];
        }
      }

      if (s.appearance) {
        themeId.value = s.appearance.theme_id || "paper";
        monoFont.value = s.appearance.mono_font || "IBM Plex Mono";
        animations.value = s.appearance.animations ?? true;
      }

      if (s.behavior) {
        restoreWorkspace.value = s.behavior.restore_workspace_on_start ?? true;
        autoSaveOnSend.value = s.behavior.auto_save_on_send ?? false;
      }
    } catch (e) {
      console.error("Failed to load settings:", e);
    } finally {
      loading.value = false;
    }

    // Git 走独立命令（脱敏视图）
    try {
      const g = await invoke("get_git_backup_settings");
      repoUrl.value = g.repo_url || "";
      branch.value = g.branch || "master";
      username.value = g.username || "";
      hasPassword.value = !!g.has_password;
      password.value = "";
      autoBackupEnabled.value = !!g.auto_backup_enabled;
      autoBackupTime.value = g.auto_backup_time || "03:00";
      autoBackupWorkspaceIds.value = Array.isArray(g.auto_backup_workspace_ids)
        ? [...g.auto_backup_workspace_ids]
        : [];

      // 工作区列表用于自动备份多选
      const list = await invoke("get_workspaces");
      workspaces.value = Array.isArray(list) ? list : [];
      // 清理已失效的选择（指向已删除的工作区），避免残留脏数据
      const validIds = new Set(workspaces.value.map((w) => w.id));
      autoBackupWorkspaceIds.value = autoBackupWorkspaceIds.value.filter((id) =>
        validIds.has(id),
      );
    } catch (e) {
      console.error("加载 Git 备份配置失败:", e);
      workspaces.value = [];
    } finally {
      await nextTick();
      suppressWatch = false;
    }
  };

  // ===== 保存：通用 + AI + 外观 + 行为（统一 update_settings） =====
  const saveMain = async () => {
    // API Key 三态：包含 **** = 保持(null)、空 = 清空、其他 = 更新
    const currentApiKey = aiApiKey.value.trim();
    const apiKeyToSave = currentApiKey.includes("****")
      ? null
      : currentApiKey || null;

    return invoke("update_settings", {
      timeout: requestTimeout.value, // 修复原 AISettingsPanel 的 timeout:60 硬编码 bug
      language: language.value,
      aiApiEndpoint: aiEndpoint.value,
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
      appearance: {
        theme_id: themeId.value,
        font_size: 13,
        mono_font: monoFont.value,
        density: "comfortable",
        animations: animations.value,
      },
      behavior: {
        restore_workspace_on_start: restoreWorkspace.value,
        keep_tab_on_send: true,
        auto_save_on_send: autoSaveOnSend.value,
      },
    });
  };

  // ===== 保存：Git（独立命令，密码三态） =====
  const saveGit = async () => {
    const pwd = password.value.trim();
    const result = await invoke("update_git_backup_settings", {
      repoUrl: repoUrl.value.trim(),
      branch: branch.value.trim() || "master",
      username: username.value.trim(),
      password: pwd ? pwd : null,
    });
    hasPassword.value = !!result.has_password;
    password.value = "";
    return result;
  };

  // ===== 自动保存：主设置（静默，外观实时应用） =====
  const autoSaveMain = async () => {
    if (suppressWatch) return;
    try {
      await saveMain();
      applyAppearance();
    } catch (e) {
      console.error("自动保存设置失败:", e);
      showToast(`${t("settings.saveFailed")}: ${e}`, "error");
    }
  };

  // ===== 自动保存：Git 配置（不含密码，保持原凭据） =====
  const saveGitConfig = async () => {
    if (suppressWatch) return;
    try {
      const result = await invoke("update_git_backup_settings", {
        repoUrl: repoUrl.value.trim(),
        branch: branch.value.trim() || "master",
        username: username.value.trim(),
        password: null,
      });
      hasPassword.value = !!result.has_password;
    } catch (e) {
      console.error("自动保存 Git 配置失败:", e);
      showToast(`${t("settings.saveFailed")}: ${e}`, "error");
    }
  };

  // ===== 自动备份设置（开关 + 每日时刻 + 目标工作区） =====
  const saveAutoBackup = async () => {
    if (suppressWatch) return;
    try {
      await invoke("update_auto_backup_settings", {
        enabled: autoBackupEnabled.value,
        time: autoBackupTime.value || "03:00",
        workspaceIds: autoBackupWorkspaceIds.value,
      });
    } catch (e) {
      console.error("自动保存备份设置失败:", e);
      showToast(`${t("settings.saveFailed")}: ${e}`, "error");
    }
  };

  // ===== 自动备份目标工作区多选 =====
  const isBackupWorkspaceSelected = (id) =>
    autoBackupWorkspaceIds.value.includes(id);
  const toggleBackupWorkspace = (id) => {
    const i = autoBackupWorkspaceIds.value.indexOf(id);
    if (i >= 0) autoBackupWorkspaceIds.value.splice(i, 1);
    else autoBackupWorkspaceIds.value.push(id);
  };
  const selectAllBackupWorkspaces = () => {
    autoBackupWorkspaceIds.value = workspaces.value.map((w) => w.id);
  };
  const clearBackupWorkspaces = () => {
    autoBackupWorkspaceIds.value = [];
  };
  const toggleBackupWsDropdown = () => {
    if (!autoBackupEnabled.value) return;
    showBackupWsDropdown.value = !showBackupWsDropdown.value;
  };
  // 下拉框触发器展示文案：选了几个就显示几个工作区名称
  const backupWsSummary = computed(() => {
    if (workspaces.value.length === 0) return t("settings.noWorkspaces");
    if (autoBackupWorkspaceIds.value.length === 0)
      return t("settings.autoBackupSelectPlaceholder");
    return workspaces.value
      .filter((w) => autoBackupWorkspaceIds.value.includes(w.id))
      .map((w) => w.name)
      .join("、");
  });

  // ===== 密码：失焦时单独保存（避免输入途中被防抖打断/清空） =====
  const savePassword = async () => {
    const pwd = password.value.trim();
    if (!pwd) return;
    try {
      const result = await invoke("update_git_backup_settings", {
        repoUrl: repoUrl.value.trim(),
        branch: branch.value.trim() || "master",
        username: username.value.trim(),
        password: pwd,
      });
      hasPassword.value = !!result.has_password;
      password.value = "";
      showToast(t("settings.saveSuccess"), "success");
    } catch (e) {
      console.error("保存凭据失败:", e);
      showToast(`${t("settings.saveFailed")}: ${e}`, "error");
    }
  };

  let mainSaveTimer = null;
  let gitSaveTimer = null;
  let autoBackupTimer = null;
  const scheduleMainSave = () => {
    if (suppressWatch) return;
    clearTimeout(mainSaveTimer);
    mainSaveTimer = setTimeout(autoSaveMain, 500);
  };
  const scheduleGitSave = () => {
    if (suppressWatch) return;
    clearTimeout(gitSaveTimer);
    gitSaveTimer = setTimeout(saveGitConfig, 500);
  };
  const scheduleAutoBackupSave = () => {
    if (suppressWatch) return;
    clearTimeout(autoBackupTimer);
    autoBackupTimer = setTimeout(saveAutoBackup, 500);
  };

  // ===== 统一保存（按当前分类分发） =====
  const save = async () => {
    try {
      loading.value = true;
      if (activeCategory.value === "git") {
        await saveGit();
      } else {
        await saveMain();
        applyAppearance();
      }
      showToast(t("settings.saveSuccess"), "success");
      await loadAll();
    } catch (e) {
      console.error("Failed to save settings:", e);
      showToast(`${t("settings.saveFailed")}: ${e}`, "error");
    } finally {
      loading.value = false;
    }
  };

  // ===== 外观实时应用到 DOM =====
  const applyAppearance = () => {
    setTheme(themeId.value);
    const root = document.documentElement;
    root.style.setProperty(
      "--font-mono",
      `"${monoFont.value}","SFMono-Regular",Consolas,Menlo,monospace`,
    );
    root.classList.toggle("no-anim", !animations.value);
  };

  // 外观即时切换（点选主题卡片时预览，不持久化）
  const previewTheme = (id) => {
    themeId.value = id;
    setTheme(id);
  };

  // ===== AI 模型 =====
  const getEnabledHeaders = () =>
    customHeaders.value
      .filter((h) => h.enabled && h.key.trim())
      .map((h) => ({
        key: h.key,
        value: h.value,
        enabled: h.enabled,
        description: h.description?.trim() || null,
      }));

  const fetchModels = async () => {
    const currentApiKey = aiApiKey.value.trim();
    const apiKeyToSend = currentApiKey.includes("****")
      ? null
      : currentApiKey || null;
    try {
      loadingModels.value = true;
      const models = await invoke("get_ai_models", {
        apiEndpoint: aiEndpoint.value,
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

  const selectModel = (model) => {
    aiModel.value = model;
    showModelDropdown.value = false;
  };
  const toggleModelDropdown = () => {
    const willOpen = !showModelDropdown.value;
    showModelDropdown.value = willOpen;
    if (willOpen) fetchModels();
  };
  const closeModelDropdown = () => {
    showModelDropdown.value = false;
  };
  const addHeader = () => {
    customHeaders.value.push({
      key: "",
      value: "",
      enabled: true,
      description: "",
    });
  };
  const removeHeader = (index) => {
    customHeaders.value.splice(index, 1);
  };

  // ===== Git 测试连接 =====
  const testGitConnection = async () => {
    if (!repoUrl.value.trim()) {
      showToast(t("gitBackup.urlRequired"), "warning");
      return;
    }
    try {
      gitTesting.value = true;
      await saveGit(); // 先静默保存
      await invoke("test_git_connection");
      showToast(t("gitBackup.connectionSuccess"), "success");
    } catch (e) {
      console.error(e);
      showToast(`${t("gitBackup.connectionFailed")}: ${e}`, "error");
    } finally {
      gitTesting.value = false;
    }
  };

  // ===== AI 测试连接 =====
  const testAiConnection = async () => {
    try {
      loading.value = true;
      await fetchModels();
    } finally {
      loading.value = false;
    }
  };

  const setCategory = (cat) => {
    activeCategory.value = cat;
  };
  const close = () => emit("close");

  // 点击外部关闭模型下拉
  const handleClickOutside = (event) => {
    const container = document.querySelector(".model-input-container");
    if (container && !container.contains(event.target)) {
      showModelDropdown.value = false;
    }
    const backupContainer = document.querySelector(".ab-ws-dd");
    if (backupContainer && !backupContainer.contains(event.target)) {
      showBackupWsDropdown.value = false;
    }
  };

  watch(
    [
      requestTimeout, language, restoreWorkspace, autoSaveOnSend,
      aiEndpoint, aiApiKey, aiModel, aiTimeout, customHeaders,
      themeId, monoFont, animations,
    ],
    scheduleMainSave,
    { deep: true },
  );
  watch([repoUrl, branch, username], scheduleGitSave);
  watch(
    [autoBackupEnabled, autoBackupTime, autoBackupWorkspaceIds],
    scheduleAutoBackupSave,
    { deep: true },
  );

  useDialogEscape(() => props.visible, close);
  watch(
    () => props.visible,
    (v) => {
      if (v) {
        loadAll();
        activeCategory.value = props.initialCategory || "general";
      }
    },
  );
  onMounted(() => {
    if (props.visible) loadAll();
    document.addEventListener("click", handleClickOutside);
  });
  onUnmounted(() => {
    document.removeEventListener("click", handleClickOutside);
  });

  return {
    t,
    activeCategory,
    loading,
    // 通用
    requestTimeout,
    language,
    // 行为
    restoreWorkspace,
    autoSaveOnSend,
    // AI
    aiEndpoint,
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
    testAiConnection,
    // Git
    repoUrl,
    branch,
    username,
    password,
    hasPassword,
    gitTesting,
    autoBackupEnabled,
    autoBackupTime,
    autoBackupWorkspaceIds,
    workspaces,
    showBackupWsDropdown,
    backupWsSummary,
    isBackupWorkspaceSelected,
    toggleBackupWorkspace,
    selectAllBackupWorkspaces,
    clearBackupWorkspaces,
    toggleBackupWsDropdown,
    testGitConnection,
    // 外观
    themeId,
    monoFont,
    animations,
    previewTheme,
    // 通用动作
    save,
    savePassword,
    setCategory,
    close,
  };
}
