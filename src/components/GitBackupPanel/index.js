import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { useDialogEscape } from "../../composables/useDialogStack.js";
import { showToast } from "../../composables/useToast.js";

export function useGitBackupSetup(props, emit) {
  const { t } = useI18n();
  const loading = ref(false);
  const testing = ref(false);

  const repoUrl = ref("");
  const branch = ref("master");
  const username = ref("");
  const password = ref("");
  const hasPassword = ref(false);

  const passwordPlaceholder = computed(() =>
    hasPassword.value
      ? t("gitBackup.passwordKeepPlaceholder")
      : t("git.passwordPlaceholder"),
  );

  const loadSettings = async () => {
    try {
      loading.value = true;
      const s = await invoke("get_git_backup_settings");
      repoUrl.value = s.repo_url || "";
      branch.value = s.branch || "master";
      username.value = s.username || "";
      hasPassword.value = !!s.has_password;
      password.value = "";
    } catch (e) {
      console.error("加载 Git 备份配置失败:", e);
      showToast(t("gitBackup.loadFailed"), "error");
    } finally {
      loading.value = false;
    }
  };

  // 持久化配置（密码三态：空=null保持、非空=更新）
  const persist = async (showMsg) => {
    const pwd = password.value.trim();
    const result = await invoke("update_git_backup_settings", {
      repoUrl: repoUrl.value.trim(),
      branch: branch.value.trim() || "main",
      username: username.value.trim(),
      password: pwd ? pwd : null,
    });
    hasPassword.value = !!result.has_password;
    password.value = "";
    if (showMsg) showToast(t("gitBackup.saveSuccess"), "success");
    return result;
  };

  const saveSettings = async () => {
    try {
      loading.value = true;
      await persist(true);
      close();
    } catch (e) {
      console.error(e);
      showToast(`${t("gitBackup.saveFailed")}: ${e}`, "error");
    } finally {
      loading.value = false;
    }
  };

  // 测试连接：先静默保存当前配置，再用后端配置测试
  const testConnection = async () => {
    if (!repoUrl.value.trim()) {
      showToast(t("gitBackup.urlRequired"), "warning");
      return;
    }
    try {
      testing.value = true;
      await persist(false);
      await invoke("test_git_connection");
      showToast(t("gitBackup.connectionSuccess"), "success");
    } catch (e) {
      console.error(e);
      showToast(`${t("gitBackup.connectionFailed")}: ${e}`, "error");
    } finally {
      testing.value = false;
    }
  };

  const close = () => emit("close");

  useDialogEscape(() => props.visible, close);
  watch(
    () => props.visible,
    (v) => {
      if (v) loadSettings();
    },
  );

  return {
    t,
    loading,
    testing,
    repoUrl,
    branch,
    username,
    password,
    hasPassword,
    passwordPlaceholder,
    testConnection,
    saveSettings,
    close,
  };
}
