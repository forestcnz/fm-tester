import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { showToast } from "../../composables/useToast";
import JSON5 from "json5";
import { mergeCollectionVariablesToObject } from "../../utils/scriptEngine.js";

export function useOrchestrationEditorSetup(props, emit) {
  const { t } = useI18n();

  const orchestration = ref(null);
  const steps = ref([]);
  const collections = ref([]);
  const runHistory = ref([]);
  const isRunning = ref(false);
  const currentRunId = ref(null);
  const currentStepIndex = ref(-1);
  const runProgress = ref([]);

  const loadOrchestration = async () => {
    if (!props.workspaceId || !props.orchestrationId) return;
    try {
      const data = await invoke("get_orchestration", {
        workspaceId: props.workspaceId,
        orchestrationId: props.orchestrationId,
      });
      orchestration.value = data;
      steps.value = data.steps || [];
      await loadCollections();
      await loadRunHistory();
    } catch (e) {
      console.error("加载编排失败:", e);
      showToast(t("toast.orchestrationCreateFailed"), "error");
    }
  };

  const loadCollections = async () => {
    if (!props.workspaceId) return;
    try {
      const data = await invoke("get_collections", {
        workspaceId: props.workspaceId,
      });
      collections.value = flattenCollections(data || []);
    } catch (e) {
      console.error("加载集合失败:", e);
      collections.value = [];
    }
  };

  const flattenCollections = (items, result = []) => {
    for (const item of items) {
      if (item.type === "api") {
        result.push(item);
      }
      if (item.type === "collection" && item.children) {
        flattenCollections(item.children, result);
      }
    }
    return result;
  };

  const loadRunHistory = async () => {
    if (!props.workspaceId || !props.orchestrationId) return;
    try {
      const data = await invoke("get_orchestration_runs", {
        workspaceId: props.workspaceId,
        orchestrationId: props.orchestrationId,
      });
      runHistory.value = data?.runs || [];
    } catch (e) {
      console.error("加载执行历史失败:", e);
      runHistory.value = [];
    }
  };

  const viewRunDetail = async (runId) => {
    if (!props.workspaceId || !props.orchestrationId) return;
    try {
      const data = await invoke("get_orchestration_run", {
        workspaceId: props.workspaceId,
        orchestrationId: props.orchestrationId,
        runId,
      });
      return data;
    } catch (e) {
      console.error("加载运行详情失败:", e);
      return null;
    }
  };

  const deleteRun = async (runId) => {
    if (!props.workspaceId || !props.orchestrationId) return;
    try {
      await invoke("delete_orchestration_run_cmd", {
        workspaceId: props.workspaceId,
        orchestrationId: props.orchestrationId,
        runId,
      });
      runHistory.value = runHistory.value.filter((r) => r.id !== runId);
      showToast(t("toast.runDeleted"), "success");
    } catch (e) {
      console.error("删除执行记录失败:", e);
      showToast(t("toast.runDeleteFailed"), "error");
    }
  };

  const clearAllRuns = async () => {
    if (!props.workspaceId || !props.orchestrationId) return;
    try {
      await invoke("clear_orchestration_runs_cmd", {
        workspaceId: props.workspaceId,
        orchestrationId: props.orchestrationId,
      });
      runHistory.value = [];
      showToast(t("toast.allRunsDeleted"), "success");
    } catch (e) {
      console.error("清空执行记录失败:", e);
      showToast(t("toast.allRunsDeleteFailed"), "error");
    }
  };

  const addStep = async (apiId, stepName = null) => {
    if (!props.workspaceId || !props.orchestrationId) return;
    try {
      const step = await invoke("add_orchestration_step_cmd", {
        workspaceId: props.workspaceId,
        orchestrationId: props.orchestrationId,
        apiId,
        name: stepName,
        enabled: true,
        waitBefore: 0,
        retryCount: 0,
        retryDelay: 1000,
        onFailure: "stop",
      });
      steps.value.push(step);
      emit("stepsChanged");
      showToast(t("toast.stepAdded"), "success");
    } catch (e) {
      console.error("添加步骤失败:", e);
      showToast(t("toast.stepAddFailed"), "error");
    }
  };

  const updateStep = async (stepId, config) => {
    if (!props.workspaceId || !props.orchestrationId) return;
    try {
      await invoke("update_orchestration_step_cmd", {
        workspaceId: props.workspaceId,
        orchestrationId: props.orchestrationId,
        stepId,
        name: config.name,
        enabled: config.enabled,
        waitBefore: config.wait_before,
        retryCount: config.retry_count,
        retryDelay: config.retry_delay,
        onFailure: config.on_failure,
      });
      await loadOrchestration();
      showToast(t("toast.stepUpdated"), "success");
    } catch (e) {
      console.error("更新步骤失败:", e);
      showToast(t("toast.stepUpdateFailed"), "error");
    }
  };

  const removeStep = async (stepId) => {
    if (!props.workspaceId || !props.orchestrationId) return;
    try {
      await invoke("remove_orchestration_step_cmd", {
        workspaceId: props.workspaceId,
        orchestrationId: props.orchestrationId,
        stepId,
      });
      steps.value = steps.value.filter((s) => s.id !== stepId);
      emit("stepsChanged");
      showToast(t("toast.stepRemoved"), "success");
    } catch (e) {
      console.error("删除步骤失败:", e);
      showToast(t("toast.stepRemoveFailed"), "error");
    }
  };

  const reorderSteps = async (newOrder) => {
    if (!props.workspaceId || !props.orchestrationId) return;
    try {
      await invoke("reorder_orchestration_steps_cmd", {
        workspaceId: props.workspaceId,
        orchestrationId: props.orchestrationId,
        stepIds: newOrder,
      });
      const reordered = [];
      for (const id of newOrder) {
        const step = steps.value.find((s) => s.id === id);
        if (step) reordered.push(step);
      }
      steps.value = reordered;
      showToast(t("toast.stepReordered"), "success");
    } catch (e) {
      console.error("排序步骤失败:", e);
      showToast(t("toast.stepReorderFailed"), "error");
    }
  };

  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

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

  const executeStep = async (step, runId, collectionsData, envConfig) => {
    const apiId = step.api_id;
    const apiInfo = getApiInfo(apiId);
    if (!apiInfo) {
      return {
        step_id: step.id,
        api_id: apiId,
        api_name: step.name || "未知接口",
        status: "failed",
        start_time: new Date().toISOString(),
        end_time: new Date().toISOString(),
        response_time: 0,
        status_code: 0,
        response_body: null,
        response_headers: {},
        test_results: [],
        failure_reason: "api_not_found",
        failure_message: "接口不存在",
        retry_count: 0,
        request_method: "",
        request_url: "",
        request_original_url: null,
        request_headers: [],
        request_body: null,
        request_body_type: null,
      };
    }

    const request = {
      method: apiInfo.method || "GET",
      url: apiInfo.url || "",
      params: apiInfo.params || [],
      headers: apiInfo.headers || [],
      body: apiInfo.body || "",
      bodyType: apiInfo.body_type || "raw",
      formData: apiInfo.form_fields || [],
      timeout: apiInfo.timeout,
    };

    const ancestorCollections = findAncestorCollectionsForApi(
      collectionsData,
      apiId,
    );
    const activeEnvVars = await invoke("get_active_variables", {
      workspaceId: props.workspaceId,
    });
    const collVarsObj = mergeCollectionVariablesToObject(ancestorCollections);
    const environmentId = envConfig?.active_environment_id;

    let retryCount = 0;
    const maxRetry = step.retry_count || 0;
    const retryDelay = step.retry_delay || 1000;
    let lastResult = null;

    while (retryCount <= maxRetry) {
      const startTime = new Date();
      let modifiedRequest = request;
      let modifiedEnvVars = activeEnvVars || {};
      let modifiedCollVars = collVarsObj;

      try {
        const preScriptResult = await invoke("execute_pre_scripts_cmd", {
          input: {
            workspace_id: props.workspaceId,
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
            silent: true,
          },
        });

        if (preScriptResult.success) {
          modifiedRequest = preScriptResult.modified_request || request;
          modifiedEnvVars = preScriptResult.modified_environment_vars || {};
          modifiedCollVars = preScriptResult.modified_collection_vars || {};
        }
      } catch (e) {
        console.error(`前置脚本执行失败: ${e}`);
      }

      let bodyToSend = modifiedRequest.body;
      const headersMap = new Map();

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

      for (const h of apiInfo.common_headers || []) {
        if (h.enabled && h.key.trim()) {
          headersMap.set(h.key.toLowerCase(), h);
        }
      }

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
          /* ignore */
        }
      }

      const collectionVariablesArray = Object.entries(modifiedCollVars).map(
        ([key, value]) => ({ key, value, enabled: true }),
      );

      let response;
      let failed = false;
      let failureReason = null;
      let failureMessage = null;

      try {
        const result = await invoke("send_http_request", {
          method: modifiedRequest.method,
          url: modifiedRequest.url,
          headers: headersToSend,
          body: bodyToSend || null,
          bodyType: request.bodyType || null,
          formFields:
            request.formData?.map((f) => ({
              key: f.key,
              value: f.value,
              type: f.type,
              enabled: f.enabled,
              files: f.files,
            })) || null,
          binaryFilePath: null,
          workspaceId: props.workspaceId,
          timeout: request.timeout || null,
          apiId,
          apiName: step.name || apiInfo.name,
          collectionVariables:
            collectionVariablesArray.length > 0
              ? collectionVariablesArray
              : null,
        });

        response = {
          status: result.status,
          statusText: result.status_text,
          headers: result.headers,
          body: result.body,
          time: result.time,
          size: result.size,
        };

        if (result.status >= 400) {
          failed = true;
          failureReason = "http_error";
          failureMessage = `HTTP ${result.status}: ${result.status_text}`;
        }
      } catch (e) {
        failed = true;
        failureReason = "request_failed";
        failureMessage = e.toString();
        response = {
          status: 0,
          statusText: "请求失败",
          headers: {},
          body: e.toString(),
          time: 0,
          size: 0,
        };
      }

      let testResults = [];
      try {
        const postScriptResult = await invoke("execute_post_scripts_cmd", {
          input: {
            workspace_id: props.workspaceId,
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
              status: response.status,
              status_text: response.statusText,
              headers: response.headers,
              body: response.body,
              time: response.time,
              size: response.size,
            },
            silent: true,
          },
        });

        testResults = postScriptResult.test_results || [];

        if (testResults.some((t) => !t.passed)) {
          failed = true;
          failureReason = "assertion_failed";
          const failedTests = testResults.filter((t) => !t.passed);
          failureMessage = failedTests
            .map((t) => t.name || t.error || "断言失败")
            .join(", ");
        }

        if (
          postScriptResult.modified_environment_vars &&
          envConfig?.active_environment_id
        ) {
          const postEnvVars = postScriptResult.modified_environment_vars;
          if (Object.keys(postEnvVars).length > 0) {
            const activeEnv = envConfig.environments.find(
              (e) => e.id === envConfig.active_environment_id,
            );
            if (activeEnv) {
              const updatedVariables = [...(activeEnv.variables || [])];
              for (const [key, value] of Object.entries(postEnvVars)) {
                const existingVar = updatedVariables.find((v) => v.key === key);
                if (existingVar) {
                  existingVar.value = value;
                } else {
                  updatedVariables.push({ key, value, enabled: true });
                }
              }
              await invoke("save_environment", {
                workspaceId: props.workspaceId,
                environment: { ...activeEnv, variables: updatedVariables },
              });
            }
          }
        }
      } catch (e) {
        console.error(`后置脚本执行失败: ${e}`);
      }

      const endTime = new Date();
      lastResult = {
        step_id: step.id,
        api_id: apiId,
        api_name: step.name || apiInfo.name,
        status: failed ? "failed" : "success",
        start_time: startTime.toISOString(),
        end_time: endTime.toISOString(),
        response_time: response.time || 0,
        status_code: response.status,
        response_body: response.body,
        response_headers: response.headers,
        test_results: testResults,
        failure_reason: failureReason,
        failure_message: failureMessage,
        retry_count: retryCount,
        request_method: modifiedRequest.method,
        request_url: modifiedRequest.url,
        request_original_url: apiInfo.url,
        request_headers: headersToSend,
        request_body: bodyToSend,
        request_body_type: request.bodyType,
      };

      await invoke("update_orchestration_run_step_cmd", {
        workspaceId: props.workspaceId,
        orchestrationId: props.orchestrationId,
        runId,
        stepResult: lastResult,
      });

      if (!failed || retryCount >= maxRetry) {
        break;
      }

      retryCount++;
      console.log(`第 ${retryCount} 次重试...`);
      await sleep(retryDelay);
    }

    return lastResult;
  };

  const runOrchestration = async () => {
    if (!props.workspaceId || !props.orchestrationId) return;
    // 使用双重检查防止重复调用（快速双击场景）
    if (isRunning.value) return;

    // 立即锁定状态，防止竞态条件
    isRunning.value = true;

    if (steps.value.length === 0) {
      isRunning.value = false;
      showToast(t("orchestration.noSteps"), "warning");
      return;
    }
    currentStepIndex.value = -1;
    runProgress.value = [];

    try {
      const run = await invoke("create_orchestration_run_cmd", {
        workspaceId: props.workspaceId,
        orchestrationId: props.orchestrationId,
      });
      currentRunId.value = run.id;

      const startTime = new Date();
      const collectionsData = await invoke("get_collections", {
        workspaceId: props.workspaceId,
      });
      const envConfig = await invoke("get_environments", {
        workspaceId: props.workspaceId,
      });

      let successCount = 0;
      let failedCount = 0;
      let skippedCount = 0;
      let shouldStop = false;

      for (let i = 0; i < steps.value.length; i++) {
        const step = steps.value[i];
        currentStepIndex.value = i;

        if (!step.enabled) {
          skippedCount++;
          runProgress.value.push({
            step_id: step.id,
            status: "skipped",
          });
          continue;
        }

        if (shouldStop) {
          skippedCount++;
          runProgress.value.push({
            step_id: step.id,
            status: "skipped",
            reason: "previous_failure",
          });
          continue;
        }

        if (step.wait_before > 0) {
          console.log(`等待 ${step.wait_before}ms...`);
          await sleep(step.wait_before);
        }

        const result = await executeStep(
          step,
          run.id,
          collectionsData,
          envConfig,
        );

        runProgress.value.push(result);

        if (result.status === "success") {
          successCount++;
        } else {
          failedCount++;
          if (step.on_failure === "stop") {
            shouldStop = true;
          }
        }
      }

      const endTime = new Date();
      const totalTime = endTime.getTime() - startTime.getTime();

      await invoke("complete_orchestration_run_cmd", {
        workspaceId: props.workspaceId,
        orchestrationId: props.orchestrationId,
        runId: run.id,
        status: shouldStop ? "stopped" : failedCount > 0 ? "failed" : "success",
        endTime: endTime.toISOString(),
        totalTime,
      });

      await loadRunHistory();

      const statusText = shouldStop
        ? t("orchestration.runStatus.stopped")
        : failedCount > 0
          ? t("orchestration.runStatus.failed")
          : t("orchestration.runStatus.success");

      showToast(
        `${statusText}: ${successCount} ${t("tests.passed")}, ${failedCount} ${t("tests.failed")}, ${skippedCount} ${t("orchestration.stepStatus.skipped")}`,
        shouldStop || failedCount > 0 ? "warning" : "success",
      );
    } catch (e) {
      console.error("执行编排失败:", e);
      showToast(t("toast.orchestrationCreateFailed"), "error");
    } finally {
      isRunning.value = false;
      currentStepIndex.value = -1;
      currentRunId.value = null;
    }
  };

  const stopOrchestration = async () => {
    isRunning.value = false;
    currentStepIndex.value = -1;
  };

  const getApiInfo = (apiId) => {
    return collections.value.find((api) => api.id === apiId);
  };

  watch(
    () => props.orchestrationId,
    async (newId) => {
      if (newId) {
        await loadOrchestration();
      } else {
        orchestration.value = null;
        steps.value = [];
        runHistory.value = [];
        collections.value = [];
      }
    },
    { immediate: true },
  );

  return {
    orchestration,
    steps,
    collections,
    runHistory,
    isRunning,
    currentRunId,
    currentStepIndex,
    runProgress,
    loadOrchestration,
    loadCollections,
    loadRunHistory,
    viewRunDetail,
    deleteRun,
    clearAllRuns,
    addStep,
    updateStep,
    removeStep,
    reorderSteps,
    runOrchestration,
    stopOrchestration,
    getApiInfo,
  };
}
