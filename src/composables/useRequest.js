import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useI18n } from "vue-i18n";
import JSON5 from "json5";
import { mergeCollectionVariablesToObject } from "../utils/scriptEngine.js";
import { buildUrlWithParams } from "../utils/collectionTree.js";
import { showToast } from "./useToast.js";

/**
 * HTTP请求管理 composable（包含Console日志）
 * @param {Ref} currentWorkspace - 当前工作区引用
 * @param {Ref} tabs - 标签页列表引用
 * @param {Ref} activeTab - 当前激活标签页引用
 * @param {Ref} sidebarRef - 侧边栏组件引用
 * @param {Ref} requestTabs - 请求子标签页状态引用
 * @param {Ref} currentRequestTab - 当前请求子标签页引用
 * @param {Function} updateCurrentRequest - 更新当前请求函数
 * @param {Function} saveOpenTabs - 保存标签页函数
 * @param {Object} currentRequest - 当前请求状态（外部传入）
 * @param {Ref} response - 响应数据引用（外部传入）
 * @param {Ref} loading - 加载状态引用（外部传入）
 */
export function useRequest(
  currentWorkspace,
  tabs,
  activeTab,
  sidebarRef,
  requestTabs,
  currentRequestTab,
  updateCurrentRequest,
  saveOpenTabs,
  currentRequest,
  response,
  loading,
) {
  const { t } = useI18n();

  // 发送请求时的 tab ID
  const sendingTabId = ref(null);

  // 测试结果
  const testResults = ref([]);

  // Console 日志
  const showConsolePanel = ref(false);
  const consoleLogs = ref([]);
  const maxConsoleLogs = 100;

  // SSE 状态
  const MAX_SSE_EVENTS = 2000;
  const isSseMode = ref(false);
  const sseUrl = ref("");
  const sseConnected = ref(false);
  const sseEvents = ref([]);
  let sseTotalBytes = 0;
  let sseStartTime = null;
  let sseDurationTimer = null;

  let unlistenHttpLog = null;
  let unlistenSseEvent = null;
  let unlistenSseState = null;
  let unlistenSseResponseInfo = null;
  let unlistenScriptLog = null;
  let unlistenWsLog = null;

  const openConsolePanel = () => {
    showConsolePanel.value = !showConsolePanel.value;
  };

  const closeConsolePanel = () => {
    showConsolePanel.value = false;
  };

  const clearConsoleLogs = () => {
    consoleLogs.value = [];
  };

  const addConsoleLog = (type, message) => {
    const now = new Date();
    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, "0");
    const day = String(now.getDate()).padStart(2, "0");
    const hour = String(now.getHours()).padStart(2, "0");
    const minute = String(now.getMinutes()).padStart(2, "0");
    const second = String(now.getSeconds()).padStart(2, "0");
    const time = `${year}-${month}-${day} ${hour}:${minute}:${second}`;
    consoleLogs.value.push({ type, message, time });
    if (consoleLogs.value.length > maxConsoleLogs) {
      consoleLogs.value.shift();
    }
  };

  const setupHttpLogListener = async () => {
    unlistenHttpLog = await listen("http-log", (event) => {
      const log = event.payload;
      const logType =
        log.log_type === "error"
          ? "error"
          : log.log_type === "response"
            ? "info"
            : "log";
      const message = JSON.stringify(log, null, 2);
      const timestamp = log.timestamp;
      consoleLogs.value.push({ type: logType, message, time: timestamp });
      if (consoleLogs.value.length > maxConsoleLogs) {
        consoleLogs.value.shift();
      }
    });

    // 监听 SSE 响应信息（包含状态和响应头）
    unlistenSseResponseInfo = await listen("sse-response-info", (event) => {
      const info = event.payload;
      // 创建或更新 response 对象
      response.value = {
        status: info.status,
        statusText: info.statusText,
        headers: info.headers,
        body: response.value?.body || "", // 保留已累积的 body
        time: response.value?.time || 0,
        size: response.value?.size || 0,
        resolvedUrl: info.resolvedUrl,
        resolvedHeaders: [],
      };
    });

    // 监听脚本日志
    unlistenScriptLog = await listen("script-log", (event) => {
      const log = event.payload;
      consoleLogs.value.push({
        type: log.type,
        message: log.message,
        time: log.time,
        level: log.level,
      });
      if (consoleLogs.value.length > maxConsoleLogs) {
        consoleLogs.value.shift();
      }
    });

    // 监听 WebSocket 日志
    unlistenWsLog = await listen("ws-log", (event) => {
      const log = event.payload;
      const logType =
        log.log_type === "error"
          ? "error"
          : log.log_type === "success"
            ? "info"
            : log.log_type === "warn"
              ? "warn"
              : "log";
      consoleLogs.value.push({
        type: logType,
        message: log.message,
        time: log.timestamp,
      });
      if (consoleLogs.value.length > maxConsoleLogs) {
        consoleLogs.value.shift();
      }
    });

    // 监听 SSE 事件
    unlistenSseEvent = await listen("sse-event", (event) => {
      const sseEvent = event.payload;
      const eventTime = new Date().toISOString();
      const data = sseEvent.data ?? "";

      sseEvents.value.push({ time: eventTime, data });
      if (sseEvents.value.length > MAX_SSE_EVENTS) {
        sseEvents.value.shift();
      }
      sseTotalBytes += data.length;

      if (response.value) {
        response.value.size = sseTotalBytes;
      } else {
        response.value = {
          status: 200,
          statusText: "SSE Stream",
          headers: {},
          body: "",
          time: 0,
          size: sseTotalBytes,
          resolvedUrl: sseUrl.value || "",
          resolvedHeaders: [],
        };
      }

      if (data === "[DONE]") {
        if (sseDurationTimer) {
          clearInterval(sseDurationTimer);
          sseDurationTimer = null;
        }
        sseStartTime = null;
        sseConnected.value = false;
      }
    });

    // 监听 SSE 状态
    unlistenSseState = await listen("sse-state", (event) => {
      const state = event.payload;

      if (state.status === "Connected") {
        sseConnected.value = true;
        loading.value = false;
        // 开始计时
        sseStartTime = Date.now();
        if (sseDurationTimer) {
          clearInterval(sseDurationTimer);
        }
        sseDurationTimer = setInterval(() => {
          if (response.value && sseStartTime) {
            response.value.time = Date.now() - sseStartTime;
          }
        }, 1000);
      }
      if (state.status === "Disconnected" || state.status === "Error") {
        loading.value = false;
        sseConnected.value = false;
        isSseMode.value = false; // ← 恢复普通模式
        // 停止计时
        if (sseDurationTimer) {
          clearInterval(sseDurationTimer);
          sseDurationTimer = null;
        }
        sseStartTime = null;
        if (state.error) {
          console.error("[SSE] 错误:", state.error);
        }
      }
    });
  };

  const cleanupHttpLogListener = () => {
    if (unlistenHttpLog) {
      unlistenHttpLog();
    }
    if (unlistenSseEvent) {
      unlistenSseEvent();
    }
    if (unlistenSseState) {
      unlistenSseState();
    }
    if (unlistenSseResponseInfo) {
      unlistenSseResponseInfo();
    }
    if (unlistenScriptLog) {
      unlistenScriptLog();
    }
    if (unlistenWsLog) {
      unlistenWsLog();
    }
    // 清理 SSE 定时器
    if (sseDurationTimer) {
      clearInterval(sseDurationTimer);
      sseDurationTimer = null;
    }
    sseStartTime = null;
  };

  const selectApi = async (apiOrId) => {
    loading.value = false;
    sseEvents.value = [];
    sseTotalBytes = 0;

    // 支持传入 api 对象或 apiId 字符串
    let api;
    let apiId;

    if (typeof apiOrId === "string") {
      apiId = apiOrId;
      // 从已打开的 tabs 中找到 api 数据
      const existingTab = tabs.value.find(
        (t) => t.id === apiId && t.tabType === "api",
      );
      if (existingTab) {
        api = existingTab;
      } else {
        // 如果 tab 不存在，只通知侧边栏展开父集合
        if (sidebarRef.value) {
          sidebarRef.value.setSelectedApi(apiId);
        }
        return;
      }
    } else {
      api = apiOrId;
      apiId = api.id;
    }

    const existingIndex = tabs.value.findIndex(
      (t) => t.id === apiId && t.tabType === "api",
    );

    if (existingIndex >= 0) {
      tabs.value[existingIndex].params = api.params || [];
      tabs.value[existingIndex].commonHeaders = api.commonHeaders || [];
      tabs.value[existingIndex].collectionVariables =
        api.collectionVariables || [];
      tabs.value[existingIndex].timeout = api.timeout;
      activeTab.value = existingIndex;
    } else {
      tabs.value.push({
        id: apiId,
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
        commonHeaders: api.commonHeaders || [],
        collectionVariables: api.collectionVariables || [],
        timeout: api.timeout,
      });
      activeTab.value = tabs.value.length - 1;
    }

    // 通知侧边栏展开父集合并选中 API
    if (sidebarRef.value) {
      sidebarRef.value.setSelectedApi(apiId);
    }

    updateCurrentRequest();
    await saveOpenTabs();
  };

  const updateRequest = (newRequest) => {
    currentRequest.method = newRequest.method;
    currentRequest.url = newRequest.url;
    currentRequest.params = newRequest.params;
    currentRequest.headers = newRequest.headers;
    currentRequest.body = newRequest.body;
    currentRequest.bodyType = newRequest.bodyType;
    currentRequest.formData = newRequest.formData;
    currentRequest.binaryFile = newRequest.binaryFile;
    currentRequest.timeout = newRequest.timeout;
  };

  const sendRequest = async (request) => {
    loading.value = true;
    response.value = null;
    testResults.value = [];
    sseEvents.value = [];
    sseTotalBytes = 0;
    isSseMode.value = false;

    const sendTabIndex = activeTab.value;
    const sendTabId = tabs.value[sendTabIndex]?.id;
    sendingTabId.value = sendTabId;

    const sendTab = tabs.value[sendTabIndex];
    if (sendTab && sendTab.tabType === "api") {
      sendTab.lastResponseData = null;
    }

    const apiId = sendTab?.tabType === "api" ? sendTab?.id : null;
    const apiName = sendTab?.tabType === "api" ? sendTab?.name : null;
    const workspaceId = currentWorkspace.value?.id;

    try {
      // ========== 自动保存（设置开启时，发送前静默保存当前编辑） ==========
      try {
        const autoSaveSettings = await invoke("get_settings");
        if (autoSaveSettings?.behavior?.auto_save_on_send) {
          await saveRequest(request, { silent: true });
        }
      } catch (e) {
        console.warn("自动保存失败:", e);
      }

      // ========== 前置脚本执行 ==========
      let modifiedRequest = request;
      let modifiedEnvVars = {};
      let modifiedCollVars = {};
      let ancestorCollections = [];
      let collectionsData = [];
      let envConfig = null;

      if (apiId && workspaceId) {
        // 获取集合数据
        collectionsData = await invoke("get_collections", { workspaceId });

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

        ancestorCollections = findAncestorCollectionsForApi(
          collectionsData,
          apiId,
        );

        // 获取当前环境变量和环境配置
        envConfig = await invoke("get_environments", { workspaceId });
        const activeEnvVars = await invoke("get_active_variables", {
          workspaceId,
        });
        const collVarsObj =
          mergeCollectionVariablesToObject(ancestorCollections);

        // 获取当前激活环境的 ID
        const environmentId = envConfig.active_environment_id;

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
              headers: [...request.headers],
              params: [...request.params],
              body: request.body,
            },
            silent: false,
          },
        });

        if (!preScriptResult.success) {
          // 前置脚本失败，记录错误日志，但不中断请求
          addConsoleLog("error", `前置脚本执行失败: ${preScriptResult.error}`);
          // 使用原始请求参数继续发送
          modifiedRequest = request;
          modifiedEnvVars = activeEnvVars || {};
          modifiedCollVars = collVarsObj;
        } else {
          // 使用脚本修改后的请求参数
          modifiedRequest = preScriptResult.modified_request || request;
          modifiedEnvVars = preScriptResult.modified_environment_vars || {};
          modifiedCollVars = preScriptResult.modified_collection_vars || {};

          // 保存脚本修改的目标环境变量（环境脚本操作自己的环境）
          if (
            preScriptResult.target_environment_id &&
            preScriptResult.modified_target_environment_vars
          ) {
            const targetEnvId = preScriptResult.target_environment_id;
            const targetEnvVars =
              preScriptResult.modified_target_environment_vars;
            const targetEnv = envConfig.environments.find(
              (e) => e.id === targetEnvId,
            );
            if (targetEnv && Object.keys(targetEnvVars).length > 0) {
              // 更新目标环境变量（深拷贝元素，避免直接修改 envConfig 缓存中的原对象）
              const updatedVariables = (targetEnv.variables || []).map((v) => ({
                ...v,
              }));
              for (const [key, value] of Object.entries(targetEnvVars)) {
                const existingVar = updatedVariables.find((v) => v.key === key);
                if (existingVar) {
                  existingVar.value = value;
                } else {
                  updatedVariables.push({ key, value, enabled: true });
                }
              }
              await invoke("save_environment", {
                workspaceId,
                environment: { ...targetEnv, variables: updatedVariables },
              });
            }
          }

          // 保存脚本修改的目标集合变量（集合脚本/API脚本操作对应集合）
          if (
            preScriptResult.target_collection_id &&
            preScriptResult.modified_target_collection_vars
          ) {
            const targetCollId = preScriptResult.target_collection_id;
            const targetCollVars =
              preScriptResult.modified_target_collection_vars;
            if (Object.keys(targetCollVars).length > 0) {
              // 更新目标集合变量
              const targetCollection = ancestorCollections.find(
                (c) => c.id === targetCollId,
              );
              if (targetCollection) {
                const existingVars =
                  targetCollection.collection_variables || [];
                // 深拷贝元素，避免直接修改源数据
                const updatedVariables = existingVars.map((v) => ({ ...v }));
                for (const [key, value] of Object.entries(targetCollVars)) {
                  const existingVar = updatedVariables.find(
                    (v) => v.key === key,
                  );
                  if (existingVar) {
                    existingVar.value = value;
                  } else {
                    updatedVariables.push({ key, value, enabled: true });
                  }
                }
                // 转换为数组格式保存
                const varsArray = updatedVariables.map((v) => ({
                  key: v.key,
                  value: v.value,
                  enabled: v.enabled !== false,
                  description: v.description || null,
                }));
                await invoke("update_collection_settings", {
                  workspaceId,
                  id: targetCollId,
                  collectionVariables: varsArray,
                });
              }
            }
          }
        }
      }

      // ========== HTTP 请求发送 ==========
      let bodyToSend = modifiedRequest.body;

      const commonHeaders = sendTab?.commonHeaders || [];

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
      for (const h of commonHeaders) {
        if (h.enabled && h.key.trim()) {
          headersMap.set(h.key.toLowerCase(), h);
        }
      }

      // 合并接口请求头（覆盖集合和环境请求头）
      for (const h of modifiedRequest.headers || []) {
        if (h.enabled && h.key.trim()) {
          headersMap.set(h.key.toLowerCase(), h);
        }
      }

      const headersToSend = Array.from(headersMap.values());

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
          } catch { /* ignore */ }
        }
      }

      const formFields =
        request.formData?.map((field) => ({
          key: field.key,
          value: field.value,
          type: field.type,
          enabled: field.enabled,
          files: field.files,
        })) || null;

      const binaryFilePath = request.binaryFile?.path || null;

      // 构建集合变量数组（用于后端变量替换）
      const collectionVariablesArray = Object.entries(modifiedCollVars).map(
        ([key, value]) => ({
          key,
          value,
          enabled: true,
        }),
      );

      // 将 params 合并到 URL 中（脚本可能修改了 params）
      const urlWithParams = buildUrlWithParams(
        modifiedRequest.url,
        modifiedRequest.params,
      );

      const result = await invoke("send_http_request", {
        method: modifiedRequest.method,
        url: urlWithParams,
        headers: headersToSend,
        body: bodyToSend || null,
        bodyType: request.bodyType || null,
        formFields: formFields,
        binaryFilePath: binaryFilePath,
        workspaceId: workspaceId,
        timeout: request.timeout || null,
        apiId: apiId,
        apiName: apiName,
        collectionVariables:
          collectionVariablesArray.length > 0 ? collectionVariablesArray : null,
      }).catch((error) => {
        // 检测 SSE 流（后端返回 "SSE_STREAM" 错误）
        if (error === "SSE_STREAM" || error.message === "SSE_STREAM") {
          isSseMode.value = true;
          sseUrl.value = modifiedRequest.url;
          // SSE 流：response 通过事件接收，不清空已有 response
          if (!response.value) {
            response.value = null;
          }
          loading.value = false; // ← 连接后设置 loading = false
          return null; // SSE 流不返回响应
        }
        throw error; // 其他错误继续抛出
      });

      // SSE 流已处理，直接返回
      if (result === null) {
        return;
      }

      const responseData = {
        status: result.status,
        statusText: result.status_text,
        headers: result.headers,
        body: result.body,
        time: result.time,
        size: result.size,
        resolvedUrl: result.resolved_url,
        resolvedHeaders: result.resolved_headers,
        avgTime: result.avg_time,
        timing: result.timing,
      };

      // ========== 后置脚本执行 ==========
      if (apiId && workspaceId) {
        const postScriptResult = await invoke("execute_post_scripts_cmd", {
          input: {
            workspace_id: workspaceId,
            api_id: apiId,
            environment_id: envConfig?.active_environment_id,
            ancestor_collections: ancestorCollections.map((c) => ({
              id: c.id,
              name: c.name,
              collection_variables: c.collection_variables || [],
            })),
            environment_variables: modifiedEnvVars,
            collection_variables: modifiedCollVars,
            request: {
              url: modifiedRequest.url,
              method: modifiedRequest.method,
              headers: modifiedRequest.headers || [],
              params: modifiedRequest.params || [],
              body: modifiedRequest.body,
            },
            response: {
              status: responseData.status,
              status_text: responseData.statusText,
              headers: responseData.headers,
              body: responseData.body,
              time: responseData.time,
              size: responseData.size,
            },
            silent: false,
          },
        });

        // 保存测试结果
        if (
          postScriptResult.test_results &&
          postScriptResult.test_results.length > 0
        ) {
          testResults.value = postScriptResult.test_results;
        }

        if (!postScriptResult.success) {
          addConsoleLog(
            "error",
            `后置脚本执行有错误: ${postScriptResult.error || "未知错误"}`,
          );
        }

        // 保存后置脚本修改的目标环境变量（环境脚本操作自己的环境）
        if (
          postScriptResult.target_environment_id &&
          postScriptResult.modified_target_environment_vars
        ) {
          const targetEnvId = postScriptResult.target_environment_id;
          const targetEnvVars =
            postScriptResult.modified_target_environment_vars;
          const targetEnv = envConfig.environments.find(
            (e) => e.id === targetEnvId,
          );
          if (targetEnv && Object.keys(targetEnvVars).length > 0) {
            // 更新目标环境变量（深拷贝元素，避免直接修改 envConfig 缓存中的原对象）
            const updatedVariables = (targetEnv.variables || []).map((v) => ({
              ...v,
            }));
            for (const [key, value] of Object.entries(targetEnvVars)) {
              const existingVar = updatedVariables.find((v) => v.key === key);
              if (existingVar) {
                existingVar.value = value;
              } else {
                updatedVariables.push({ key, value, enabled: true });
              }
            }
            await invoke("save_environment", {
              workspaceId,
              environment: { ...targetEnv, variables: updatedVariables },
            });
          }
        }

        // 保存后置脚本修改的目标集合变量（集合脚本/API脚本操作对应集合）
        if (
          postScriptResult.target_collection_id &&
          postScriptResult.modified_target_collection_vars
        ) {
          const targetCollId = postScriptResult.target_collection_id;
          const targetCollVars =
            postScriptResult.modified_target_collection_vars;
          if (Object.keys(targetCollVars).length > 0) {
            // 更新目标集合变量
            const targetCollection = ancestorCollections.find(
              (c) => c.id === targetCollId,
            );
            if (targetCollection) {
              const existingVars = targetCollection.collection_variables || [];
              // 深拷贝元素，避免直接修改源数据
              const updatedVariables = existingVars.map((v) => ({ ...v }));
              for (const [key, value] of Object.entries(targetCollVars)) {
                const existingVar = updatedVariables.find((v) => v.key === key);
                if (existingVar) {
                  existingVar.value = value;
                } else {
                  updatedVariables.push({ key, value, enabled: true });
                }
              }
              // 转换为数组格式保存
              const varsArray = updatedVariables.map((v) => ({
                key: v.key,
                value: v.value,
                enabled: v.enabled !== false,
                description: v.description || null,
              }));
              await invoke("update_collection_settings", {
                workspaceId,
                id: targetCollId,
                collectionVariables: varsArray,
              });
            }
          }
        }
      }

      // ========== 更新响应数据 ==========
      const targetTab = tabs.value.find((t) => t.id === sendingTabId.value);
      if (targetTab && targetTab.tabType === "api") {
        targetTab.lastResponseData = responseData;
        targetTab.lastTestResults = [...testResults.value];
      }

      if (sendingTabId.value === tabs.value[activeTab.value]?.id) {
        response.value = responseData;
      }
    } catch (error) {
      addConsoleLog("error", `请求失败: ${error}`);

      const errorResponse = {
        status: 0,
        statusText: "请求失败",
        headers: {},
        body: `错误: ${error}`,
        time: 0,
        size: 0,
      };

      const targetTab = tabs.value.find((t) => t.id === sendingTabId.value);
      if (targetTab && targetTab.tabType === "api") {
        targetTab.lastResponseData = errorResponse;
      }

      if (sendingTabId.value === tabs.value[activeTab.value]?.id) {
        response.value = errorResponse;
      }
    } finally {
      loading.value = false;
      sendingTabId.value = null;
    }
  };

  const saveRequest = async (request, { silent = false } = {}) => {
    if (!currentWorkspace.value?.id) return;
    if (tabs.value.length === 0) return;

    const currentTab = tabs.value[activeTab.value];
    if (!currentTab) return;
    if (currentTab.tabType !== "api") {
      if (!silent) showToast(t("toast.wsSaveInPanel"), "warning");
      return;
    }

    const formFields =
      request.formData?.map((field) => ({
        key: field.key,
        value: field.value,
        type: field.type,
        enabled: field.enabled,
        files: field.files,
      })) || null;

    const binaryFilePath = request.binaryFile?.path || null;

    try {
      await invoke("update_api", {
        workspaceId: currentWorkspace.value?.id,
        id: currentTab.id,
        name: currentTab.name,
        method: request.method,
        url: request.url,
        params: request.params,
        headers: request.headers,
        body: request.body,
        bodyType: request.bodyType,
        formFields: formFields,
        binaryFilePath: binaryFilePath,
      });

      // method 变化时只更新侧边栏显示，不刷新整个集合树（不影响已保存的响应）
      const oldMethod = currentTab.method;
      currentTab.method = request.method;
      currentTab.url = request.url;
      currentTab.params = request.params;
      currentTab.headers = request.headers;
      currentTab.body = request.body;
      currentTab.bodyType = request.bodyType;

      if (sidebarRef.value && request.method !== oldMethod) {
        sidebarRef.value.refreshApiInSidebar?.(currentTab.id, request.method);
      }
      if (!silent) showToast(t("toast.apiSaved"), "success");
    } catch (e) {
      console.error("保存失败:", e);
    }
  };

  const onRenameApi = async (data) => {
    // data 可能是对象 { id, name } 或者两个参数 (apiId, newName)
    const apiId = typeof data === "object" ? data.id : data;
    const newName = typeof data === "object" ? data.name : arguments[1];

    if (!currentWorkspace.value?.id) return;

    try {
      await invoke("update_api", {
        workspaceId: currentWorkspace.value?.id,
        id: apiId,
        name: newName,
      });

      const tabIndex = tabs.value.findIndex(
        (t) => t.id === apiId && t.tabType === "api",
      );
      if (tabIndex >= 0) {
        tabs.value[tabIndex].name = newName;
      }

      sidebarRef.value?.loadCollections();
    } catch (e) {
      console.error("重命名失败:", e);
    }
  };

  // 监听 activeTab 变化
  const setupActiveTabWatcher = () => {
    watch(activeTab, async () => {
      loading.value = false;

      updateCurrentRequest();
      const currentTab = tabs.value[activeTab.value];
      if (currentTab?.id) {
        if (currentTab.tabType === "api" && sidebarRef.value) {
          sidebarRef.value.setSelectedApi(currentTab.id);
        }
        currentRequestTab.value =
          requestTabs.value[currentTab.id] ||
          (currentTab.method?.toUpperCase() === "POST" ? "body" : "params");
        // 恢复该 tab 的校验结果
        if (currentTab.savedResponseData?.testResults) {
          testResults.value = currentTab.savedResponseData.testResults;
        } else if (currentTab.lastTestResults) {
          testResults.value = currentTab.lastTestResults;
        } else {
          testResults.value = [];
        }
      }
      await saveOpenTabs();
    });
  };

  // ========== SSE 请求函数 ==========
  const startSse = async (request) => {
    loading.value = true;
    isSseMode.value = true;
    sseUrl.value = request.url;
    sseConnected.value = false;
    sseEvents.value = [];
    sseTotalBytes = 0;
    response.value = null; // 清空响应

    try {
      // 获取全局超时设置
      let globalTimeout = 120000; // 默认 2 分钟
      try {
        const settings = await invoke("get_settings");
        if (settings?.request_timeout) {
          globalTimeout = settings.request_timeout * 1000; // 秒转毫秒
        }
      } catch (e) {
        console.warn("[SSE] 获取全局设置失败，使用默认超时:", e);
      }

      // 获取 headers
      const headersMap = new Map();
      for (const h of request.headers || []) {
        if (h.enabled && h.key.trim()) {
          headersMap.set(h.key.toLowerCase(), h);
        }
      }
      const headersToSend = Array.from(headersMap.values());

      // 开始 SSE 连接（支持 POST + body）
      const timeoutMs = request.timeout || globalTimeout;
      await invoke("start_sse_cmd", {
        method: request.method || "GET",
        url: request.url,
        headers: headersToSend,
        body: request.body || null,
        timeoutMs: timeoutMs,
        lastEventId: null,
      });
    } catch (err) {
      loading.value = false;
      isSseMode.value = false;
      console.error("[SSE] 连接失败:", err);
      response.value = {
        status: 500,
        statusText: "SSE Error",
        headers: {},
        body: `SSE 连接失败: ${err}`,
        time: 0,
        size: 0,
        resolvedUrl: request.url || "",
        resolvedHeaders: [],
      };
    }
  };

  const stopSse = async () => {
    try {
      await invoke("stop_sse_cmd");
      sseConnected.value = false;
      // 停止计时
      if (sseDurationTimer) {
        clearInterval(sseDurationTimer);
        sseDurationTimer = null;
      }
      sseStartTime = null;
    } catch (err) {
      console.error("[SSE] 停止失败:", err);
    }
  };

  return {
    currentRequest,
    response,
    loading,
    testResults,
    sendingTabId,
    showConsolePanel,
    consoleLogs,
    openConsolePanel,
    closeConsolePanel,
    clearConsoleLogs,
    addConsoleLog,
    setupHttpLogListener,
    cleanupHttpLogListener,
    selectApi,
    updateRequest,
    sendRequest,
    saveRequest,
    onRenameApi,
    setupActiveTabWatcher,
    // SSE 相关
    isSseMode,
    sseUrl,
    sseConnected,
    sseEvents,
    startSse,
    stopSse,
  };
}
