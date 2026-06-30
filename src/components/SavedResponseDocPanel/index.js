import { ref, computed, watch } from "vue";
import { renderMarkdown } from "../../utils/markdown.js";

/**
 * SavedResponseDocPanel composable
 * 用于展示保存响应的 MD 文档预览
 */
export function useSavedResponseDocPanelSetup(props, emit) {
  // 文档内容（从 savedResponse 中获取）
  const docContent = ref("");

  // 渲染后的 Markdown HTML（已通过 DOMPurify 消毒）
  const renderedDocHtml = computed(() => {
    const content = docContent.value || "";
    if (!content) return "";
    return renderMarkdown(content);
  });

  // 加载文档内容
  const loadDocContent = () => {
    if (props.savedResponse?.doc_content) {
      docContent.value = props.savedResponse.doc_content;
    } else {
      docContent.value = "";
    }
  };

  // 监听 savedResponse 变化
  watch(
    () => props.savedResponse,
    () => {
      loadDocContent();
    },
    { immediate: true },
  );

  // 关闭面板
  const close = () => {
    emit("close");
  };

  return {
    docContent,
    renderedDocHtml,
    close,
  };
}
