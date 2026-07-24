import { ref, computed, watch, inject, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { showToast } from "../../../composables/useToast";
import { mergeCollectionVariablesToObject } from "../../../utils/scriptEngine.js";
import { useDialogEscape } from "../../../composables/useDialogStack.js";
import JSON5 from "json5";

// 导出 composable 函数
export function useCollectionPanelSetup(props, emit) {
  const { t } = useI18n();

  // 从父组件注入 tabs 数据（用于 curl 导出时获取实时编辑数据）
  const appTabs = inject("appTabs", ref([]));

  // 集合数据
  const collections = ref([]);
  const expandedItems = ref({});
  const selectedApi = ref(null);
  const selectedCollectionId = ref(null); // 选中的集合 ID
  const searchQuery = ref("");

  // Inline 编辑状态（用于新建和重命名）
  const editingItem = ref(null); // { tempId, parentId, type: 'collection'|'api', isNew: boolean }
  const editingName = ref("");

  // 右键菜单
  const contextMenu = ref({
    visible: false,
    x: 0,
    y: 0,
    item: null,
    depth: 0,
    type: "", // 'collection' 或 'root' 或 'api'
  });

  // 保存响应展开状态
  const expandedResponses = ref({});
  // 已加载响应列表的 API ID 集合（避免重复查询）
  const loadedResponsesApis = ref(new Set());

  // 拖拽排序状态（鼠标事件实现）
  const dragState = ref({
    draggingId: null,
    dragOverId: null,
    dragPosition: null, // 'before' | 'after'
  });
  let isDragging = false;
  let dragStartY = 0;
  let dragStartId = null;
  let dragListenersAdded = false; // 跟踪监听器是否已添加
  let dragStartRow = null;
  let dragStartOnName = false; // 记录是否点击在集合名字上
  const DRAG_THRESHOLD = 4; // 移动超过4px才触发拖拽

  const treeListRef = ref(null);

  // 加载集合列表
  const loadCollections = async () => {
    if (!props.workspace?.id) return;
    try {
      const data = await invoke("get_collections_tree", {
        workspaceId: props.workspace.id,
      });
      collections.value = data || [];

      // 恢复展开状态
      const expandedIds = await invoke("get_expanded_collections", {
        workspaceId: props.workspace.id,
      });
      expandedItems.value = {};
      for (const id of expandedIds || []) {
        expandedItems.value[id] = true;
      }
    } catch (e) {
      console.error("加载集合失败:", e);
      collections.value = [];
    }
  };

  // 保存展开状态
  const saveExpandState = async () => {
    if (!props.workspace?.id) return;
    const expandedIds = Object.keys(expandedItems.value).filter(
      (id) => expandedItems.value[id],
    );
    try {
      await invoke("save_expanded_collections", {
        workspaceId: props.workspace.id,
        expandedIds,
      });
    } catch (e) {
      console.error("保存展开状态失败:", e);
    }
  };

  // 递归收集集合下所有子孙集合 ID
  const collectDescendantCollectionIds = (items, result = []) => {
    for (const item of items) {
      if (item.type === "collection") {
        result.push(item.id);
      }
      if (item.children && item.children.length > 0) {
        collectDescendantCollectionIds(item.children, result);
      }
    }
    return result;
  };

  // 在集合树中查找指定 ID 的节点
  const findCollectionNode = (items, id) => {
    for (const item of items) {
      if (item.id === id) return item;
      if (item.children && item.children.length > 0) {
        const found = findCollectionNode(item.children, id);
        if (found) return found;
      }
    }
    return null;
  };

  // 展开/折叠
  const toggleExpand = async (id) => {
    const willExpand = !expandedItems.value[id];
    expandedItems.value[id] = willExpand;
    // 折叠时，同步折叠所有子孙集合
    if (!willExpand) {
      const node = findCollectionNode(collections.value, id);
      if (node && node.children) {
        const descendantIds = collectDescendantCollectionIds(node.children);
        for (const descId of descendantIds) {
          expandedItems.value[descId] = false;
        }
      }
    }
    await saveExpandState();
  };

  const isExpanded = (id) => expandedItems.value[id];

  // 选择 API
  const selectApiItem = async (api) => {
    selectedApi.value = api.id;
    selectedCollectionId.value = null; // 清空集合选中状态

    // 展开所有祖先集合，使 API 可见
    const ancestorIds = findAncestorIds(api.id);
    for (const ancestorId of ancestorIds) {
      if (!expandedItems.value[ancestorId]) {
        expandedItems.value[ancestorId] = true;
      }
    }
    await saveExpandState();

    // 获取完整的 API 详情（包括祖先链）
    const itemsWithAncestors = await invoke("get_items_by_ids", {
      workspaceId: props.workspace.id,
      itemIds: [api.id],
    });

    if (itemsWithAncestors && itemsWithAncestors[0]) {
      const apiDetail = itemsWithAncestors[0].item;
      const ancestors = itemsWithAncestors[0].ancestors || [];

      // 合并所有祖先集合的请求头（从根到父，子覆盖父）
      const commonHeaders = [];
      for (const collection of ancestors) {
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

      // 合并所有祖先集合的变量（从根到父，子覆盖父）
      const collectionVariables = [];
      for (const collection of ancestors) {
        if (collection.collection_variables) {
          for (const v of collection.collection_variables) {
            const existingIndex = collectionVariables.findIndex(
              (cv) => cv.key === v.key,
            );
            if (existingIndex >= 0) {
              collectionVariables[existingIndex] = v;
            } else {
              collectionVariables.push(v);
            }
          }
        }
      }

      const apiWithHeaders = {
        ...apiDetail,
        commonHeaders,
        collectionVariables,
      };
      emit("selectApi", apiWithHeaders);
    }
  };

  // 查找 API 的祖先 ID 链（仅返回 ID）
  const findAncestorIds = (apiId) => {
    const ids = [];
    const search = (items) => {
      for (const item of items) {
        if (item.type === "api" && item.id === apiId) {
          return true;
        }
        if (item.type === "collection" && item.children) {
          ids.push(item.id);
          if (search(item.children)) return true;
          ids.pop();
        }
      }
      return false;
    };
    search(collections.value);
    return ids;
  };

  // 选择集合（打开设置页面）
  const selectCollectionItem = async (collection) => {
    selectedCollectionId.value = collection.id;
    selectedApi.value = null; // 清空 API 选中状态

    // 展开该集合的所有祖先集合，使其可见
    const ancestors = findAncestorCollectionsForItem(
      collections.value,
      collection.id,
    );
    if (ancestors) {
      // 找到了（可能是空数组，表示顶级集合）
      for (const ancestor of ancestors) {
        if (!expandedItems.value[ancestor.id]) {
          expandedItems.value[ancestor.id] = true;
        }
      }
      await saveExpandState();
    }

    emit("selectCollection", collection);
  };

  // 外部设置选中 API（用于标签页联动）
  const setSelectedApiId = async (apiId) => {
    selectedApi.value = apiId;
    selectedCollectionId.value = null; // 清空集合选中状态

    // 展开该 API 的所有祖先集合，使其可见
    if (apiId) {
      const ancestors = findAncestorCollectionsForItem(
        collections.value,
        apiId,
      );
      if (ancestors) {
        // 找到了（可能是空数组，表示顶级项）
        for (const ancestor of ancestors) {
          if (!expandedItems.value[ancestor.id]) {
            expandedItems.value[ancestor.id] = true;
          }
        }
        await saveExpandState();
      }
    }
  };

  // 外部设置选中集合（用于标签页联动）
  const setSelectedCollectionId = async (collectionId) => {
    selectedCollectionId.value = collectionId;
    selectedApi.value = null; // 清空 API 选中状态

    // 展开该集合的所有祖先集合，使其可见
    if (collectionId) {
      const ancestors = findAncestorCollectionsForItem(
        collections.value,
        collectionId,
      );
      if (ancestors) {
        // 找到了（可能是空数组，表示顶级集合）
        for (const ancestor of ancestors) {
          if (!expandedItems.value[ancestor.id]) {
            expandedItems.value[ancestor.id] = true;
          }
        }
        await saveExpandState();
      }
    }
  };

  // 查找集合的所有祖先集合（不包括自己）
  const findAncestorCollectionsForItem = (items, targetId, path = []) => {
    for (const item of items) {
      if (item.id === targetId) {
        return path; // 找到目标，返回祖先路径（可能是空数组，表示是顶级项）
      }
      if (item.type === "collection" && item.children) {
        const newPath = [...path, item];
        const found = findAncestorCollectionsForItem(
          item.children,
          targetId,
          newPath,
        );
        if (found !== null) return found; // 找到了，返回结果（包括空数组）
      }
    }
    return null; // 没找到，返回 null（与空数组区分）
  };

  // 是否可以创建子集合（层级限制：只有根级别可以创建子集合）
  const canCreateSubCollection = (depth) => {
    return depth < 1; // depth=0 才能创建子集合，depth=1 的集合不能再创建子集合
  };

  // 打开右键菜单
  const openContextMenu = (event, item, depth, type) => {
    event.preventDefault();
    event.stopPropagation();

    contextMenu.value = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      item: item,
      depth: depth,
      type: type,
    };
  };

  // 打开保存响应右键菜单
  const openSavedResponseContextMenu = (event, savedResponse) => {
    event.preventDefault();
    event.stopPropagation();

    contextMenu.value = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      item: savedResponse,
      depth: 0,
      type: "saved-response",
    };
  };

  // 关闭右键菜单
  const closeContextMenu = () => {
    contextMenu.value.visible = false;
  };

  useDialogEscape(() => contextMenu.value.visible, closeContextMenu);

  // 右键菜单操作
  const handleContextAction = async (action) => {
    const { item, depth, type } = contextMenu.value;

    if (action === "new-collection") {
      if (type === "root") {
        startInlineEdit(null, "collection", 0);
      } else if (canCreateSubCollection(depth)) {
        startInlineEdit(item.id, "collection", depth + 1);
      }
    } else if (action === "new-api") {
      if (type === "root") {
        startInlineEdit(null, "api", 0);
      } else {
        startInlineEdit(item.id, "api", depth + 1);
      }
    } else if (action === "rename") {
      if (type === "collection") {
        startInlineEdit(item.id, "collection", depth, false);
      } else {
        startInlineEdit(item.id, "api", depth, false);
      }
    } else if (action === "delete") {
      deleteItem(item);
    } else if (action === "delete-saved-response") {
      deleteSavedResponse(item);
    } else if (action === "duplicate") {
      duplicateItem(item);
    } else if (action === "export-curl") {
      await exportAsCurl(item);
    } else if (action === "export-postman") {
      await exportAsPostman(item);
    }

    closeContextMenu();
  };

  // 导出 API 为 curl 命令（执行前置脚本、变量替换，复制到剪贴板）
  const exportAsCurl = async (api) => {
    if (!props.workspace?.id) {
      showToast(t("toast.apiMoveFailed"), "error");
      return;
    }

    const workspaceId = props.workspace.id;
    const apiId = api.id;

    try {
      // 优先使用当前打开 tab 的实时数据（包含用户未保存的编辑），
      // 避免导出数据库中的旧数据导致请求头等内容缺失
      const currentTab = appTabs.value.find(
        (t) => t.id === apiId && t.tabType === "api",
      );

      let fullApi;
      if (currentTab) {
        // 从 tab 实时数据构建（字段名映射 tab → Collection 结构）
        fullApi = {
          url: currentTab.url || "",
          method: currentTab.method || "GET",
          headers: currentTab.headers || [],
          body: currentTab.body || "",
          body_type: currentTab.bodyType,
          form_fields: currentTab.formData,
        };
      } else {
        // API 未在 tab 中打开，从数据库获取完整数据
        const itemsWithAncestors = await invoke("get_items_by_ids", {
          workspaceId,
          itemIds: [apiId],
        });

        if (!itemsWithAncestors || itemsWithAncestors.length === 0) {
          showToast(t("toast.apiNotFound"), "error");
          return;
        }

        fullApi = itemsWithAncestors[0].item;
      }

      // 获取集合数据
      const collectionsData = await invoke("get_collections", {
        workspaceId,
      });

      // 查找祖先集合链（从根到父）
      const findAncestorCollectionsForApi = (collections, targetApiId) => {
        const search = (items, path = []) => {
          for (const item of items) {
            if (item.type === "api" && item.id === targetApiId) {
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
        return search(collections) || [];
      };

      const ancestorCollections = findAncestorCollectionsForApi(
        collectionsData,
        apiId,
      );

      // 获取环境变量和环境配置
      const envConfig = await invoke("get_environments", { workspaceId });
      const activeEnvVars = await invoke("get_active_variables", {
        workspaceId,
      });
      const collVarsObj = mergeCollectionVariablesToObject(ancestorCollections);

      // 获取当前激活环境的 ID
      const environmentId = envConfig.active_environment_id;

      // 构建请求对象（使用完整 API 数据）
      const request = {
        url: fullApi.url || "",
        method: fullApi.method || "GET",
        headers: fullApi.headers || [],
        body: fullApi.body || "",
      };

      // 执行前置脚本链（调用后端命令）
      const preScriptResult = await invoke("execute_pre_scripts_cmd", {
        input: {
          workspace_id: workspaceId,
          api_id: apiId,
          environment_id: environmentId,
          ancestor_collections: ancestorCollections.map((c) => ({
            id: c.id,
            name: c.name,
            collection_variables: c.collection_variables || [],
          })),
          environment_variables: activeEnvVars || {},
          collection_variables: collVarsObj,
          request: {
            url: request.url,
            method: request.method,
            headers: request.headers,
            body: request.body,
          },
          silent: true,
        },
      });

      let modifiedRequest = request;
      let modifiedCollVars = collVarsObj;

      if (preScriptResult.success) {
        modifiedRequest = preScriptResult.modified_request || request;
        modifiedCollVars =
          preScriptResult.modified_collection_vars || collVarsObj;
      }

      // 合并请求头（环境 → 集合 → 接口）
      const headersMap = new Map();

      // 合并环境请求头（优先级最低）
      if (envConfig && envConfig.active_environment_id) {
        const activeEnv = envConfig.environments.find(
          (e) => e.id === envConfig.active_environment_id,
        );
        if (activeEnv && activeEnv.common_headers) {
          for (const h of activeEnv.common_headers) {
            if (h.enabled && h.key.trim()) {
              headersMap.set(h.key.toLowerCase(), h);
            }
          }
        }
      }

      // 合并集合公共请求头（覆盖环境请求头）
      for (const collection of ancestorCollections) {
        if (collection.common_headers) {
          for (const h of collection.common_headers) {
            if (h.enabled && h.key.trim()) {
              headersMap.set(h.key.toLowerCase(), h);
            }
          }
        }
      }

      // 合并接口请求头（覆盖集合和环境请求头）
      for (const h of modifiedRequest.headers || []) {
        if (h.enabled && h.key.trim()) {
          headersMap.set(h.key.toLowerCase(), h);
        }
      }

      const headersToSend = Array.from(headersMap.values());

      // 处理 JSON5 转 JSON（如果 Content-Type 是 json）
      let bodyToSend = modifiedRequest.body;
      const contentTypeHeader = headersToSend.find(
        (h) => h.key.toLowerCase() === "content-type",
      );

      if (contentTypeHeader?.value?.includes("json") && modifiedRequest.body) {
        try {
          const parsed = JSON5.parse(modifiedRequest.body);
          bodyToSend = JSON.stringify(parsed);
        } catch {
          try {
            const parsed = JSON.parse(modifiedRequest.body);
            bodyToSend = JSON.stringify(parsed);
          } catch {
            /* ignore */
          }
        }
      }

      // 构建集合变量数组
      const collectionVariablesArray = Object.entries(modifiedCollVars).map(
        ([key, value]) => ({
          key,
          value,
          enabled: true,
        }),
      );

      // 构建 form_fields 数组
      const formFields =
        fullApi.form_fields?.map((field) => ({
          key: field.key,
          value: field.value,
          type: field.type || field.field_type || "text",
          enabled: field.enabled,
          files: field.files,
        })) || null;

      // 调用后端生成 curl 命令
      const curlCommand = await invoke("export_as_curl", {
        method: modifiedRequest.method,
        url: modifiedRequest.url,
        headers: headersToSend,
        body: bodyToSend || null,
        bodyType: fullApi.body_type || null,
        formFields: formFields,
        workspaceId: workspaceId,
        collectionVariables:
          collectionVariablesArray.length > 0 ? collectionVariablesArray : null,
      });

      // 复制到剪贴板
      await navigator.clipboard.writeText(curlCommand);
      showToast(t("toast.curlCopied"), "success");
    } catch (e) {
      console.error("导出 curl 失败:", e);
      showToast(t("toast.saveFailed"), "error");
    }
  };

  // 导出集合为 Postman 2.1 格式（执行前置脚本、变量替换）
  const exportAsPostman = async (collection) => {
    if (!props.workspace?.id) {
      showToast(t("toast.collectionExportFailed"), "error");
      return;
    }

    const workspaceId = props.workspace.id;

    try {
      // 打开保存文件对话框
      const filePath = await invoke("safe_save_file", {
        defaultName: `${collection.name}.postman_collection.json`,
      });

      if (!filePath) {
        return;
      }

      // 获取集合数据
      const collectionsData = await invoke("get_collections", {
        workspaceId,
      });

      // 获取环境变量和环境配置
      const envConfig = await invoke("get_environments", { workspaceId });
      const activeEnvVars = await invoke("get_active_variables", {
        workspaceId,
      });
      const environmentId = envConfig?.active_environment_id || null;

      // 递归处理集合中的所有 API（执行前置脚本、变量替换）
      const processCollection = async (col, ancestorCollections = []) => {
        const processedCol = {
          id: col.id,
          name: col.name,
          description: col.description,
          type: "collection",
          children: [],
          common_headers: col.common_headers,
          collection_variables: col.collection_variables,
        };

        // 处理子项
        if (col.children && col.children.length > 0) {
          for (const child of col.children) {
            if (child.type === "api") {
              // 处理 API：执行前置脚本、变量替换
              const processedApi = await processApi(child, [
                ...ancestorCollections,
                col,
              ]);
              processedCol.children.push(processedApi);
            } else {
              // 递归处理子集合
              const processedChild = await processCollection(child, [
                ...ancestorCollections,
                col,
              ]);
              processedCol.children.push(processedChild);
            }
          }
        }

        return processedCol;
      };

      // 处理单个 API（执行前置脚本、变量替换）
      const processApi = async (api, ancestorCollections) => {
        // 合并集合变量
        const collVarsObj =
          mergeCollectionVariablesToObject(ancestorCollections);

        // 构建请求对象
        const request = {
          url: api.url || "",
          method: api.method || "GET",
          headers: api.headers || [],
          body: api.body || "",
        };

        // 执行前置脚本链（调用后端命令）
        const preScriptResult = await invoke("execute_pre_scripts_cmd", {
          input: {
            workspace_id: workspaceId,
            api_id: api.id,
            environment_id: environmentId,
            ancestor_collections: ancestorCollections.map((c) => ({
              id: c.id,
              name: c.name,
              collection_variables: c.collection_variables || [],
            })),
            environment_variables: activeEnvVars || {},
            collection_variables: collVarsObj,
            request: {
              url: request.url,
              method: request.method,
              headers: request.headers,
              body: request.body,
            },
            silent: true,
          },
        });

        let modifiedRequest = request;
        let modifiedCollVars = collVarsObj;

        if (preScriptResult.success) {
          modifiedRequest = preScriptResult.modified_request || request;
          modifiedCollVars =
            preScriptResult.modified_collection_vars || collVarsObj;
        }

        // 合并所有变量（环境变量 + 集合变量 + 脚本修改后的变量）
        const allVars = { ...(activeEnvVars || {}), ...modifiedCollVars };

        // 替换 URL 中的变量
        let urlToSend = modifiedRequest.url;
        if (urlToSend && urlToSend.includes("{{")) {
          urlToSend = await invoke("replace_variables_text", {
            text: urlToSend,
            variables: Object.entries(allVars).map(([key, value]) => ({
              key,
              value,
              enabled: true,
            })),
          });
        }

        // 合并请求头（环境 → 集合 → 接口）
        const headersMap = new Map();

        // 合并环境请求头
        if (envConfig && envConfig.active_environment_id) {
          const activeEnv = envConfig.environments.find(
            (e) => e.id === envConfig.active_environment_id,
          );
          if (activeEnv && activeEnv.common_headers) {
            for (const h of activeEnv.common_headers) {
              if (h.enabled && h.key.trim()) {
                headersMap.set(h.key.toLowerCase(), h);
              }
            }
          }
        }

        // 合并集合公共请求头
        for (const collection of ancestorCollections) {
          if (collection.common_headers) {
            for (const h of collection.common_headers) {
              if (h.enabled && h.key.trim()) {
                headersMap.set(h.key.toLowerCase(), h);
              }
            }
          }
        }

        // 合并接口请求头
        for (const h of modifiedRequest.headers || []) {
          if (h.enabled && h.key.trim()) {
            headersMap.set(h.key.toLowerCase(), h);
          }
        }

        const headersToSend = Array.from(headersMap.values());

        // 替换 Headers 中的变量
        const replacedHeaders = await Promise.all(
          headersToSend.map(async (h) => {
            if (h.value && h.value.includes("{{")) {
              const replaced = await invoke("replace_variables_text", {
                text: h.value,
                variables: Object.entries(allVars).map(([key, value]) => ({
                  key,
                  value,
                  enabled: true,
                })),
              });
              return { ...h, value: replaced };
            }
            return h;
          }),
        );

        // 处理 JSON5 转 JSON
        let bodyToSend = modifiedRequest.body;
        const contentTypeHeader = replacedHeaders.find(
          (h) => h.key.toLowerCase() === "content-type",
        );

        if (
          contentTypeHeader?.value?.includes("json") &&
          modifiedRequest.body
        ) {
          try {
            const parsed = JSON5.parse(modifiedRequest.body);
            bodyToSend = JSON.stringify(parsed);
          } catch {
            try {
              const parsed = JSON.parse(modifiedRequest.body);
              bodyToSend = JSON.stringify(parsed);
            } catch {
              /* ignore */
            }
          }
        }

        // 替换 Body 中的变量
        if (bodyToSend && bodyToSend.includes("{{")) {
          bodyToSend = await invoke("replace_variables_text", {
            text: bodyToSend,
            variables: Object.entries(allVars).map(([key, value]) => ({
              key,
              value,
              enabled: true,
            })),
          });
        }

        // 处理 form_fields 中的变量替换
        let formFieldsToSend = null;
        if (api.form_fields) {
          formFieldsToSend = await Promise.all(
            api.form_fields.map(async (field) => {
              if (!field.enabled) return { ...field };

              const processedField = { ...field };

              if (field.value && field.value.includes("{{")) {
                const replaced = await invoke("replace_variables_text", {
                  text: field.value,
                  variables: Object.entries(allVars).map(([key, value]) => ({
                    key,
                    value,
                    enabled: true,
                  })),
                });
                processedField.value = replaced;
              }

              return processedField;
            }),
          );
        }

        return {
          id: api.id,
          name: api.name,
          description: api.description,
          type: "api",
          children: [],
          method: modifiedRequest.method,
          url: urlToSend,
          headers: replacedHeaders,
          body: bodyToSend || null,
          body_type: api.body_type,
          form_fields: formFieldsToSend,
          params: api.params,
          collection_variables: api.collection_variables,
        };
      };

      // 查找要导出的集合
      const findCollection = (items, targetId) => {
        for (const item of items) {
          if (item.id === targetId && item.type === "collection") {
            return item;
          }
          if (item.type === "collection" && item.children) {
            const found = findCollection(item.children, targetId);
            if (found) return found;
          }
        }
        return null;
      };

      const targetCollection = findCollection(collectionsData, collection.id);
      if (!targetCollection) {
        showToast(t("toast.collectionExportFailed"), "error");
        return;
      }

      // 处理集合（递归处理所有 API）
      const processedCollection = await processCollection(targetCollection);

      // 调用后端生成 Postman JSON
      const postmanJson = await invoke("export_collection_postman_with_data", {
        collection: processedCollection,
      });

      // 使用 fs plugin 写入文件
      const { writeTextFile } = await import("@tauri-apps/plugin-fs");
      await writeTextFile(filePath, postmanJson);

      showToast(t("toast.collectionExportSuccess"), "success");
    } catch (e) {
      console.error("导出 Postman 失败:", e);
      showToast(t("toast.collectionExportFailed"), "error");
    }
  };

  // 删除保存的响应
  const deleteSavedResponse = async (savedResponse) => {
    if (!props.workspace?.id) return;
    try {
      await invoke("delete_saved_response", {
        workspaceId: props.workspace.id,
        id: savedResponse.id,
      });

      // 只刷新该 API 的保存响应列表，而不是重新加载整个集合树
      await refreshApiSavedResponses(savedResponse.api_id);
    } catch (e) {
      console.error("删除保存响应失败:", e);
    }
  };

  const duplicateItem = async (item) => {
    if (!props.workspace?.id) return;
    try {
      if (item.type === "collection") {
        await invoke("duplicate_collection", {
          workspaceId: props.workspace.id,
          collectionId: item.id,
        });
      } else {
        await invoke("duplicate_api", {
          workspaceId: props.workspace.id,
          apiId: item.id,
        });
      }
      await loadCollections();
      showToast(
        item.type === "collection"
          ? t("toast.collectionDuplicated")
          : t("toast.apiDuplicated"),
        "success",
      );
    } catch (e) {
      console.error("复制失败:", e);
      showToast(
        item.type === "collection"
          ? t("toast.collectionDuplicateFailed")
          : t("toast.apiDuplicateFailed"),
        "error",
      );
    }
  };

  // ============ Inline 编辑功能 ============

  // 开始 inline 编辑（新建或重命名）
  const startInlineEdit = (parentId, itemType, depth, isNew = true) => {
    const tempId = `temp-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;

    // 如果是新建子集合，先展开父集合（同步，确保 computed 能立即响应）
    if (isNew && parentId) {
      expandedItems.value[parentId] = true;
    }

    // 设置编辑状态
    editingItem.value = {
      tempId,
      parentId,
      type: itemType,
      depth,
      isNew,
    };

    // 新建时有默认名称，重命名时使用原名称
    if (isNew) {
      editingName.value =
        itemType === "collection"
          ? t("buttons.newCollection")
          : t("buttons.newApi");
    } else {
      editingName.value = findItemById(parentId)?.name || "";
    }

    // 异步保存展开状态（不阻塞）
    if (isNew && parentId) {
      saveExpandState();
    }
  };

  // 通过 ID 查找项
  const findItemById = (id) => {
    const search = (items) => {
      for (const item of items) {
        if (item.id === id) return item;
        if (item.type === "collection" && item.children) {
          const found = search(item.children);
          if (found) return found;
        }
      }
      return null;
    };
    return search(collections.value);
  };

  // 完成编辑（保存）
  const isSavingEdit = ref(false);
  const finishInlineEdit = async () => {
    if (!editingItem.value) return;
    if (isSavingEdit.value) return;
    isSavingEdit.value = true;

    const { parentId, type, isNew } = editingItem.value;
    const name = editingName.value.trim();

    // 名字为空时取消
    if (!name) {
      cancelInlineEdit();
      isSavingEdit.value = false;
      return;
    }

    if (!props.workspace?.id) {
      cancelInlineEdit();
      isSavingEdit.value = false;
      return;
    }

    try {
      if (isNew) {
        // 新建
        if (type === "collection") {
          const newCollection = await invoke("create_collection", {
            workspaceId: props.workspace.id,
            name,
            description: null,
            parentId,
          });

          await loadCollections();
          cancelInlineEdit();

          // 确保父集合展开
          if (parentId) {
            expandedItems.value[parentId] = true;
            await saveExpandState();
          }

          // 选中新创建的集合
          selectCollectionItem(newCollection);

          showToast(t("toast.collectionCreated"), "success");
        } else if (type === "api" || type === "ws") {
          // 新建接口或 WebSocket
          const method = type === "ws" ? "WebSocket" : "GET";
          const newApi = await invoke("create_api", {
            workspaceId: props.workspace.id,
            name,
            method,
            url: "",
            parentId,
          });

          await loadCollections();
          cancelInlineEdit();

          // 确保父集合展开
          if (parentId) {
            expandedItems.value[parentId] = true;
            await saveExpandState();
          }

          // 选中新创建的接口并打开请求面板
          selectApiItem(newApi);

          showToast(t("toast.apiCreated"), "success");
        }
      } else {
        // 重命名
        if (type === "collection") {
          const item = findItemById(parentId);
          if (item) {
            await invoke("update_collection", {
              workspaceId: props.workspace.id,
              id: parentId,
              name,
              description: item.description,
            });
          }
          await loadCollections();
          cancelInlineEdit();
          showToast(t("toast.collectionRenamed"), "success");
        } else if (type === "api" || type === "ws") {
          // API 重命名
          await invoke("update_api", {
            workspaceId: props.workspace.id,
            id: parentId,
            name,
            method: null,
            url: null,
            headers: null,
            body: null,
            bodyType: null,
          });
          // 通知父组件更新 tabs 中的名称
          emit("renameApi", { id: parentId, name });
          await loadCollections();
          cancelInlineEdit();
          showToast(t("toast.apiRenamed"), "success");
        }
      }
    } catch (e) {
      console.error("保存失败:", e);
      showToast(t("toast.saveFailed"), "error");
      cancelInlineEdit();
    } finally {
      isSavingEdit.value = false;
    }
  };

  // 取消编辑
  const cancelInlineEdit = () => {
    editingItem.value = null;
    editingName.value = "";
  };

  // 编辑输入框键盘事件
  const handleEditKeydown = (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      finishInlineEdit();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelInlineEdit();
    }
  };

  // ============ End Inline 编辑功能 ============

  // 递归收集集合下的所有接口 ID
  const collectApiIds = (item) => {
    const ids = [];
    if (item.type === "api") {
      ids.push(item.id);
    } else if (item.type === "collection" && item.children) {
      for (const child of item.children) {
        ids.push(...collectApiIds(child));
      }
    }
    return ids;
  };

  // 删除集合或接口
  const deleteItem = async (item) => {
    if (!props.workspace?.id) return;
    try {
      // 先收集要删除的接口 ID
      const apiIds = collectApiIds(item);

      await invoke("delete_collection_item", {
        workspaceId: props.workspace.id,
        id: item.id,
      });
      await loadCollections();

      // 通知 App 关闭所有相关标签页
      if (apiIds.length > 0) {
        emit("deleteApis", apiIds);
      }

      // 如果删除的是集合，通知关闭集合设置 tab
      if (item.type === "collection") {
        emit("deleteCollection", item.id);
      }
    } catch (e) {
      console.error("删除失败:", e);
    }
  };

  // 获取 HTTP 方法样式类
  const getMethodClass = (method) => method?.toLowerCase() || "";

  // 切换保存响应展开状态（展开时查询响应列表）
  const toggleResponses = async (apiId) => {
    // 如果已经展开，直接折叠
    if (expandedResponses.value[apiId]) {
      expandedResponses.value[apiId] = false;
      return;
    }

    // 如果已经加载过响应列表，直接展开
    if (loadedResponsesApis.value.has(apiId)) {
      expandedResponses.value[apiId] = true;
      return;
    }

    // 第一次展开时，查询该接口的保存响应列表
    try {
      const responses = await invoke("get_api_saved_responses", {
        workspaceId: props.workspace.id,
        apiId: apiId,
      });

      // 更新侧边栏树形数据中对应项的 saved_responses
      updateApiSavedResponses(apiId, responses);

      // 标记已加载
      loadedResponsesApis.value.add(apiId);

      // 展开响应列表
      expandedResponses.value[apiId] = true;
    } catch (e) {
      console.error("加载保存响应失败:", e);
    }
  };

  // 更新侧边栏树形数据中的 saved_responses
  const updateApiSavedResponses = (apiId, responses) => {
    const updateInTree = (items) => {
      for (const item of items) {
        if (item.id === apiId) {
          item.saved_responses = responses || [];
          return true;
        }
        if (item.children && updateInTree(item.children)) {
          return true;
        }
      }
      return false;
    };
    updateInTree(collections.value);
  };

  // 刷新指定 API 的保存响应列表（供外部调用）
  const refreshApiSavedResponses = async (apiId) => {
    try {
      const responses = await invoke("get_api_saved_responses", {
        workspaceId: props.workspace.id,
        apiId: apiId,
      });
      updateApiSavedResponses(apiId, responses);
      loadedResponsesApis.value.add(apiId);

      // 如果当前是展开状态，保持展开；否则不改变
      // 这样用户保存响应后，可以看到新保存的响应
      if (expandedResponses.value[apiId]) {
        // 已经展开，数据已更新，无需额外操作
      } else {
        // 未展开，可以选择自动展开或保持原状
        // 这里选择保持原状，让用户自己决定何时查看
      }
    } catch (e) {
      console.error("刷新保存响应失败:", e);
    }
  };

  // 刷新侧边栏中指定 API 的显示信息（不重新加载整个集合树）
  const refreshApiInSidebar = (apiId, method) => {
    // 找到并更新 collections 中的 API method
    const findAndUpdate = (items) => {
      for (let i = 0; i < items.length; i++) {
        if (items[i].id === apiId) {
          // 使用响应式更新
          items[i] = { ...items[i], method };
          return true;
        }
        if (items[i].children && findAndUpdate(items[i].children)) {
          return true;
        }
      }
      return false;
    };
    findAndUpdate(collections.value);
  };

  // 选择保存响应
  const selectSavedResponse = (response, apiName) => {
    emit("selectSavedResponse", { ...response, apiName });
  };

  // 获取状态码样式类
  const getStatusClass = (status) => {
    if (!status) return "";
    const code = parseInt(status);
    if (code >= 200 && code < 300) return "success";
    if (code >= 400 && code < 500) return "warning";
    if (code >= 500) return "error";
    return "";
  };

  // 格式化时间
  const formatTime = (timestamp) => {
    if (!timestamp) return "";
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now - date;
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return t("time.justNow");
    if (minutes < 60) return t("time.minutesAgo", { n: minutes });
    if (hours < 24) return t("time.hoursAgo", { n: hours });
    if (days < 7) return t("time.daysAgo", { n: days });
    return date.toLocaleDateString();
  };

  // ============ 鼠标事件拖拽排序 ============

  // 通过 ID 查找 DOM 元素
  const getItemEl = (id) => {
    return treeListRef.value?.querySelector(`[data-item-id="${id}"]`);
  };

  const onMouseDown = (e, row) => {
    // 如果有正在编辑的项，先完成编辑
    if (editingItem.value && editingItem.value.tempId !== row.item.id) {
      finishInlineEdit();
    }

    // 只响应左键
    if (e.button !== 0) return;
    dragStartY = e.clientY;
    dragStartId = row.item.id;
    dragStartRow = row;
    dragStartOnName = e.target.closest(".folder-name-text") !== null;
    isDragging = false;
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
    dragListenersAdded = true;
  };

  const onMouseMove = (e) => {
    if (!dragStartId) {
      cleanupDrag();
      return;
    }

    const deltaY = Math.abs(e.clientY - dragStartY);
    if (!isDragging && deltaY < DRAG_THRESHOLD) return;

    if (!isDragging) {
      isDragging = true;
      dragState.value = {
        draggingId: dragStartId,
        dragOverId: null,
        dragPosition: null,
      };
    }

    // 根据 Y 坐标找到目标行
    const target = findRowAtY(e.clientY);
    if (target) {
      if (target.position === "root") {
        // 移动到根级别
        dragState.value = {
          ...dragState.value,
          dragOverId: "root",
          dragPosition: "root",
        };
      } else {
        dragState.value = {
          ...dragState.value,
          dragOverId: target.row.item.id,
          dragPosition: target.position,
        };
      }
    } else {
      dragState.value = {
        ...dragState.value,
        dragOverId: null,
        dragPosition: null,
      };
    }
  };

  const onMouseUp = async (_e) => {
    const wasDragging = isDragging;

    if (isDragging && dragState.value.dragOverId) {
      const { dragPosition, draggingId, dragOverId } = dragState.value;

      if (dragPosition === "root") {
        // 移动到根级别
        await performMoveToRoot();
      } else {
        // 判断是移动到集合还是同级排序
        const targetRow = flatTreeList.value.find(
          (r) => r.item.id === dragOverId,
        );
        const dragRow = flatTreeList.value.find(
          (r) => r.item.id === draggingId,
        );

        if (targetRow?.isCollection && dragPosition === "into") {
          // 移动到集合内
          await performMove();
        } else if (
          dragRow &&
          targetRow &&
          dragRow.depth === targetRow.depth &&
          dragRow.parentId === targetRow.parentId
        ) {
          // 同级排序
          await performReorder();
        }
      }
    }

    // 如果没有拖拽，则是点击 - 集合项切换展开/收起（点击名字区域除外，名字区域由 click 事件处理选择）
    if (
      !wasDragging &&
      dragStartRow?.isCollection &&
      !dragStartRow.isEditing &&
      !dragStartOnName
    ) {
      await toggleExpand(dragStartRow.item.id);
    }

    cleanupDrag();
  };

  const performMoveToRoot = async () => {
    const { draggingId } = dragState.value;
    if (!draggingId) return;

    const dragRow = flatTreeList.value.find((r) => r.item.id === draggingId);
    if (!dragRow) return;

    // 检查是否已经在根级别
    if (dragRow.depth === 0) return;

    try {
      if (dragRow.isCollection) {
        // 移动集合到根级别
        await invoke("move_collection", {
          workspaceId: props.workspace.id,
          collectionId: draggingId,
          targetCollectionId: null,
        });
      } else {
        // 移动 API 到根级别
        await invoke("move_api", {
          workspaceId: props.workspace.id,
          apiId: draggingId,
          targetCollectionId: null,
        });
      }
      await loadCollections();
    } catch (err) {
      console.error("移动到根级别失败:", err);
      showToast(err, "error");
    }
  };

  const performMove = async () => {
    const { draggingId, dragOverId } = dragState.value;
    if (!draggingId || !dragOverId || draggingId === dragOverId) return;

    const dragRow = flatTreeList.value.find((r) => r.item.id === draggingId);
    const targetRow = flatTreeList.value.find((r) => r.item.id === dragOverId);
    if (!dragRow || !targetRow || !targetRow.isCollection) return;

    try {
      if (dragRow.isCollection) {
        // 移动集合
        await invoke("move_collection", {
          workspaceId: props.workspace.id,
          collectionId: draggingId,
          targetCollectionId: targetRow.item.id,
        });
      } else {
        // 移动 API
        await invoke("move_api", {
          workspaceId: props.workspace.id,
          apiId: draggingId,
          targetCollectionId: targetRow.item.id,
        });
      }
      await loadCollections();
    } catch (err) {
      console.error("移动失败:", err);
      showToast(err, "error");
    }
  };

  const performReorder = async () => {
    const { draggingId, dragOverId, dragPosition } = dragState.value;
    if (
      !draggingId ||
      !dragOverId ||
      !dragPosition ||
      draggingId === dragOverId
    )
      return;

    const dragRow = flatTreeList.value.find((r) => r.item.id === draggingId);
    const targetRow = flatTreeList.value.find((r) => r.item.id === dragOverId);
    if (!dragRow || !targetRow) return;

    // 同级判断
    if (
      dragRow.depth !== targetRow.depth ||
      dragRow.parentId !== targetRow.parentId
    )
      return;

    // 计算同一层级的兄弟列表
    const siblings = flatTreeList.value.filter(
      (r) => r.depth === targetRow.depth && r.parentId === targetRow.parentId,
    );
    const targetIndex = siblings.findIndex((r) => r.item.id === dragOverId);
    const newIndex = dragPosition === "before" ? targetIndex : targetIndex + 1;

    try {
      await invoke("reorder_collection_items", {
        workspaceId: props.workspace.id,
        parentId: targetRow.parentId || null,
        itemId: draggingId,
        newIndex,
      });
      await loadCollections();
    } catch (err) {
      console.error("排序失败:", err);
    }
  };

  const cleanupDrag = () => {
    isDragging = false;
    dragStartId = null;
    dragStartRow = null;
    dragStartOnName = false;
    dragState.value = {
      draggingId: null,
      dragOverId: null,
      dragPosition: null,
    };
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
    dragListenersAdded = false;
  };

  // 根据鼠标 Y 坐标找到目标行和位置
  const findRowAtY = (clientY) => {
    if (!treeListRef.value) return null;
    const list = flatTreeList.value;

    // 检查是否在根级别拖放区域（tree-list-footer）
    const footerEl = treeListRef.value.querySelector(".tree-list-footer");
    if (footerEl) {
      const footerRect = footerEl.getBoundingClientRect();
      if (clientY >= footerRect.top && clientY <= footerRect.bottom) {
        return { row: null, position: "root" };
      }
    }

    for (const row of list) {
      // 跳过自身
      if (row.item.id === dragState.value.draggingId) continue;
      // 跳过保存响应区域等非主干行
      if (!row.isCollection && !row.item) continue;

      const el = getItemEl(row.item.id);
      if (!el) continue;

      const rect = el.getBoundingClientRect();
      if (clientY >= rect.top && clientY <= rect.bottom) {
        // 集合项：上半部分 before，下半部分 after，中间区域 into
        if (row.isCollection) {
          const topThird = rect.top + rect.height * 0.25;
          const bottomThird = rect.top + rect.height * 0.75;
          if (clientY < topThird) {
            return { row, position: "before" };
          } else if (clientY > bottomThird) {
            return { row, position: "after" };
          } else {
            return { row, position: "into" };
          }
        } else {
          // API 项：只有 before/after
          const midY = rect.top + rect.height / 2;
          const position = clientY < midY ? "before" : "after";
          return { row, position };
        }
      }
    }
    return null;
  };

  // 计算扁平化的树列表（用于渲染）
  const flatTreeList = computed(() => {
    const result = [];

    const processItems = (items, depth = 0, parentId = null) => {
      for (const item of items) {
        const isCollection = item.type === "collection";
        const hasChildren = item.children && item.children.length > 0;
        const expanded = isExpanded(item.id);

        // 检查这个项是否正在编辑（重命名）
        const isRenaming =
          editingItem.value &&
          !editingItem.value.isNew &&
          editingItem.value.parentId === item.id;

        result.push({
          item,
          isCollection,
          hasChildren,
          expanded,
          depth,
          parentId,
          isEditing: isRenaming,
        });

        // 如果这个集合正在新建子项，在展开子列表前插入临时项
        if (
          isCollection &&
          editingItem.value?.isNew &&
          editingItem.value.parentId === item.id
        ) {
          result.push({
            item: {
              id: editingItem.value.tempId,
              type: editingItem.value.type,
              name: "",
              children: [],
            },
            isCollection: editingItem.value.type === "collection",
            hasChildren: false,
            expanded: false,
            depth: depth + 1,
            parentId: item.id,
            isEditing: true,
          });
        }

        if (hasChildren && expanded) {
          processItems(item.children, depth + 1, item.id);
        }
      }
    };

    processItems(collections.value);

    // 根级别新建项：在列表末尾添加
    if (editingItem.value?.isNew && editingItem.value.parentId === null) {
      result.push({
        item: {
          id: editingItem.value.tempId,
          type: editingItem.value.type,
          name: "",
          children: [],
        },
        isCollection: editingItem.value.type === "collection",
        hasChildren: false,
        expanded: false,
        depth: 0,
        parentId: null,
        isEditing: true,
      });
    }

    return result;
  });

  // 监听工作区变化
  watch(
    () => props.workspace,
    async (ws) => {
      if (ws) {
        await loadCollections();
      }
    },
    { immediate: true },
  );

  // 全局点击关闭右键菜单
  const handleGlobalClick = () => {
    closeContextMenu();
    // 如果有正在编辑的项，完成编辑
    if (editingItem.value) {
      finishInlineEdit();
    }
  };

  onMounted(() => {
    // 全局点击事件
    document.addEventListener("click", handleGlobalClick);
  });

  onUnmounted(() => {
    document.removeEventListener("click", handleGlobalClick);
    // 强制清理拖拽监听器，防止内存泄漏
    if (dragListenersAdded) {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
      dragListenersAdded = false;
    }
  });

  return {
    collections,
    expandedItems,
    selectedApi,
    selectedCollectionId,
    searchQuery,
    editingItem,
    editingName,
    finishInlineEdit,
    cancelInlineEdit,
    handleEditKeydown,
    isSavingEdit,
    contextMenu,
    expandedResponses,
    loadCollections,
    toggleExpand,
    isExpanded,
    selectApiItem,
    selectCollectionItem,
    setSelectedApiId,
    setSelectedCollectionId,
    canCreateSubCollection,
    openContextMenu,
    closeContextMenu,
    openSavedResponseContextMenu,
    handleContextAction,
    deleteItem,
    duplicateItem,
    getMethodClass,
    toggleResponses,
    selectSavedResponse,
    getStatusClass,
    formatTime,
    flatTreeList,
    dragState,
    treeListRef,
    onMouseDown,
    refreshApiSavedResponses,
    refreshApiInSidebar,
  };
}
