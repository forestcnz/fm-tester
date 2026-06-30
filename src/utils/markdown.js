import { marked } from "marked";
import DOMPurify from "dompurify";

// 统一配置 marked，避免各组件分别 setOptions 造成全局污染
marked.setOptions({
  breaks: true,
  gfm: true,
});

// 允许常规 Markdown 渲染所需的标签/属性，剥离脚本/事件处理器等危险内容
const SANITIZE_CONFIG = {
  ALLOWED_URI_REGEXP:
    /^(?:(?:https?|mailto|ftp|tel|data|blob|vbscript|javascript|file):|[^a-z]|[a-z+.-]+(?:[^a-z+.\-:]|$))/i,
  FORBID_ATTR: ["onerror", "onload", "onclick", "onmouseover", "style"],
  FORBID_TAGS: ["script", "iframe", "object", "embed", "form"],
};

/**
 * 将 Markdown 内容渲染为已消毒的 HTML 字符串
 * @param {string} content - Markdown 原文
 * @returns {string} 安全的 HTML
 */
export function renderMarkdown(content) {
  if (!content) return "";
  try {
    const rawHtml = marked.parse(content);
    return DOMPurify.sanitize(rawHtml, SANITIZE_CONFIG);
  } catch (e) {
    console.error("Markdown 渲染失败:", e);
    return DOMPurify.sanitize(String(content), SANITIZE_CONFIG);
  }
}

export { marked };
