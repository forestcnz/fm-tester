// ============================================================================
// ESLint 配置（Flat Config，适用于 ESLint 9+）
//
// 设计目标：以「能落地、不误伤」为优先，给纯 JavaScript + Vue 3 SFC 项目提供
// 一道静态检查防线。规则取舍说明如下：
//   - 采用 flat/essential（Vue 关键错误规则），不强制 Vue 格式类规则，避免与
//     既有大量代码风格冲突导致 lint 全面飘红；
//   - no-console 放行：项目惯例使用 console.* 做运行时日志；
//   - no-undef 放行：纯 JS 无类型声明文件，浏览器全局（document/window 等）会
//     被误报，引入 Vitest/TS 后再收紧；
//   - vue/no-v-html 设为 warn：项目多处用 v-html 渲染经 DOMPurify 净化的
//     markdown，作为风险提示渐进收紧。
// ============================================================================

import js from "@eslint/js";
import pluginVue from "eslint-plugin-vue";

export default [
  // 全局忽略：构建产物、依赖、后端、静态资源
  {
    ignores: ["dist/**", "node_modules/**", "src-tauri/**", "public/**"],
  },

  // JavaScript 推荐规则
  js.configs.recommended,

  // Vue 3 关键错误规则（essential，非格式类）
  ...pluginVue.configs["flat/essential"],

  // 项目自定义规则
  {
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
    },
    rules: {
      "no-console": "off",
      "no-undef": "off",
      // 存量空 catch 块（错误被静默吞掉）待逐步补充日志，先降为 warn 让 CI 通过
      "no-empty": "warn",
      // 以下两项为代码质量类（正则冗余转义、无效赋值），存量较多，先降为 warn 渐进清理
      "no-useless-escape": "warn",
      "no-useless-assignment": "warn",
      "no-unused-vars": ["warn", { argsIgnorePattern: "^_" }],
      "vue/multi-word-component-names": "off", // 允许 Icon 等单字组件名
      "vue/no-v-html": "warn", // v-html 风险点：渐进收紧
      "vue/attributes-order": "off",
      "vue/max-attributes-per-line": "off",
    },
  },
];
