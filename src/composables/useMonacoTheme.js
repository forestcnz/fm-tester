import { watch } from "vue";
import * as monaco from "monaco-editor";

/**
 * Monaco Editor 主题定义
 * 为每个应用主题定义对应的 Monaco Editor 主题配色
 */

// 主题配色定义
const MONACO_THEMES = {
  // One Dark Pro 风格（dark 主题复用此配置）
  "one-dark": {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "comment", foreground: "5c6370", fontStyle: "italic" },
      { token: "keyword", foreground: "c678dd" },
      { token: "string", foreground: "98c379" },
      { token: "number", foreground: "d19a66" },
      { token: "regexp", foreground: "98c379" },
      { token: "type", foreground: "e5c07b" },
      { token: "class", foreground: "e5c07b" },
      { token: "function", foreground: "61afef" },
      { token: "variable", foreground: "e06c75" },
      { token: "variable.predefined", foreground: "e5c07b" },
      { token: "constant", foreground: "d19a66" },
      { token: "tag", foreground: "e06c75" },
      { token: "attribute.name", foreground: "d19a66" },
      { token: "attribute.value", foreground: "98c379" },
    ],
    colors: {
      "editor.background": "#282c34",
      "editor.foreground": "#abb2bf",
      "editorLineNumber.foreground": "636d83",
      "editorLineNumber.activeForeground": "abb2bf",
      "editor.selectionBackground": "#3e4451",
      "editor.lineHighlightBackground": "#2c313a",
      "editorCursor.foreground": "#528bff",
      "editorIndentGuide.background": "#3b4048",
      "editorIndentGuide.activeBackground": "#c8c8c8",
    },
  },
};

// 墨砚深色主题（id: dark）复用 One Dark Pro 配色
MONACO_THEMES["dark"] = MONACO_THEMES["one-dark"];

// 标记主题是否已注册
let themesRegistered = false;

/**
 * 注册所有 Monaco Editor 自定义主题
 */
function registerMonacoThemes() {
  if (themesRegistered) return;

  for (const [themeId, themeConfig] of Object.entries(MONACO_THEMES)) {
    monaco.editor.defineTheme(themeId, themeConfig);
  }
  themesRegistered = true;
}

/**
 * Monaco Editor 主题管理
 * 根据应用主题自动切换 Monaco Editor 主题
 */
export function useMonacoTheme(_currentTheme) {
  /**
   * 获取 Monaco 主题名称
   */
  const getMonacoThemeName = (appTheme) => {
    // 如果主题已定义，使用自定义主题
    if (MONACO_THEMES[appTheme]) {
      return appTheme;
    }

    // 降级处理：深色主题使用 vs-dark，浅色使用 vs
    return appTheme === "dark" ? "vs-dark" : "vs";
  };

  /**
   * 设置 Monaco Editor 主题
   */
  const setMonacoTheme = (appTheme) => {
    // 确保主题已注册
    registerMonacoThemes();

    const monacoTheme = getMonacoThemeName(appTheme);
    monaco.editor.setTheme(monacoTheme);
  };

  /**
   * 获取 Monaco Editor 创建时的主题选项
   */
  const getMonacoThemeOption = (appTheme) => {
    // 确保主题已注册
    registerMonacoThemes();

    return getMonacoThemeName(appTheme);
  };

  /**
   * 监听主题变化并更新 Monaco 主题
   */
  const watchThemeChange = (themeRef, editors) => {
    watch(themeRef, (newTheme) => {
      // 确保主题已注册
      registerMonacoThemes();

      const monacoTheme = getMonacoThemeName(newTheme);

      // 更新所有已创建的编辑器实例
      if (editors && editors.length > 0) {
        editors.forEach((editor) => {
          if (editor) {
            editor.updateOptions({ theme: monacoTheme });
          }
        });
      }

      // 也设置全局主题
      monaco.editor.setTheme(monacoTheme);
    });
  };

  return {
    getMonacoThemeName,
    setMonacoTheme,
    getMonacoThemeOption,
    watchThemeChange,
    registerMonacoThemes,
  };
}

// 导出主题配置供外部使用
export { MONACO_THEMES };
