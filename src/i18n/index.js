import { createI18n } from "vue-i18n";
import zhCN from "../locales/zh-CN.json";
import en from "../locales/en.json";

export const SUPPORTED_LOCALES = [
  { code: "zh-CN", name: "简体中文" },
  { code: "en", name: "English" },
];

const i18n = createI18n({
  legacy: false, // Composition API 模式
  locale: "zh-CN", // 默认中文
  fallbackLocale: "zh-CN",
  messages: {
    "zh-CN": zhCN,
    en: en,
  },
  missingWarn: false,
  fallbackWarn: false,
});

export default i18n;

// 语言切换函数
export function setLocale(locale) {
  const composer = i18n.global;
  composer.locale.value = locale;
  document.documentElement.setAttribute("lang", locale);
}

// 获取当前语言
export function getLocale() {
  return i18n.global.locale.value;
}
