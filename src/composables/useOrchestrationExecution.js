import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { showToast } from "./useToast";
import JSON5 from "json5";
import { mergeCollectionVariablesToObject } from "../utils/scriptEngine.js";
import {
  buildUrlWithParams,
  findApiInCollections,
  findAncestorCollectionsForApi,
} from "../utils/collectionTree.js";

// 每个编排独立的运行状态（按 ID 区分）
const runningOrchestrations = new Map();

/**
 * 编排执行服务（可在全局调用）
 */
export function useOrchestrationExecution() {
  const { t } = useI18n();
  const currentRunId = ref(null);

  /**
   * 执行编排（自动触发时使用）
   * @param {string} workspaceId - 工作区路径
   * @param {string} orchestrationId - 编排 ID
   */
  const executeOrchestration = async (workspaceId, orchestrationId) => {
    // 检查参数
    if (!workspaceId || !orchestrationId) return;

    // 检查该编排是否正在执行（按 ID 区分，不同编排可并发）
    if (runningOrchestrations.has(orchestrationId)) {
      return;
    }

    // 标记该编排正在执行
    runningOrchestrations.set(orchestrationId, true);

    try {
      // 加载编排数据
      const orchestration = await invoke("get_orchestration", {
        workspaceId,
        orchestrationId,
      });

      const steps = orchestration.steps || [];
      if (steps.length === 0) {
        currentRunId.value = null;
        return;
      }

      // 创建执行记录
      const run = await invoke("create_orchestration_run_cmd", {
        workspaceId,
        orchestrationId,
      });
      currentRunId.value = run.id;

      const startTime = new Date();
      const collectionsData = await invoke("get_collections", {
        workspaceId,
      });
      const envConfig = await invoke("get_environments", {
        workspaceId,
      });

      // 静默执行，不需要日志器

      let successCount = 0;
      let failedCount = 0;
      let shouldStop = false;

      for (let i = 0; i < steps.length; i++) {
        const step = steps[i];

        if (!step.enabled) {
          continue;
        }

        if (shouldStop) {
          continue;
        }

        if (step.wait_before > 0) {
          await sleep(step.wait_before);
        }

        const result = await executeStepGlobal(
          step,
          run.id,
          workspaceId,
          orchestrationId,
          collectionsData,
          envConfig,
        );

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
        workspaceId,
        orchestrationId,
        runId: run.id,
        status: shouldStop ? "stopped" : failedCount > 0 ? "failed" : "success",
        endTime: endTime.toISOString(),
        totalTime,
      });

      showToast(
        t("toast.scheduledRunComplete", {
          success: successCount,
          failed: failedCount,
        }),
        failedCount > 0 ? "warning" : "success",
      );
    } catch (e) {
      console.error("[OrchestrationExecution] 执行编排失败:", e);
      showToast(
        t("toast.scheduledRunFailed", { error: e.message || e }),
        "error",
      );

      // 如果已创建执行记录，更新状态为失败
      if (currentRunId.value) {
        try {
          await invoke("complete_orchestration_run_cmd", {
            workspaceId,
            orchestrationId,
            runId: currentRunId.value,
            status: "failed",
            endTime: new Date().toISOString(),
            totalTime: 0,
          });
        } catch (updateError) {
          console.error(
            "[OrchestrationExecution] 更新执行状态失败:",
            updateError,
          );
        }
      }
    } finally {
      runningOrchestrations.delete(orchestrationId);
      currentRunId.value = null;
    }
  };

  return {
    executeOrchestration,
  };
}

/**
 * 执行单个步骤（全局版本）
 * 参考 OrchestrationEditor 的 executeStep 逻辑
 */
