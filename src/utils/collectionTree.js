/**
 * 集合树相关公共工具函数
 *
 * 此前 useTabs / useRequest / useOrchestrationExecution / OrchestrationEditor /
 * StressTestPanel 等多处各自实现了 findAncestorCollectionsForApi、buildUrlWithParams、
 * mergeCollectionVariables 等工具，逻辑相同，统一抽取到此处。
 */

/**
 * 从 params 构建带参数的 URL
 * @param {string} baseUrl - 基础 URL
 * @param {Array} params - 参数数组 [{key, value, enabled}]
 * @returns {string} 带查询参数的 URL
 */
export function buildUrlWithParams(baseUrl, params) {
  if (!baseUrl) return "";

  const enabledParams = params?.filter((p) => p.enabled && p.key) || [];
  if (enabledParams.length === 0) {
    // 移除 URL 中的查询参数
    const queryIndex = baseUrl.indexOf("?");
    return queryIndex < 0 ? baseUrl : baseUrl.slice(0, queryIndex);
  }

  const queryStr = enabledParams
    .map(
      (p) =>
        `${encodeURIComponent(p.key)}=${encodeURIComponent(p.value || "")}`,
    )
    .join("&");

  // 移除原有查询参数
  const queryIndex = baseUrl.indexOf("?");
  const cleanUrl = queryIndex < 0 ? baseUrl : baseUrl.slice(0, queryIndex);

  return `${cleanUrl}?${queryStr}`;
}

/**
 * 查找 API 所属的所有祖先集合（从根到直接父集合）
 * @param {Array} collections - 集合树
 * @param {string} apiId - 目标 API ID
 * @returns {Array} 祖先集合数组（外层在前，父集合在后；未找到返回空数组）
 */
export function findAncestorCollectionsForApi(collections, apiId) {
  const search = (items, path = []) => {
    for (const item of items) {
      if (item.type === "api" && item.id === apiId) {
        return path;
      }
      if (item.type === "collection" && item.children) {
        const newPath = [...path, item];
        const found = search(item.children, newPath);
        if (found) return found;
      }
    }
    return null;
  };
  return search(collections || []) || [];
}

/**
 * 在集合树中递归查找指定 ID 的 API
 * @param {Array} collections - 集合树
 * @param {string} apiId - 目标 API ID
 * @returns {Object|null} 找到的 API 对象
 */
export function findApiInCollections(collections, apiId) {
  const flatten = (items, result = []) => {
    for (const item of items || []) {
      if (item.type === "api" && item.id === apiId) {
        result.push(item);
      }
      if (item.type === "collection" && item.children) {
        flatten(item.children, result);
      }
    }
    return result;
  };
  return flatten(collections)[0] || null;
}

/**
 * 合并所有祖先集合的变量（数组形式，子覆盖父）
 * @param {Array} ancestorCollections - 祖先集合数组
 * @returns {Array} 合并后的变量数组 [{key, value, ...}]
 */
export function mergeCollectionVariables(ancestorCollections) {
  const result = [];
  for (const collection of ancestorCollections || []) {
    if (collection.collection_variables) {
      for (const v of collection.collection_variables) {
        const idx = result.findIndex((cv) => cv.key === v.key);
        if (idx >= 0) {
          result[idx] = v;
        } else {
          result.push(v);
        }
      }
    }
  }
  return result;
}
