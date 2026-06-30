/**
 * 脚本引擎工具函数
 *
 * 仅保留 URL 处理相关的工具函数，脚本执行已迁移到后端。
 */

/**
 * 解析 URL 获取 baseUrl（不含路径）
 * @param {string} url - 完整 URL
 * @returns {string} baseUrl
 */
export function extractBaseUrl(url) {
  if (!url) return "";
  try {
    // 处理带变量的 URL，先尝试匹配
    const match = url.match(/^https?:\/\/[^/]+/);
    if (match) return match[0];

    // 尝试解析为 URL
    const urlObj = new URL(url);
    return urlObj.origin;
  } catch {
    // URL 不完整（纯路径如 /api/users），没有 baseUrl
    if (url.startsWith("/")) {
      return "";
    }
    // 尝试手动解析
    const idx = url.indexOf("/");
    if (idx > 0 && url[idx + 1] !== "/") {
      // 找到第一个单斜杠（路径开始）
      return url.slice(0, idx);
    }
    // 没有 path，整个 URL 就是 baseUrl
    return url;
  }
}

/**
 * 解析 URL 获取路径部分（不含 baseUrl）
 * @param {string} url - 完整 URL
 * @returns {string} path
 */
export function extractPath(url) {
  if (!url) return "";
  try {
    const baseUrl = extractBaseUrl(url);
    if (!baseUrl) return url;
    return url.slice(baseUrl.length) || "/";
  } catch {
    return "";
  }
}

/**
 * 构建请求 URL（从 baseUrl 和 path）
 * @param {string} baseUrl
 * @param {string} path
 * @returns {string} 完整 URL
 */
export function buildUrl(baseUrl, path) {
  if (!baseUrl) return path || "";
  if (!path) return baseUrl;
  // 确保 path 以 / 开头
  if (!path.startsWith("/")) path = "/" + path;
  return baseUrl + path;
}

/**
 * 合并集合变量为对象
 * @param {Array} ancestorCollections - 祖先集合数组（按层级顺序）
 * @returns {Object} 变量对象
 */
export function mergeCollectionVariablesToObject(ancestorCollections) {
  const variables = {};

  for (const collection of ancestorCollections || []) {
    if (collection.collection_variables) {
      for (const v of collection.collection_variables) {
        if (v.enabled) {
          variables[v.key] = v.value;
        }
      }
    }
  }

  return variables;
}
