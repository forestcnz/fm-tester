import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { useDialogEscape } from "../../composables/useDialogStack.js";
import { useTheme, THEMES } from "../../composables/useTheme";

export function useSettingsSetup(props, emit) {
  const { t } = useI18n();
  const timeout = ref(60);
  const loading = ref(false);

  // 主题相关
  const {
    currentTheme,
    setTheme,
    getDarkThemes,
    getLightThemes,
    getCurrentThemeInfo,
  } = useTheme();
  const darkThemes = getDarkThemes();
  const lightThemes = getLightThemes();

  // 加载设置
  const loadSettings = async () => {
    try {
      loading.value = true;
      const settings = await invoke("get_settings");
      timeout.value = settings.request_timeout;
    } catch (e) {
      console.error("Failed to load settings:", e);
    } finally {
      loading.value = false;
    }
  };

  // 保存设置
  const saveSettings = async () => {
    try {
      loading.value = true;
      const settings = await invoke("update_settings", {
        timeout: timeout.value,
      });
      timeout.value = settings.request_timeout;

      emit("saved");
      emit("close");
    } catch (e) {
      console.error("Failed to save settings:", e);
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

  onMounted(() => {
    loadSettings();
  });

  return {
    t,
    timeout,
    loading,
    saveSettings,
    close,
    // 主题相关
    currentTheme,
    setTheme,
    THEMES,
    darkThemes,
    lightThemes,
    getCurrentThemeInfo,
  };
}
