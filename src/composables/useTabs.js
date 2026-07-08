import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  findAncestorCollectionsForApi,
  mergeCollectionVariables,
} from "../utils/collectionTree.js";

/**
 * 标签页管理 composable
 * @param {Ref} currentWorkspace - 当前工作区引用
 * @param {Ref} currentNavKey - 当前导航项引用
 * @param {Ref} sidebarRef - 侧边栏组件引用
 * @param {Object} currentRequest - 当前请求状态（响应式对象）
 * @param {Ref} response - 响应数据引用
 * @param {Ref} loading - 加载状态引用
 * @param {Ref} requestTabs - 请求子标签页状态引用
 * @param {Ref} tabs - 标签页列表引用（外部传入）
 * @param {Ref} activeTab - 当前激活标签页引用（外部传入）
 * @param {Ref} currentRequestTab - 当前请求子标签页引用（外部传入）
 */
export function useTabs(
  currentWorkspace,
  currentNavKey,
  sidebarRef,
  currentRequest,
  response,
  loading,
  requestTabs,
  tabs,
  activeTab,
  currentRequestTab,
) {
  const collectionTabsData = ref({});

  const displayTabs = computed(() => {
    if (currentNavKey.value !== "collection") return [];
    return tabs.value;
  });

  const selectedCollection = computed(() => {
    const currentTab = tabs.value[activeTab.value];
    if (currentTab?.tabType === "collection") {
      return collectionTabsData.value[currentTab.id];
    }
    return null;
  });

  const showRequestResponse = computed(() => {
    const currentTab = tabs.value[activeTab.value];
    return (
      currentNavKey.value === "collection" && currentTab?.tabType === "api"
    );
  });

  const showCollectionSettings = computed(() => {
    const currentTab = tabs.value[activeTab.value];
    return (
      currentNavKey.value === "collection" &&
      currentTab?.tabType === "collection"
    );
  });

  const findApisInCollections = (collections, apiIds) => {
    const apis = [];
    for (const item of collections) {
      if (item.type === "api" && apiIds.includes(item.id)) {
        apis.push(item);
      }
      if (item.children && item.children.length > 0) {
        const childApis = findApisInCollections(item.children, apiIds);
        apis.push(...childApis);
      }
    }
    return apis;
  };

  const findCollection = (collections, collectionId) => {
    for (const collection of collections) {
      if (collection.id === collectionId) {
        return collection;
      }
      if (collection.children) {
        const found = findCollection(collection.children, collectionId);
        if (found) return found;
      }
    }
    return null;
  };

  // 收集集合及其所有子集合下的 API ID
  const collectApiIdsInCollection = (collection) => {
    const apiIds = [];
    if (collection.children) {
      for (const child of collection.children) {
        if (child.type === "api") {
          apiIds.push(child.id);
        } else if (child.type === "collection") {
          apiIds.push(...collectApiIdsInCollection(child));
        }
      }
    }
    return apiIds;
  };

  // 查找 API 所属的所有祖先集合（从根到直接父集合）
  // 实现已抽取到 utils/collectionTree.js 的 findAncestorCollectionsForApi

  // 合并所有祖先集合的变量（从根到父，子覆盖父）
  // 实现已抽取到 utils/collectionTree.js 的 mergeCollectionVariables

  // 合并所有祖先集合的请求头（从根到父，子覆盖父）
  const mergeCommonHeaders = (ancestorCollections) => {
    const commonHeaders = [];
    for (const collection of ancestorCollections) {
      if (collection.common_headers) {
        for (const h of collection.common_headers) {
          if (h.enabled && h.key.trim()) {
            const existingIndex = commonHeaders.findIndex(
              (ch) => ch.key.toLowerCase() === h.key.toLowerCase(),
            );
            if (existingIndex >= 0) {
              commonHeaders[existingIndex] = h;
            } else {
              commonHeaders.push(h);
            }
          }
        }
      }
    }
    return commonHeaders;
  };

  const loadOpenTabs = async (workspaceId) => {
    try {
      const [openTabIds, openTabTypes, activeIndex, savedRequestTabs] =
        await invoke("get_open_tabs", { workspaceId });
      requestTabs.value = savedRequestTabs || {};
      if (openTabIds.length > 0) {
        // 使用轻量级接口只加载需要的项
        const itemsWithAncestors = await invoke("get_items_by_ids", {
          workspaceId,
          itemIds: openTabIds,
        });

        for (let i = 0; i < openTabIds.length; i++) {
          const tabId = openTabIds[i];
          const tabType = openTabTypes[i] || "api";
          const itemData = itemsWithAncestors[i];

          if (!itemData) continue;

          if (tabType === "api" || itemData.item.item_type === "api") {
            const api = itemData.item;
            const ancestors = itemData.ancestors || [];
            const commonHeaders = mergeCommonHeaders(ancestors);
            const collectionVariables = mergeCollectionVariables(ancestors);

            tabs.value.push({
              id: api.id,
              name: api.name,
              method: api.method || "GET",
              url: api.url || "",
              params: api.params || [],
              headers: api.headers || [],
              body: api.body || "",
              bodyType: api.body_type || "raw",
              formData: api.form_fields || [],
              binaryFile: api.binary_file_path
                ? {
                    path: api.binary_file_path,
                    name: api.binary_file_path.split(/[/\\]/).pop(),
                  }
                : null,
              tabType: "api",
              commonHeaders,
              collectionVariables,
              timeout: api.timeout,
            });
          } else if (
            tabType === "collection" ||
            itemData.item.item_type === "collection"
          ) {
            const collection = itemData.item;
            collectionTabsData.value[tabId] = collection;
            tabs.value.push({
              id: tabId,
              name: collection.name,
              tabType: "collection",
            });
          }
        }

        if (tabs.value.length > 0) {
          activeTab.value = Math.min(activeIndex || 0, tabs.value.length - 1);
          updateCurrentRequest();
          const currentTab = tabs.value[activeTab.value];
          if (currentTab?.id) {
            currentRequestTab.value =
              requestTabs.value[currentTab.id] ||
              (currentTab.method?.toUpperCase() === "POST" ? "body" : "params");
          }
        }
      }
    } catch (e) {
      console.error("加载打开的标签页失败:", e);
    }
  };

  const saveOpenTabs = async () => {
    if (!currentWorkspace.value?.id) return;
    try {
      const openTabs = [];
      const openTabTypes = [];

      tabs.value.forEach((tab) => {
        openTabs.push(tab.id);
        openTabTypes.push(tab.tabType || "api");
      });

      await invoke("save_open_tabs", {
        workspaceId: currentWorkspace.value?.id,
        openTabs: openTabs,
        openTabTypes: openTabTypes,
        activeTabIndex: activeTab.value,
        requestTabs: requestTabs.value,
      });
    } catch (e) {
      console.error("保存标签页失败:", e);
    }
  };

  const selectCollection = (collectionOrId) => {
    // 支持传入 collection 对象或 collectionId
    let collection;
    let collectionId;

    if (typeof collectionOrId === "string") {
      collectionId = collectionOrId;
      collection = collectionTabsData.value[collectionId];
    } else {
      collection = collectionOrId;
      collectionId = collection.id;
    }

    if (!collection) {
      // 如果没有找到 collection 对象，只切换 tab
      const existingIndex = tabs.value.findIndex(
        (t) => t.id === collectionId && t.tabType === "collection",
      );
      if (existingIndex >= 0) {
        activeTab.value = existingIndex;
      }
      // 通知侧边栏选中集合
      if (sidebarRef.value) {
        sidebarRef.value.setSelectedCollection(collectionId);
      }
      return;
    }

    const existingIndex = tabs.value.findIndex(
      (t) => t.id === collectionId && t.tabType === "collection",
    );

    if (existingIndex >= 0) {
      activeTab.value = existingIndex;
    } else {
      collectionTabsData.value[collectionId] = collection;
      tabs.value.push({
        id: collectionId,
        name: collection.name,
        tabType: "collection",
      });
      activeTab.value = tabs.value.length - 1;
    }

    // 通知侧边栏选中集合
    if (sidebarRef.value) {
      sidebarRef.value.setSelectedCollection(collectionId);
    }
  };

  const onCollectionSettingsSaved = async () => {
    // 重新加载侧边栏集合列表
    sidebarRef.value?.loadCollections();

    // 更新当前打开的集合标签页数据
    const currentTab = tabs.value[activeTab.value];
    if (currentTab?.tabType === "collection" && currentWorkspace.value?.id) {
      try {
        const collections = await invoke("get_collections", {
          workspaceId: currentWorkspace.value?.id,
        });
        const updatedCollection = findCollection(collections, currentTab.id);
        if (updatedCollection) {
          collectionTabsData.value[currentTab.id] = updatedCollection;

          // 更新所有属于该集合的子 API 标签页的 commonHeaders 和 collectionVariables
          const childApiIds = collectApiIdsInCollection(updatedCollection);
          for (const apiId of childApiIds) {
            const apiTab = tabs.value.find(
              (t) => t.id === apiId && t.tabType === "api",
            );
            if (apiTab) {
              // 获取该 API 的祖先集合
              const ancestors = findAncestorCollectionsForApi(
                collections,
                apiId,
              );
              // 合并所有祖先集合的请求头和变量
              apiTab.commonHeaders = mergeCommonHeaders(ancestors);
              apiTab.collectionVariables = mergeCollectionVariables(ancestors);
            }
          }
        }
      } catch (e) {
        console.error("更新集合标签页数据失败:", e);
      }
    }
  };

  const closeTab = async (index) => {
    loading.value = false;

    const wasActive = index === activeTab.value;
    const closedTab = tabs.value[index];
    tabs.value.splice(index, 1);

    // 清理集合数据
    if (closedTab?.tabType === "collection") {
      delete collectionTabsData.value[closedTab.id];
    }

    if (tabs.value.length === 0) {
      activeTab.value = 0;
      currentRequest.method = "GET";
      currentRequest.url = "";
      currentRequest.headers = [];
      currentRequest.body = "";
      currentRequest.bodyType = "raw";
      currentRequest.formData = [];
      currentRequest.binaryFile = null;
      response.value = null;
      if (sidebarRef.value) {
        sidebarRef.value.setSelectedApi(null);
      }
    } else if (activeTab.value >= tabs.value.length) {
      activeTab.value = tabs.value.length - 1;
    } else if (wasActive) {
      const currentTab = tabs.value[activeTab.value];
      if (currentTab?.tabType === "api" && sidebarRef.value) {
        sidebarRef.value.setSelectedApi(currentTab.id);
      }
    }

    await saveOpenTabs();
  };

  // 关闭所有标签页
  const closeAllTabs = async () => {
    loading.value = false;
    tabs.value = [];
    collectionTabsData.value = {};
    activeTab.value = 0;
    currentRequest.method = "GET";
    currentRequest.url = "";
    currentRequest.headers = [];
    currentRequest.body = "";
    currentRequest.bodyType = "raw";
    currentRequest.formData = [];
    currentRequest.binaryFile = null;
    response.value = null;
    if (sidebarRef.value) {
      sidebarRef.value.setSelectedApi(null);
    }
    await saveOpenTabs();
  };

  // 关闭其他标签页（保留指定索引的标签页）
  const closeOtherTabs = async (keepIndex = null) => {
    if (tabs.value.length <= 1) return;

    loading.value = false;
    // 如果没有指定保留索引，则保留当前激活的标签页
    const targetIndex = keepIndex !== null ? keepIndex : activeTab.value;
    const targetTab = tabs.value[targetIndex];

    if (!targetTab) return;

    // 清理其他集合标签页的数据
    for (let i = 0; i < tabs.value.length; i++) {
      if (i !== targetIndex && tabs.value[i].tabType === "collection") {
        delete collectionTabsData.value[tabs.value[i].id];
      }
    }

    // 只保留目标标签页
    tabs.value = [targetTab];
    activeTab.value = 0;

    // 通知侧边栏选中当前 API
    if (targetTab?.tabType === "api" && sidebarRef.value) {
      sidebarRef.value.setSelectedApi(targetTab.id);
    }

    await saveOpenTabs();
  };

  const onDeleteApis = (apiIds) => {
    for (const apiId of apiIds) {
      const index = tabs.value.findIndex(
        (t) => t.id === apiId && t.tabType === "api",
      );
      if (index >= 0) {
        tabs.value.splice(index, 1);
      }
    }
    if (tabs.value.length === 0) {
      activeTab.value = 0;
      currentRequest.method = "GET";
      currentRequest.url = "";
      currentRequest.headers = [];
      currentRequest.body = "";
      currentRequest.bodyType = "raw";
    } else if (activeTab.value >= tabs.value.length) {
      activeTab.value = tabs.value.length - 1;
    }
  };

  const onDeleteCollection = (collectionId) => {
    const indicesToRemove = [];
    for (let i = 0; i < tabs.value.length; i++) {
      if (
        tabs.value[i].id === collectionId &&
        tabs.value[i].tabType === "collection"
      ) {
        indicesToRemove.push(i);
      }
    }

    for (const index of indicesToRemove.reverse()) {
      tabs.value.splice(index, 1);
      delete collectionTabsData.value[collectionId];
    }

    if (tabs.value.length === 0) {
      activeTab.value = 0;
    } else if (activeTab.value >= tabs.value.length) {
      activeTab.value = tabs.value.length - 1;
    }
  };

  const onUpdateRequestTab = async (tabKey) => {
    currentRequestTab.value = tabKey;
    const currentTab = tabs.value[activeTab.value];
    if (currentTab?.id && currentTab.tabType === "api") {
      requestTabs.value[currentTab.id] = tabKey;
      await saveOpenTabs();
    }
  };

  const updateCurrentRequest = () => {
    if (tabs.value.length === 0) {
      currentRequest.method = "GET";
      currentRequest.url = "";
      currentRequest.params = [];
      currentRequest.headers = [];
      currentRequest.body = "";
      currentRequest.bodyType = "raw";
      currentRequest.formData = [];
      currentRequest.binaryFile = null;
      response.value = null;
      return;
    }

    const currentTab = tabs.value[activeTab.value];
    if (!currentTab || currentTab.tabType !== "api") {
      currentRequest.method = "GET";
      currentRequest.url = "";
      currentRequest.params = [];
      currentRequest.headers = [];
      currentRequest.body = "";
      currentRequest.bodyType = "raw";
      currentRequest.formData = [];
      currentRequest.binaryFile = null;
      response.value = null;
      return;
    }

    currentRequest.method = currentTab.method || "GET";
    currentRequest.url = currentTab.url || "";
    currentRequest.params = currentTab.params || [];
    currentRequest.headers = currentTab.headers || [];
    currentRequest.body = currentTab.body || "";
    currentRequest.bodyType = currentTab.bodyType || "raw";
    currentRequest.formData = currentTab.formData || [];
    currentRequest.binaryFile = currentTab.binaryFile || null;
    currentRequest.timeout = currentTab.timeout;

    // 恢复保存的响应数据
    response.value = currentTab.lastResponseData || null;
  };

  return {
    tabs,
    activeTab,
    collectionTabsData,
    currentRequestTab,
    displayTabs,
    selectedCollection,
    showRequestResponse,
    showCollectionSettings,
    findApisInCollections,
    findCollection,
    loadOpenTabs,
    saveOpenTabs,
    selectCollection,
    onCollectionSettingsSaved,
    closeTab,
    closeAllTabs,
    closeOtherTabs,
    onDeleteApis,
    onDeleteCollection,
    onUpdateRequestTab,
    updateCurrentRequest,
  };
}
