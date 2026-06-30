import { watch } from "vue";
import * as monaco from "monaco-editor";

/**
 * Monaco Editor 主题定义
 * 为每个应用主题定义对应的 Monaco Editor 主题配色
 */

// 主题配色定义
const MONACO_THEMES = {
  // VS Code Dark+ 风格
  "vscode-dark": {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "comment", foreground: "6A9955" },
      { token: "keyword", foreground: "569cd6" },
      { token: "string", foreground: "ce9178" },
      { token: "number", foreground: "b5cea8" },
      { token: "regexp", foreground: "d16969" },
      { token: "type", foreground: "4ec9b0" },
      { token: "class", foreground: "4ec9b0" },
      { token: "function", foreground: "dcdcaa" },
      { token: "variable", foreground: "9cdcfe" },
      { token: "variable.predefined", foreground: "4fc1ff" },
      { token: "constant", foreground: "4fc1ff" },
      { token: "tag", foreground: "569cd6" },
      { token: "attribute.name", foreground: "9cdcfe" },
      { token: "attribute.value", foreground: "ce9178" },
    ],
    colors: {
      "editor.background": "#1e1e1e",
      "editor.foreground": "#d4d4d4",
      "editorLineNumber.foreground": "#858585",
      "editorLineNumber.activeForeground": "#c6c6c6",
      "editor.selectionBackground": "#264f78",
      "editor.lineHighlightBackground": "#2a2d2e",
      "editorCursor.foreground": "#aeafad",
      "editorIndentGuide.background": "#404040",
      "editorIndentGuide.activeBackground": "#707070",
    },
  },

  // Nord 风格
  nord: {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "comment", foreground: "616e88", fontStyle: "italic" },
      { token: "keyword", foreground: "81a1c1" },
      { token: "string", foreground: "a3be8c" },
      { token: "number", foreground: "b48ead" },
      { token: "regexp", foreground: "ebcb8b" },
      { token: "type", foreground: "8fbcbb" },
      { token: "class", foreground: "8fbcbb" },
      { token: "function", foreground: "88c0d0" },
      { token: "variable", foreground: "d8dee9" },
      { token: "variable.predefined", foreground: "81a1c1" },
      { token: "constant", foreground: "81a1c1" },
      { token: "tag", foreground: "81a1c1" },
      { token: "attribute.name", foreground: "8fbcbb" },
      { token: "attribute.value", foreground: "a3be8c" },
    ],
    colors: {
      "editor.background": "#2e3440",
      "editor.foreground": "#d8dee9",
      "editorLineNumber.foreground": "#4c566a",
      "editorLineNumber.activeForeground": "#eceff4",
      "editor.selectionBackground": "#434c5e",
      "editor.lineHighlightBackground": "#3b4252",
      "editorCursor.foreground": "#d8dee9",
      "editorIndentGuide.background": "#4c566a",
      "editorIndentGuide.activeBackground": "#616e88",
    },
  },

  // Dracula 风格
  dracula: {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "comment", foreground: "6272a4", fontStyle: "italic" },
      { token: "keyword", foreground: "ff79c6" },
      { token: "string", foreground: "f1fa8c" },
      { token: "number", foreground: "bd93f9" },
      { token: "regexp", foreground: "f1fa8c" },
      { token: "type", foreground: "8be9fd" },
      { token: "class", foreground: "8be9fd" },
      { token: "function", foreground: "50fa7b" },
      { token: "variable", foreground: "f8f8f2" },
      { token: "variable.predefined", foreground: "ffb86c" },
      { token: "constant", foreground: "ffb86c" },
      { token: "tag", foreground: "ff79c6" },
      { token: "attribute.name", foreground: "50fa7b" },
      { token: "attribute.value", foreground: "f1fa8c" },
    ],
    colors: {
      "editor.background": "#282a36",
      "editor.foreground": "#f8f8f2",
      "editorLineNumber.foreground": "6272a4",
      "editorLineNumber.activeForeground": "f8f8f2",
      "editor.selectionBackground": "#44475a",
      "editor.lineHighlightBackground": "#44475a",
      "editorCursor.foreground": "#f8f8f2",
      "editorIndentGuide.background": "#6272a4",
      "editorIndentGuide.activeBackground": "bd93f9",
    },
  },

  // One Dark Pro 风格
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

  // Material Darker 风格
  "material-darker": {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "comment", foreground: "545454", fontStyle: "italic" },
      { token: "keyword", foreground: "c792ea" },
      { token: "string", foreground: "c3e88d" },
      { token: "number", foreground: "f78c6c" },
      { token: "regexp", foreground: "89ddff" },
      { token: "type", foreground: "ffcb6b" },
      { token: "class", foreground: "ffcb6b" },
      { token: "function", foreground: "82aaff" },
      { token: "variable", foreground: "eeffff" },
      { token: "variable.predefined", foreground: "89ddff" },
      { token: "constant", foreground: "f78c6c" },
      { token: "tag", foreground: "f07178" },
      { token: "attribute.name", foreground: "ffcb6b" },
      { token: "attribute.value", foreground: "c3e88d" },
    ],
    colors: {
      "editor.background": "#121212",
      "editor.foreground": "#eeffff",
      "editorLineNumber.foreground": "424242",
      "editorLineNumber.activeForeground": "eeffff",
      "editor.selectionBackground": "#2a2a2a",
      "editor.lineHighlightBackground": "#1e1e1e",
      "editorCursor.foreground": "eeffff",
      "editorIndentGuide.background": "2a2a2a",
      "editorIndentGuide.activeBackground": "424242",
    },
  },

  // Monokai Pro 风格
  "monokai-pro": {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "comment", foreground: "727072", fontStyle: "italic" },
      { token: "keyword", foreground: "ff6188" },
      { token: "string", foreground: "ffd866" },
      { token: "number", foreground: "ab9df2" },
      { token: "regexp", foreground: "ffd866" },
      { token: "type", foreground: "78dce8" },
      { token: "class", foreground: "78dce8" },
      { token: "function", foreground: "a9dc76" },
      { token: "variable", foreground: "fcfcfa" },
      { token: "variable.predefined", foreground: "fc9867" },
      { token: "constant", foreground: "ab9df2" },
      { token: "tag", foreground: "ff6188" },
      { token: "attribute.name", foreground: "78dce8" },
      { token: "attribute.value", foreground: "ffd866" },
    ],
    colors: {
      "editor.background": "#2d2a2e",
      "editor.foreground": "#fcfcfa",
      "editorLineNumber.foreground": "5b5956",
      "editorLineNumber.activeForeground": "fcfcfa",
      "editor.selectionBackground": "#403e3a",
      "editor.lineHighlightBackground": "#36332f",
      "editorCursor.foreground": "fcfcfa",
      "editorIndentGuide.background": "5b5956",
      "editorIndentGuide.activeBackground": "727072",
    },
  },

  // GitHub Dark 风格
  "github-dark": {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "comment", foreground: "8b949e" },
      { token: "keyword", foreground: "ff7b72" },
      { token: "string", foreground: "a5d6ff" },
      { token: "number", foreground: "79c0ff" },
      { token: "regexp", foreground: "7ee787" },
      { token: "type", foreground: "ffa657" },
      { token: "class", foreground: "ffa657" },
      { token: "function", foreground: "d2a8ff" },
      { token: "variable", foreground: "c9d1d9" },
      { token: "variable.predefined", foreground: "79c0ff" },
      { token: "constant", foreground: "79c0ff" },
      { token: "tag", foreground: "7ee787" },
      { token: "attribute.name", foreground: "79c0ff" },
      { token: "attribute.value", foreground: "a5d6ff" },
    ],
    colors: {
      "editor.background": "#0d1117",
      "editor.foreground": "#c9d1d9",
      "editorLineNumber.foreground": "6e7681",
      "editorLineNumber.activeForeground": "c9d1d9",
      "editor.selectionBackground": "#264f78",
      "editor.lineHighlightBackground": "#161b22",
      "editorCursor.foreground": "c9d1d9",
      "editorIndentGuide.background": "21262d",
      "editorIndentGuide.activeBackground": "6e7681",
    },
  },

  // Tokyo Night 风格
  "tokyo-night": {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "comment", foreground: "565f89", fontStyle: "italic" },
      { token: "keyword", foreground: "bb9af7" },
      { token: "string", foreground: "9ece6a" },
      { token: "number", foreground: "ff9e64" },
      { token: "regexp", foreground: "f7768e" },
      { token: "type", foreground: "7aa2f7" },
      { token: "class", foreground: "7aa2f7" },
      { token: "function", foreground: "7dcfff" },
      { token: "variable", foreground: "c0caf5" },
      { token: "variable.predefined", foreground: "e0af68" },
      { token: "constant", foreground: "ff9e64" },
      { token: "tag", foreground: "f7768e" },
      { token: "attribute.name", foreground: "7dcfff" },
      { token: "attribute.value", foreground: "9ece6a" },
    ],
    colors: {
      "editor.background": "#1a1b26",
      "editor.foreground": "#c0caf5",
      "editorLineNumber.foreground": "3b4261",
      "editorLineNumber.activeForeground": "c0caf5",
      "editor.selectionBackground": "#364a82",
      "editor.lineHighlightBackground": "#16161e",
      "editorCursor.foreground": "c0caf5",
      "editorIndentGuide.background": "3b4261",
      "editorIndentGuide.activeBackground": "565f89",
    },
  },

  // Arctic Ice 浅色风格
  "arctic-ice": {
    base: "vs",
    inherit: true,
    rules: [
      { token: "comment", foreground: "94a3b8", fontStyle: "italic" },
      { token: "keyword", foreground: "7c3aed" },
      { token: "string", foreground: "059669" },
      { token: "number", foreground: "ea580c" },
      { token: "regexp", foreground: "dc2626" },
      { token: "type", foreground: "0369a1" },
      { token: "class", foreground: "0369a1" },
      { token: "function", foreground: "2563eb" },
      { token: "variable", foreground: "1e293b" },
      { token: "variable.predefined", foreground: "0369a1" },
      { token: "constant", foreground: "ea580c" },
      { token: "tag", foreground: "dc2626" },
      { token: "attribute.name", foreground: "2563eb" },
      { token: "attribute.value", foreground: "059669" },
    ],
    colors: {
      "editor.background": "#ffffff",
      "editor.foreground": "#1e293b",
      "editorLineNumber.foreground": "cbd5e1",
      "editorLineNumber.activeForeground": "475569",
      "editor.selectionBackground": "#e0f2fe",
      "editor.lineHighlightBackground": "#f8fafc",
      "editorCursor.foreground": "1e293b",
      "editorIndentGuide.background": "e2e8f0",
      "editorIndentGuide.activeBackground": "94a3b8",
    },
  },

  // Storm Gray 浅色风格
  "storm-gray": {
    base: "vs",
    inherit: true,
    rules: [
      { token: "comment", foreground: "90a4ae", fontStyle: "italic" },
      { token: "keyword", foreground: "546e7a" },
      { token: "string", foreground: "4caf50" },
      { token: "number", foreground: "f57c00" },
      { token: "regexp", foreground: "d32f2f" },
      { token: "type", foreground: "1976d2" },
      { token: "class", foreground: "1976d2" },
      { token: "function", foreground: "00796b" },
      { token: "variable", foreground: "263238" },
      { token: "variable.predefined", foreground: "1976d2" },
      { token: "constant", foreground: "f57c00" },
      { token: "tag", foreground: "d32f2f" },
      { token: "attribute.name", foreground: "00796b" },
      { token: "attribute.value", foreground: "4caf50" },
    ],
    colors: {
      "editor.background": "#ffffff",
      "editor.foreground": "#263238",
      "editorLineNumber.foreground": "b0bec5",
      "editorLineNumber.activeForeground": "546e7a",
      "editor.selectionBackground": "#eceff1",
      "editor.lineHighlightBackground": "#f5f7f9",
      "editorCursor.foreground": "263238",
      "editorIndentGuide.background": "cfd8dc",
      "editorIndentGuide.activeBackground": "90a4ae",
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
    const darkThemes = [
      "dark",
      "vscode-dark",
      "nord",
      "dracula",
      "one-dark",
      "material-darker",
      "monokai-pro",
      "github-dark",
      "tokyo-night",
    ];

    return darkThemes.includes(appTheme) ? "vs-dark" : "vs";
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
