import { ref } from "vue";

/**
 * 设置面板 composable
 * 统一设置中心（合并原 SettingsPanel / AISettingsPanel / GitBackupPanel）
 */
export function useSettings() {
  const showSettingsPanel = ref(false);
  const settingsCategory = ref("general");

  const openSettings = (category) => {
    settingsCategory.value = category || "general";
    showSettingsPanel.value = true;
  };

  const closeSettings = () => {
    showSettingsPanel.value = false;
  };

  // 分类快捷入口（兼容旧调用）
  const openAiSettings = () => openSettings("ai");
  const closeAiSettings = closeSettings;
  const openGitBackup = () => openSettings("git");
  const closeGitBackup = closeSettings;

  // 兼容旧 ref（旧三弹层已并入设置中心，保留以避免外部解构报错）
  const showAiSettingsPanel = ref(false);
  const showGitBackupPanel = ref(false);

  return {
    showSettingsPanel,
    settingsCategory,
    openSettings,
    closeSettings,
    showAiSettingsPanel,
    openAiSettings,
    closeAiSettings,
    showGitBackupPanel,
    openGitBackup,
    closeGitBackup,
  };
}
