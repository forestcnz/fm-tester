import { ref } from "vue";
import { useI18n } from "vue-i18n";

export function useRunDetailModalSetup(props) {
  const { t } = useI18n();

  const expandedSteps = ref([]);
  const activeTabs = ref({});
  const detailData = ref(null);

  const loadDetail = async () => {
    if (props.runDetail) {
      detailData.value = props.runDetail;
    }
  };

  const toggleStep = (stepId) => {
    const index = expandedSteps.value.indexOf(stepId);
    if (index === -1) {
      expandedSteps.value.push(stepId);
      if (!activeTabs.value[stepId]) {
        activeTabs.value[stepId] = "response";
      }
    } else {
      expandedSteps.value.splice(index, 1);
    }
  };

  const setActiveTab = (stepId, tab) => {
    activeTabs.value[stepId] = tab;
  };

  const isStepExpanded = (stepId) => {
    return expandedSteps.value.includes(stepId);
  };

  const getActiveTab = (stepId) => {
    return activeTabs.value[stepId] || "response";
  };

  const formatTime = (timestamp) => {
    if (!timestamp) return "";
    const date = new Date(timestamp);
    return date.toLocaleString();
  };

  const getTotalTime = (run) => {
    if (!run?.total_time) return "0ms";
    if (run.total_time < 1000) return `${run.total_time}ms`;
    return `${(run.total_time / 1000).toFixed(2)}s`;
  };

  const formatJson = (str) => {
    if (!str) return "";
    try {
      const parsed = JSON.parse(str);
      return JSON.stringify(parsed, null, 2);
    } catch {
      return str;
    }
  };

  const formatUrl = (url) => {
    if (!url) return "";
    try {
      const urlObj = new URL(url);
      return urlObj.pathname + urlObj.search;
    } catch {
      return url;
    }
  };

  const getMethodClass = (method) => {
    return method?.toLowerCase() || "";
  };

  return {
    expandedSteps,
    activeTabs,
    detailData,
    loadDetail,
    toggleStep,
    setActiveTab,
    isStepExpanded,
    getActiveTab,
    formatTime,
    getTotalTime,
    formatJson,
    formatUrl,
    getMethodClass,
    t,
  };
}
