import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

/**
 * 响应管理 composable（包含Cookie、历史记录、保存响应）
 * @param {Ref} currentWorkspace - 当前工作区引用
 * @param {Ref} tabs - 标签页列表引用
 * @param {Ref} activeTab - 当前激活标签页引用
 * @param {Ref} currentNavKey - 当前导航项引用
 * @param {Ref} sidebarRef - 侧边栏组件引用
 * @param {Ref} response - 响应数据引用
 * @param {Object} currentRequest - 当前请求状态
 * @param {Function} updateCurrentRequest - 更新当前请求函数
 */
export function useResponse(
  currentWorkspace,
  tabs,
  activeTab,
  currentNavKey,
  sidebarRef,
  response,
  currentRequest,
  updateCurrentRequest,
  testResults,
  sseEvents,
) {
  // Cookie 管理
  const cookies = ref([]);
  const showCookiePanel = ref(false);

  // 保存响应对话框
  const showSaveResponseDialog = ref(false);
  const saveResponseDefaultName = ref("");

  // 历史记录
  const selectedHistoryEntry = ref(null);

  // 选中查看的工作区（用于设置面板）
  const selectedWorkspace = ref(null);

  // 保存响应 MD 文档展示状态
  const showSavedResponseDoc = ref(false);
  const selectedSavedResponse = ref(null);

  /**
   * 生成 MD 文档内容
   */
  const generateDocContent = (
    name,
    createdAt,
    request,
    responseData,
    cookiesData,
    testResultsData,
  ) => {
    let doc = "";

    // 标题
    doc += `# ${name}\n\n`;

    // 请求信息
    doc += "## 请求信息\n\n";
    doc += `- **方法**: ${request.method}\n`;
    doc += `- **URL**: ${request.resolvedUrl}\n`;
    if (request.url !== request.resolvedUrl) {
      doc += `- **原始 URL**: ${request.url}\n`;
    }
    doc += "\n";

    // 请求头
    if (request.headers && request.headers.length > 0) {
      const enabledHeaders = request.headers.filter((h) => h.enabled);
      if (enabledHeaders.length > 0) {
        doc += "### 请求头\n\n";
        doc += "| Header | Value |\n";
        doc += "|--------|-------|\n";
        for (const h of enabledHeaders) {
          doc += `| ${h.key} | ${h.value} |\n`;
        }
        doc += "\n";
      }
    }

    // 请求体
    if (request.body) {
      doc += "### 请求体\n\n";
      const bodyType = request.bodyType || "raw";
      doc += `- **类型**: ${bodyType}\n\n`;
      const lang = bodyType.includes("json")
        ? "json"
        : bodyType.includes("xml")
          ? "xml"
          : "";
      doc += "```" + lang + "\n" + request.body + "\n```\n\n";
    }

    // 响应信息
    doc += "## 响应信息\n\n";
    doc += `- **状态码**: ${responseData.status} ${responseData.statusText}\n`;
    doc += `- **响应时间**: ${responseData.time}ms\n`;
    doc += `- **响应大小**: ${responseData.size} bytes\n\n`;

    // 响应头
    if (responseData.headers) {
      const headerKeys = Object.keys(responseData.headers);
      if (headerKeys.length > 0) {
        doc += "### 响应头\n\n";
        doc += "| Header | Value |\n";
        doc += "|--------|-------|\n";
        for (const key of headerKeys) {
          doc += `| ${key} | ${responseData.headers[key]} |\n`;
        }
        doc += "\n";
      }
    }

    // 响应体
    if (responseData.body) {
      doc += "### 响应体\n\n";
      const contentType = (
        responseData.headers["content-type"] ||
        responseData.headers["Content-Type"] ||
        ""
      ).toLowerCase();
      const lang = contentType.includes("json")
        ? "json"
        : contentType.includes("xml") || contentType.includes("html")
          ? "xml"
          : contentType.includes("javascript")
            ? "javascript"
            : "";
      doc += "```" + lang + "\n" + responseData.body + "\n```\n\n";
    }

    // Cookies
    if (cookiesData && cookiesData.length > 0) {
      doc += "## Cookies\n\n";
      doc += "| Name | Value | Domain | Path | Secure | HttpOnly |\n";
      doc += "|------|-------|--------|------|--------|----------|\n";
      for (const c of cookiesData) {
        doc += `| ${c.name} | ${c.value} | ${c.domain} | ${c.path} | ${c.secure ? "✓" : ""} | ${c.http_only ? "✓" : ""} |\n`;
      }
      doc += "\n";
    }

    // 测试结果
    if (testResultsData && testResultsData.length > 0) {
      doc += "## 测试结果\n\n";

      // 统计
      const passedCount = testResultsData.filter((r) => r.passed).length;
      const failedCount = testResultsData.length - passedCount;
      const totalCount = testResultsData.length;
      doc += `- **总计**: ${totalCount} 项测试\n`;
      doc += `- **通过**: ${passedCount} 项\n`;
      doc += `- **失败**: ${failedCount} 项\n\n`;

      doc += "| 测试名称 | 结果 | 错误信息 |\n";
      doc += "|----------|------|----------|\n";
      for (const r of testResultsData) {
        const status = r.passed ? "✓ 通过" : "✗ 失败";
        const error = r.error || "";
        doc += `| ${r.name} | ${status} | ${error} |\n`;
      }
      doc += "\n";
    }

    // 创建时间
    doc += "---\n\n";

    return doc;
  };

  const loadCookies = async () => {
    if (!currentWorkspace.value?.id) {
      cookies.value = [];
      return;
    }
    try {
      const cookieList = await invoke("get_cookies", {
        workspaceId: currentWorkspace.value?.id,
      });
      cookies.value = cookieList || [];
    } catch (e) {
      console.error("加载 Cookies 失败:", e);
    }
  };

  const openCookiePanel = async () => {
    await loadCookies();
    showCookiePanel.value = true;
  };

  const closeCookiePanel = () => {
    showCookiePanel.value = false;
  };

  const showHistoryDetail = computed(() => {
    return currentNavKey.value === "history";
  });

  const showWorkspaceInfo = computed(() => {
    return currentNavKey.value === "workspace" && selectedWorkspace.value;
  });

  // 选择工作区（用于显示设置面板）
  const onSelectWorkspace = (ws) => {
    selectedWorkspace.value = ws;
  };

  const onSelectHistory = (historyEntry) => {
    selectedHistoryEntry.value = historyEntry;
  };

  const onSaveResponse = () => {
    if (!response.value) return;

    const currentTab = tabs.value[activeTab.value];
    if (!currentTab?.name) return;

    const statusCode = response.value?.status ?? "";
    saveResponseDefaultName.value = `${currentTab.name}-${statusCode}`;
    showSaveResponseDialog.value = true;
  };

  const handleSaveResponse = async (name) => {
    if (
      !currentWorkspace.value?.id ||
      !response.value ||
      tabs.value.length === 0
    )
      return;

    const currentTab = tabs.value[activeTab.value];
    if (!currentTab?.id || currentTab.tabType !== "api") return;

    // 获取解析后的 URL（来自上次响应）
    const resolvedUrl =
      currentTab?.lastResponseData?.resolvedUrl || currentRequest.url;

    // 构建请求和响应数据（用于生成 MD 文档）
    const requestData = {
      method: currentRequest.method,
      url: currentRequest.url,
      resolvedUrl: resolvedUrl,
      headers: currentRequest.headers || [],
      body: currentRequest.body || null,
      bodyType: currentRequest.bodyType || null,
    };

    const sseBodyText =
      sseEvents?.value?.length > 0
        ? sseEvents.value
            .map((e) => `${e.time || ""}|${e.data ?? ""}`)
            .join("\n---\n")
        : "";
    const responseData = {
      status: response.value.status,
      statusText: response.value.statusText,
      headers: response.value.headers || {},
      body: sseBodyText || response.value.body || "",
      time: response.value.time || 0,
      size: response.value.size || 0,
    };

    // 生成当前时间戳
    const now = new Date();
    const createdAt = now.toISOString();

    // 生成 MD 文档内容
    const docContent = generateDocContent(
      name,
      createdAt,
      requestData,
      responseData,
      cookies.value,
      testResults?.value || null,
    );

    try {
      await invoke("save_response", {
        workspaceId: currentWorkspace.value?.id,
        name: name,
        apiId: currentTab.id,
        docContent: docContent,
      });

      showSaveResponseDialog.value = false;

      // 只刷新当前 API 的保存响应列表
      if (currentTab?.id) {
        await sidebarRef.value?.refreshApiSavedResponses(currentTab.id);
      }
    } catch (e) {
      console.error("保存响应失败:", e);
    }
  };

  const onSelectSavedResponse = async (responseItem) => {
    if (!currentWorkspace.value?.id) return;

    try {
      const fullResponse = await invoke("get_saved_response", {
        workspaceId: currentWorkspace.value?.id,
        id: responseItem.id,
      });

      // 设置保存响应 MD 文档展示状态
      selectedSavedResponse.value = fullResponse;
      showSavedResponseDoc.value = true;

      // 切换导航到 collection，确保面板可以显示
      currentNavKey.value = "collection";
    } catch (e) {
      console.error("获取保存响应失败:", e);
    }
  };

  // 关闭保存响应 MD 文档面板
  const closeSavedResponseDoc = () => {
    showSavedResponseDoc.value = false;
    selectedSavedResponse.value = null;
  };

  return {
    cookies,
    showCookiePanel,
    loadCookies,
    openCookiePanel,
    closeCookiePanel,
    showSaveResponseDialog,
    saveResponseDefaultName,
    selectedHistoryEntry,
    showHistoryDetail,
    showWorkspaceInfo,
    selectedWorkspace,
    onSelectWorkspace,
    onSelectHistory,
    onSaveResponse,
    handleSaveResponse,
    onSelectSavedResponse,
    showSavedResponseDoc,
    selectedSavedResponse,
    closeSavedResponseDoc,
  };
}
