import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import * as echarts from "echarts";
import { mergeCollectionVariablesToObject } from "../../utils/scriptEngine.js";

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

export function useStressTestPanelSetup(props, _emit) {
  const { t } = useI18n();

  const config = ref({
    concurrent: 10,
    totalRequests: 100,
    durationSeconds: null,
    rampUpSeconds: 0,
    timeoutMs: 30000,
  });

  const FIELD_LIMITS = {
    concurrent: { min: 1, max: 999 },
    rampUpSeconds: { min: 0, max: 3600 },
    totalRequests: { min: 1, max: 1000000 },
    durationSeconds: { min: 1, max: 86400 },
    timeoutMs: { min: 1000, max: 600000 },
  };
  watch(
    () => ({
      concurrent: config.value.concurrent,
      rampUpSeconds: config.value.rampUpSeconds,
      totalRequests: config.value.totalRequests,
      durationSeconds: config.value.durationSeconds,
      timeoutMs: config.value.timeoutMs,
    }),
    (vals) => {
      for (const [field, { min, max }] of Object.entries(FIELD_LIMITS)) {
        const v = vals[field];
        if (v == null || isNaN(v)) continue;
        const clamped = Math.min(max, Math.max(min, Math.floor(v)));
        if (clamped !== v) config.value[field] = clamped;
      }
    },
  );

  const historyResults = ref([]);
  let isLoadingParams = false;

  const loadStressParams = async () => {
    if (!props.workspaceId || !props.apiId) return;
    isLoadingParams = true;
    try {
      const paramsConfig = await invoke("get_stress_params", {
        workspaceId: props.workspaceId,
        apiId: props.apiId,
      });
      config.value = {
        concurrent: paramsConfig.concurrent || 10,
        totalRequests: paramsConfig.total_requests || 100,
        durationSeconds: paramsConfig.duration_seconds || null,
        rampUpSeconds: paramsConfig.ramp_up_seconds || 0,
        timeoutMs: paramsConfig.timeout_ms || 30000,
      };
    } catch (e) {
      console.error("加载压测参数失败:", e);
    } finally {
      isLoadingParams = false;
    }
  };

  const saveStressParams = async () => {
    if (!props.workspaceId || !props.apiId) return;
    try {
      await invoke("save_stress_params", {
        workspaceId: props.workspaceId,
        apiId: props.apiId,
        config: {
          concurrent: config.value.concurrent,
          total_requests: config.value.totalRequests || null,
          duration_seconds: config.value.durationSeconds || null,
          ramp_up_seconds: config.value.rampUpSeconds,
          timeout_ms: config.value.timeoutMs,
        },
      });
    } catch (e) {
      console.error("保存压测参数失败:", e);
    }
  };

  const isRunning = ref(false);
  const testId = ref(null);
  const progress = ref(null);
  const result = ref(null);
  const error = ref(null);
  const viewingHistoryId = ref(null);
  const showFailedDetails = ref(false);
  const chartRef = ref(null);
  let chartInstance = null;
  let resizeObserver = null;
  let pendingChartData = null;

  const getChartOption = () => ({
    grid: {
      top: 30,
      right: 20,
      bottom: 30,
      left: 50,
    },
    xAxis: {
      type: "category",
      name: t("stress.seconds"),
      nameLocation: "end",
      nameTextStyle: { fontSize: 11, color: "#666" },
      axisLabel: { fontSize: 11, color: "#666" },
      axisLine: { lineStyle: { color: "#e9ecef" } },
    },
    yAxis: [
      {
        type: "value",
        name: t("stress.qps") + "/" + t("stress.avgTime"),
        nameTextStyle: { fontSize: 11, color: "#666" },
        axisLabel: { fontSize: 11, color: "#666" },
        axisLine: { lineStyle: { color: "#e9ecef" } },
        splitLine: { lineStyle: { color: "#e9ecef" } },
      },
      {
        type: "value",
        name: t("stress.count"),
        nameTextStyle: { fontSize: 11, color: "#666" },
        axisLabel: { fontSize: 11, color: "#666" },
        axisLine: { lineStyle: { color: "#e9ecef" } },
        splitLine: { show: false },
      },
    ],
    series: [
      {
        name: t("stress.qps"),
        type: "line",
        smooth: true,
        symbol: "none",
        lineStyle: { width: 2, color: "#1890ff" },
        itemStyle: { color: "#1890ff" },
        yAxisIndex: 0,
      },
      {
        name: t("stress.avgTime"),
        type: "line",
        smooth: true,
        symbol: "none",
        lineStyle: { width: 2, color: "#722ed1" },
        itemStyle: { color: "#722ed1" },
        yAxisIndex: 0,
      },
      {
        name: t("stress.concurrent"),
        type: "line",
        smooth: true,
        symbol: "none",
        lineStyle: { width: 2, color: "#faad14" },
        itemStyle: { color: "#faad14" },
        yAxisIndex: 1,
      },
      {
        name: t("stress.totalRequests"),
        type: "line",
        smooth: true,
        symbol: "none",
        lineStyle: { width: 2, color: "#13c2c2" },
        itemStyle: { color: "#13c2c2" },
        yAxisIndex: 1,
      },
      {
        name: t("stress.success"),
        type: "line",
        smooth: true,
        symbol: "none",
        lineStyle: { width: 2, color: "#52c416" },
        itemStyle: { color: "#52c416" },
        yAxisIndex: 1,
      },
      {
        name: t("stress.failed"),
        type: "line",
        smooth: true,
        symbol: "none",
        lineStyle: { width: 2, color: "#ff4d4f" },
        itemStyle: { color: "#ff4d4f" },
        yAxisIndex: 1,
      },
    ],
    tooltip: {
      trigger: "axis",
      confine: true,
    },
    legend: {
      top: 0,
      left: "center",
      itemWidth: 15,
      itemHeight: 8,
      textStyle: { fontSize: 11 },
    },
  });

  const initChart = () => {
    if (!chartRef.value) return;
    const { clientWidth, clientHeight } = chartRef.value;
    if (clientWidth === 0 || clientHeight === 0) return false;
    if (chartInstance) chartInstance.dispose();
    chartInstance = echarts.init(chartRef.value);
    chartInstance.setOption(getChartOption());
    return true;
  };

  const updateChart = () => {
    if (!chartRef.value) return;
    if (!chartInstance) {
      if (!initChart()) return;
    }
    if (!progress.value?.history?.length) return;
    const history = progress.value.history;
    chartInstance.setOption({
      xAxis: {
        data: history.map((h) => h.second + "s"),
      },
      series: [
        { data: history.map((h) => h.qps.toFixed(2)) },
        { data: history.map((h) => h.avg_time_ms.toFixed(1)) },
        { data: history.map((h) => h.concurrent || 0) },
        { data: history.map((h) => h.requests) },
        { data: history.map((h) => h.successful) },
        { data: history.map((h) => h.failed) },
      ],
    });
  };

  const resizeChart = () => {
    if (chartInstance && chartRef.value) {
      const { clientWidth, clientHeight } = chartRef.value;
      if (clientWidth > 0 && clientHeight > 0) {
        chartInstance.resize();
      }
    }
  };

  const renderPendingData = () => {
    if (!chartInstance || !pendingChartData) return;
    chartInstance.setOption(pendingChartData);
    pendingChartData = null;
  };

  const tryInitAndRender = () => {
    if (!chartRef.value) return;
    const { clientWidth, clientHeight } = chartRef.value;
    if (clientWidth === 0 || clientHeight === 0) return;
    if (!chartInstance) {
      initChart();
    }
    if (chartInstance) {
      renderPendingData();
    }
  };

  let unlistenProgress = null;
  let unlistenComplete = null;

  const setupListeners = async () => {
    unlistenProgress = await listen("stress-test-progress", (event) => {
      const data = event.payload;
      if (data.id === testId.value) {
        progress.value = data;
        nextTick(() => updateChart());
      }
    });

    unlistenComplete = await listen("stress-test-complete", (event) => {
      const data = event.payload;
      if (data.id === testId.value) {
        result.value = data;
        isRunning.value = false;
        loadHistory();
      }
    });
  };

  const generateId = () => {
    return (
      "stress-" +
      Date.now().toString(36) +
      Math.random().toString(36).substr(2, 5)
    );
  };

  const startTest = async () => {
    if (!props.apiId) {
      error.value = t("stress.noApi");
      return;
    }

    error.value = null;
    isRunning.value = true;
    progress.value = null;
    result.value = null;
    viewingHistoryId.value = null;

    let modifiedUrl = props.url;
    let modifiedHeaders = props.headers || [];
    let modifiedBody = props.body;
    let modifiedCollVars = props.collectionVariables || [];

    try {
      const collectionsData = await invoke("get_collections", {
        workspaceId: props.workspaceId,
      });
      const ancestorCollections = findAncestorCollectionsForApi(
        collectionsData,
        props.apiId,
      );
      const collVarsObj = mergeCollectionVariablesToObject(ancestorCollections);

      const envConfig = await invoke("get_environments", {
        workspaceId: props.workspaceId,
      });
      const activeEnvVars = await invoke("get_active_variables", {
        workspaceId: props.workspaceId,
      });
      const environmentId = envConfig.active_environment_id;

      const preScriptResult = await invoke("execute_pre_scripts_cmd", {
        input: {
          workspace_id: props.workspaceId,
          api_id: props.apiId,
          environment_id: environmentId,
          ancestor_collections: ancestorCollections.map((c) => ({
            id: c.id,
            name: c.name,
            collection_variables: c.collection_variables || [],
          })),
          environment_variables: activeEnvVars || {},
          collection_variables: collVarsObj,
          request: {
            url: props.url,
            method: props.method,
            headers: [...(props.headers || [])],
            body: props.body,
          },
          silent: true,
        },
      });

      if (preScriptResult.success) {
        const modifiedRequest = preScriptResult.modified_request;
        if (modifiedRequest) {
          modifiedUrl = modifiedRequest.url;
          modifiedHeaders = modifiedRequest.headers;
          modifiedBody = modifiedRequest.body;
        }

        const modifiedCollVarsArray = Object.entries(
          preScriptResult.modified_collection_vars || {},
        ).map(([key, value]) => ({ key, value, enabled: true }));
        modifiedCollVars = [...modifiedCollVars, ...modifiedCollVarsArray];
      }

      // 构建压测配置，包含脚本执行所需的数据
      const testConfig = {
        id: generateId(),
        api_id: props.apiId,
        api_name: props.apiName || "API",
        method: props.method,
        url: modifiedUrl,
        headers: modifiedHeaders,
        body: modifiedBody,
        body_type: props.bodyType,
        form_fields: props.formFields,
        collection_variables: modifiedCollVars,
        concurrent: config.value.concurrent,
        total_requests: config.value.totalRequests || null,
        duration_seconds: config.value.durationSeconds || null,
        ramp_up_seconds: config.value.rampUpSeconds,
        timeout_ms: config.value.timeoutMs,
        environment_id: environmentId,
        ancestor_collections: ancestorCollections.map((c) => ({
          id: c.id,
          name: c.name,
        })),
      };

      testId.value = testConfig.id;

      await invoke("start_stress_test", {
        workspaceId: props.workspaceId,
        config: testConfig,
      });
    } catch (e) {
      console.error(`前置脚本执行失败: ${e}`);
      error.value = e;
      isRunning.value = false;
    }
  };

  const stopTest = async () => {
    if (!testId.value) return;

    try {
      result.value = await invoke("stop_stress_test", {
        id: testId.value,
        workspaceId: props.workspaceId,
      });
      isRunning.value = false;
      loadHistory();
    } catch (e) {
      error.value = e;
    }
  };

  const loadHistory = async () => {
    if (!props.workspaceId || !props.apiId) return;
    try {
      historyResults.value = await invoke("get_api_stress_test_results", {
        workspaceId: props.workspaceId,
        apiId: props.apiId,
      });

      if (historyResults.value.length > 0) {
        const latest = historyResults.value[0];
        const latestResult = await invoke("get_stress_test_result", {
          workspaceId: props.workspaceId,
          apiId: props.apiId,
          id: latest.id,
        });

        if (latestResult?.history?.length) {
          pendingChartData = {
            xAxis: {
              data: latestResult.history.map((h) => h.second + "s"),
            },
            series: [
              { data: latestResult.history.map((h) => h.qps.toFixed(2)) },
              {
                data: latestResult.history.map((h) => h.avg_time_ms.toFixed(1)),
              },
              { data: latestResult.history.map((h) => h.concurrent || 0) },
              { data: latestResult.history.map((h) => h.requests) },
              { data: latestResult.history.map((h) => h.successful) },
              { data: latestResult.history.map((h) => h.failed) },
            ],
          };
          await nextTick();
          tryInitAndRender();
        }
      } else {
        pendingChartData = {
          xAxis: { data: [] },
          series: [
            { data: [] },
            { data: [] },
            { data: [] },
            { data: [] },
            { data: [] },
            { data: [] },
          ],
        };
        await nextTick();
        tryInitAndRender();
      }
    } catch (e) {
      console.error("加载压测记录失败:", e);
    }
  };

  const viewHistoryDetail = async (id) => {
    if (!props.apiId) return;
    try {
      result.value = await invoke("get_stress_test_result", {
        workspaceId: props.workspaceId,
        apiId: props.apiId,
        id: id,
      });
      viewingHistoryId.value = id;
      showFailedDetails.value = false;

      if (result.value?.history?.length) {
        pendingChartData = {
          xAxis: {
            data: result.value.history.map((h) => h.second + "s"),
          },
          series: [
            { data: result.value.history.map((h) => h.qps.toFixed(2)) },
            { data: result.value.history.map((h) => h.avg_time_ms.toFixed(1)) },
            { data: result.value.history.map((h) => h.concurrent || 0) },
            { data: result.value.history.map((h) => h.requests) },
            { data: result.value.history.map((h) => h.successful) },
            { data: result.value.history.map((h) => h.failed) },
          ],
        };
        nextTick(() => tryInitAndRender());
      }
    } catch (e) {
      error.value = e;
    }
  };

  const backToList = () => {
    viewingHistoryId.value = null;
    result.value = null;
    showFailedDetails.value = false;
  };

  const toggleFailedDetails = () => {
    showFailedDetails.value = !showFailedDetails.value;
  };

  const deleteHistory = async (id) => {
    if (!props.apiId) return;
    try {
      await invoke("delete_stress_test_result", {
        workspaceId: props.workspaceId,
        apiId: props.apiId,
        id: id,
      });
      if (viewingHistoryId.value === id) {
        viewingHistoryId.value = null;
        result.value = null;
      }
      loadHistory();
    } catch (e) {
      error.value = e;
    }
  };

  const progressPercent = computed(() => {
    if (!progress.value) return 0;
    if (config.value.totalRequests) {
      return (
        (progress.value.completed_requests / config.value.totalRequests) * 100
      );
    }
    if (config.value.durationSeconds) {
      return (
        (progress.value.elapsed_seconds / config.value.durationSeconds) * 100
      );
    }
    return 0;
  });

  const getStatusClass = (status) => {
    if (status >= 200 && status < 300) return "success";
    if (status >= 300 && status < 400) return "redirect";
    if (status >= 400 && status < 500) return "client-error";
    if (status >= 500) return "server-error";
    return "";
  };

  const formatDate = (dateStr) => {
    if (!dateStr) return "";
    try {
      const date = new Date(dateStr);
      return date.toLocaleString();
    } catch {
      return dateStr;
    }
  };

  onMounted(async () => {
    await setupListeners();
    await loadStressParams();
    await loadHistory();
    window.addEventListener("resize", resizeChart);

    resizeObserver = new ResizeObserver(() => {
      tryInitAndRender();
    });
    if (chartRef.value) {
      resizeObserver.observe(chartRef.value);
    }
  });

  onUnmounted(() => {
    if (unlistenProgress) unlistenProgress();
    if (unlistenComplete) unlistenComplete();
    if (chartInstance) {
      chartInstance.dispose();
      chartInstance = null;
    }
    if (resizeObserver) {
      resizeObserver.disconnect();
      resizeObserver = null;
    }
    window.removeEventListener("resize", resizeChart);
  });

  watch(
    () => props.apiId,
    async () => {
      await loadStressParams();
      await loadHistory();
      result.value = null;
      progress.value = null;
      isRunning.value = false;
      testId.value = null;
      viewingHistoryId.value = null;
      showFailedDetails.value = false;
    },
  );

  watch(
    () => props.workspaceId,
    async () => {
      await loadStressParams();
      await loadHistory();
    },
  );

  // 监听配置变化自动保存
  watch(
    config,
    async () => {
      if (!isLoadingParams) {
        await saveStressParams();
      }
    },
    { deep: true },
  );

  // 请求数和持续时间互斥
  watch(
    () => config.value.totalRequests,
    (val) => {
      if (val && !isLoadingParams && config.value.durationSeconds) {
        config.value.durationSeconds = null;
      }
    },
  );

  watch(
    () => config.value.durationSeconds,
    (val) => {
      if (val && !isLoadingParams && config.value.totalRequests) {
        config.value.totalRequests = null;
      }
    },
  );

  return {
    config,
    isRunning,
    progress,
    result,
    historyResults,
    error,
    viewingHistoryId,
    progressPercent,
    startTest,
    stopTest,
    deleteHistory,
    viewHistoryDetail,
    backToList,
    getStatusClass,
    formatDate,
    showFailedDetails,
    toggleFailedDetails,
    chartRef,
    t,
  };
}