async function executeStepGlobal(
  step,
  runId,
  workspaceId,
  orchestrationId,
  collectionsData,
  envConfig,
) {
  const startTime = new Date();
  const apiId = step.api_id;

  // 查找 API 信息
  const api = findApiInCollections(collectionsData, apiId);

  if (!api) {
    console.error(`[OrchestrationExecution] API not found: ${apiId}`);
    const endTime = new Date();
    await invoke("update_orchestration_run_step_cmd", {
      workspaceId,
      orchestrationId,
      runId,
      stepResult: {
        step_id: step.id,
        api_id: apiId,
        api_name: "",
        status: "failed",
        start_time: startTime.toISOString(),
        end_time: endTime.toISOString(),
        response_time: 0,
        status_code: 0,
        response_body: null,
        response_headers: {},
        test_results: [],
        failure_reason: "API not found",
        failure_message: null,
        retry_count: 0,
        request_method: "",
        request_url: "",
        request_original_url: null,
        request_headers: [],
        request_body: null,
        request_body_type: null,
      },
    });
    return { step_id: step.id, status: "failed" };
  }

  // 找祖先集合（用于脚本执行）
  const ancestorCollections = findAncestorCollectionsForApi(
    collectionsData,
    apiId,
  );

  // 获取环境变量
  const activeEnvVars = await invoke("get_active_variables", {
    workspaceId,
  });

  // 合并集合变量
  const collVarsObj = mergeCollectionVariablesToObject(ancestorCollections);
  const environmentId = envConfig?.active_environment_id;

  const request = {
    method: api.method || "GET",
    url: api.url || "",
    params: api.params || [],
    headers: api.headers || [],
    body: api.body || "",
    bodyType: api.body_type || "raw",
    formData: api.form_fields || [],
    timeout: api.timeout,
  };

  // 执行前置脚本（调用后端命令）
  let modifiedRequest = request;
  let modifiedEnvVars = activeEnvVars || {};
  let modifiedCollVars = collVarsObj;

  try {
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

  // 合并公共请求头
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

  for (const h of api.common_headers || []) {
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

  // 处理 JSON5 body
  let bodyToSend = modifiedRequest.body;
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

  // 发送请求
  let response;
  let failed = false;
  let failureReason = null;
  let failureMessage = null;

  const collectionVariablesArray = Object.entries(modifiedCollVars).map(
    ([key, value]) => ({ key, value, enabled: true }),
  );

  // 将 params 合并到 URL 中（脚本可能修改了 params）
  const urlWithParams = buildUrlWithParams(
    modifiedRequest.url,
    modifiedRequest.params,
  );

  try {
    const result = await invoke("send_http_request", {
      method: modifiedRequest.method,
      url: urlWithParams,
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
      workspaceId,
      timeout: request.timeout || null,
      apiId,
      apiName: step.name || api.name,
      collectionVariables:
        collectionVariablesArray.length > 0 ? collectionVariablesArray : null,
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

  // 执行后置脚本（调用后端命令）
  let testResults = [];
  try {
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

    // 保存后置脚本修改的目标环境变量（环境脚本操作自己的环境）
    if (
      postScriptResult.target_environment_id &&
      postScriptResult.modified_target_environment_vars
    ) {
      const targetEnvId = postScriptResult.target_environment_id;
      const targetEnvVars = postScriptResult.modified_target_environment_vars;
      const targetEnv = envConfig.environments.find(
        (e) => e.id === targetEnvId,
      );
      if (targetEnv && Object.keys(targetEnvVars).length > 0) {
        const updatedVariables = [...(targetEnv.variables || [])];
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
      const targetCollVars = postScriptResult.modified_target_collection_vars;
      if (Object.keys(targetCollVars).length > 0) {
        const targetCollection = ancestorCollections.find(
          (c) => c.id === targetCollId,
        );
        if (targetCollection) {
          const existingVars = targetCollection.collection_variables || [];
          const updatedVariables = [...existingVars];
          for (const [key, value] of Object.entries(targetCollVars)) {
            const existingVar = updatedVariables.find((v) => v.key === key);
            if (existingVar) {
              existingVar.value = value;
            } else {
              updatedVariables.push({ key, value, enabled: true });
            }
          }
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
  } catch (e) {
    console.error(`后置脚本执行失败: ${e}`);
  }

  const requestEndTime = new Date();
  const stepResult = {
    step_id: step.id,
    api_id: apiId,
    api_name: step.name || api.name,
    status: failed ? "failed" : "success",
    start_time: startTime.toISOString(),
    end_time: requestEndTime.toISOString(),
    response_time: response.time || 0,
    status_code: response.status,
    response_body: response.body,
    response_headers: response.headers,
    test_results: testResults,
    failure_reason: failureReason,
    failure_message: failureMessage,
    retry_count: 0,
    request_method: modifiedRequest.method,
    request_url: modifiedRequest.url,
    request_original_url: api.url,
    request_headers: headersToSend,
    request_body: bodyToSend,
    request_body_type: request.bodyType,
  };

  await invoke("update_orchestration_run_step_cmd", {
    workspaceId,
    orchestrationId,
    runId,
    stepResult,
  });

  return { step_id: step.id, status: stepResult.status };
}

/**
 * 休眠函数
 */
function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
