import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { setLocale, SUPPORTED_LOCALES } from "../i18n";

/**
 * 国际化设置 composable
 * 提供语言切换和持久化功能
 */
export function useI18nSetup() {
  const { t, locale } = useI18n();
  const loading = ref(false);

  // 从后端加载语言设置
  const loadLanguage = async () => {
    try {
      loading.value = true;
      const settings = await invoke("get_settings");
      if (settings.language && settings.language !== locale.value) {
        setLocale(settings.language);
      }
    } catch (e) {
      console.error("Failed to load language settings:", e);
    } finally {
      loading.value = false;
    }
  };

  // 切换语言并持久化
  const switchLanguage = async (newLocale) => {
    try {
      loading.value = true;
      setLocale(newLocale);

      // 获取当前设置，保持其他配置不变
      const settings = await invoke("get_settings");
      await invoke("update_settings", {
        timeout: settings.request_timeout,
        language: newLocale,
      });
    } catch (e) {
      console.error("Failed to save language settings:", e);
    } finally {
      loading.value = false;
    }
  };

  // 组件挂载时自动加载语言
  onMounted(loadLanguage);

  return {
    t,
    locale,
    loading,
    supportedLocales: SUPPORTED_LOCALES,
    loadLanguage,
    switchLanguage,
  };
}
