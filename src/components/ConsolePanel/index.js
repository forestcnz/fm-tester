import { ref } from "vue";

// ConsolePanel composable
export function useConsolePanel(_props, _emit) {
  // 展开状态：记录哪些日志索引是展开的
  const expandedLogs = ref(new Set());

  // 面板高度（可拖拽调整）
  const panelHeight = ref(200);
  const minHeight = 100;
  const maxHeight = 400;
  const isDragging = ref(false);

  // 切换日志展开状态
  const toggleExpand = (index) => {
    if (expandedLogs.value.has(index)) {
      expandedLogs.value.delete(index);
    } else {
      expandedLogs.value.add(index);
    }
  };

  // 判断是否展开
  const isExpanded = (index) => {
    return expandedLogs.value.has(index);
  };

  // 开始拖拽
  const startDrag = (e) => {
    isDragging.value = true;
    document.addEventListener("mousemove", onDrag);
    document.addEventListener("mouseup", stopDrag);
    e.preventDefault();
  };

  // 拖拽中
  const onDrag = (e) => {
    if (!isDragging.value) return;
    const newHeight = panelHeight.value - e.movementY;
    panelHeight.value = Math.min(maxHeight, Math.max(minHeight, newHeight));
  };

  // 停止拖拽
  const stopDrag = () => {
    isDragging.value = false;
    document.removeEventListener("mousemove", onDrag);
    document.removeEventListener("mouseup", stopDrag);
  };

  // 解析日志消息为结构化数据
  const parseLogMessage = (message) => {
    // 尝试解析为 JSON 对象
    try {
      const parsed = JSON.parse(message);
      if (typeof parsed === "object" && parsed !== null) {
        return { type: "object", data: parsed };
      }
    } catch {
      // 不是 JSON，检查是否是多行文本
      if (message.includes("\n")) {
        const lines = message.split("\n");
        return { type: "multiline", data: lines };
      }
      // 单行文本
      return { type: "text", data: message };
    }
    return { type: "text", data: message };
  };

  // 获取日志摘要（用于折叠状态显示）
  const getLogSummary = (log) => {
    const parsed = parseLogMessage(log.message);
    if (parsed.type === "object") {
      const data = parsed.data;
      // 请求摘要
      if (data.request) {
        return `${data.request.method} ${data.request.url}`;
      }
      // 响应摘要
      if (data.response) {
        return `${data.response.status} ${data.response.statusText} (${data.response.time}ms)`;
      }
      // 错误摘要
      if (data.error) {
        return `${data.error}`;
      }
      // 其他对象摘要
      return JSON.stringify(data);
    }
    if (parsed.type === "multiline") {
      return parsed.data[0];
    }
    return log.message;
  };

  // 获取展开后的详细内容
  const getLogDetail = (log) => {
    const parsed = parseLogMessage(log.message);
    if (parsed.type === "object") {
      return JSON.stringify(parsed.data, null, 2);
    }
    return log.message;
  };

  return {
    expandedLogs,
    panelHeight,
    isDragging,
    toggleExpand,
    isExpanded,
    startDrag,
    parseLogMessage,
    getLogSummary,
    getLogDetail,
  };
}
