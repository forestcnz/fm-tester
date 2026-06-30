import { ref, watch, onMounted } from "vue";
import { useMonacoTheme, MONACO_THEMES } from "./useMonacoTheme";

/**
 * 主题定义
 */
export const THEMES = [
  {
    id: "dark",
    name: "Dark",
    label: "Dark",
    description: "One Dark Pro 风格 - 柔和舒适的墨砚深色",
    type: "dark",
  },
  {
    id: "paper",
    name: "Paper",
    label: "Paper",
    description: "纸张米白底 - 墨蓝点缀的阅读感",
    type: "light",
  },
];

/**
 * 默认主题
 */
const DEFAULT_THEME = "paper";

/**
 * 本地存储键名
 */
const THEME_STORAGE_KEY = "fm-tester-theme";

/**
 * 获取 Monaco Editor 主题名称
 * 使用自定义的 Monaco Editor 主题
 */
export const getMonacoTheme = (appThemeId) => {
  // 如果有自定义主题定义，使用自定义主题
  if (MONACO_THEMES[appThemeId]) {
    return appThemeId;
  }

  // 降级处理
  const darkThemes = ["dark", "one-dark"];

  return darkThemes.includes(appThemeId) ? "vs-dark" : "vs";
};

/**
 * 主题管理 composable
 */
export function useTheme() {
  const currentTheme = ref(DEFAULT_THEME);

  /**
   * 获取主题 CSS 类名
   */
  const getThemeClass = (themeId) => {
    return `theme-${themeId}`;
  };

  /**
   * 应用主题到 DOM
   */
  const applyTheme = (themeId) => {
    // 移除所有主题类（含历史遗留 theme-one-dark，确保切换干净）
    const themeClasses = [
      ...THEMES.map((t) => getThemeClass(t.id)),
      "theme-one-dark",
    ];
    document.body.classList.remove(...themeClasses);

    // 添加新主题类
    const newClass = getThemeClass(themeId);
    document.body.classList.add(newClass);

    // 更新当前主题
    currentTheme.value = themeId;

    // 注册并设置 Monaco Editor 主题
    const { registerMonacoThemes, setMonacoTheme } =
      useMonacoTheme(currentTheme);
    registerMonacoThemes();
    setMonacoTheme(themeId);

    // 触发自定义事件，通知 Monaco Editor 更新主题
    window.dispatchEvent(
      new CustomEvent("fm-theme-change", {
        detail: { themeId, monacoTheme: getMonacoTheme(themeId) },
      }),
    );
  };

  /**
   * 切换主题
   */
  const setTheme = (themeId) => {
    // 验证主题是否存在
    const theme = THEMES.find((t) => t.id === themeId);
    if (!theme) {
      console.warn(`Theme "${themeId}" not found, using default`);
      themeId = DEFAULT_THEME;
    }

    applyTheme(themeId);
    saveTheme(themeId);
  };

  /**
   * 保存主题到本地存储
   */
  const saveTheme = (themeId) => {
    localStorage.setItem(THEME_STORAGE_KEY, themeId);
  };

  /**
   * 从本地存储加载主题
   */
  const loadTheme = () => {
    let savedTheme = localStorage.getItem(THEME_STORAGE_KEY);
    // 迁移历史主题 id：one-dark → dark
    if (savedTheme === "one-dark") {
      savedTheme = "dark";
      localStorage.setItem(THEME_STORAGE_KEY, "dark");
    }
    if (savedTheme && THEMES.find((t) => t.id === savedTheme)) {
      return savedTheme;
    }
    return DEFAULT_THEME;
  };

  /**
   * 获取当前主题信息
   */
  const getCurrentThemeInfo = () => {
    return THEMES.find((t) => t.id === currentTheme.value);
  };

  /**
   * 获取所有深色主题
   */
  const getDarkThemes = () => {
    return THEMES.filter((t) => t.type === "dark");
  };

  /**
   * 获取所有浅色主题
   */
  const getLightThemes = () => {
    return THEMES.filter((t) => t.type === "light");
  };

  /**
   * 判断当前是否为深色主题
   */
  const isDarkTheme = () => {
    const theme = getCurrentThemeInfo();
    return theme?.type === "dark";
  };

  /**
   * 初始化主题
   */
  const initTheme = () => {
    const themeId = loadTheme();
    applyTheme(themeId);
  };

  // 监听主题变化
  watch(currentTheme, (newTheme) => {
    applyTheme(newTheme);
  });

  // 组件挂载时初始化主题
  onMounted(() => {
    initTheme();
  });

  return {
    currentTheme,
    THEMES,
    setTheme,
    getCurrentThemeInfo,
    getDarkThemes,
    getLightThemes,
    isDarkTheme,
    initTheme,
  };
}
