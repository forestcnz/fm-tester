import { onMounted, onUnmounted } from "vue";
import { useDialogStack } from "./useDialogStack.js";

export function useKeyboardShortcuts({ onSave }) {
  const { closeTop } = useDialogStack();

  const handleKeyDown = (e) => {
    if (e.key === "Escape") {
      if (closeTop()) {
        e.preventDefault();
        return;
      }
      if (
        document.activeElement &&
        (document.activeElement.tagName === "INPUT" ||
          document.activeElement.tagName === "TEXTAREA" ||
          document.activeElement.isContentEditable)
      ) {
        document.activeElement.blur();
        e.preventDefault();
      }
      return;
    }

    if ((e.ctrlKey || e.metaKey) && (e.key === "s" || e.key === "S")) {
      // Monaco 编辑器内自带保存快捷键，跳过
      if (document.activeElement?.closest(".monaco-editor")) return;
      // 在文本输入框 / 文本域 / 可编辑区域中时不拦截，
      // 避免重命名、对话框输入等场景误触发接口保存
      const el = document.activeElement;
      const isInTextInput =
        el &&
        (el.tagName === "INPUT" ||
          el.tagName === "TEXTAREA" ||
          el.tagName === "SELECT" ||
          el.isContentEditable);
      if (isInTextInput && !el?.closest(".request-panel")) {
        return;
      }
      e.preventDefault();
      onSave?.();
    }
  };

  onMounted(() => {
    document.addEventListener("keydown", handleKeyDown);
  });

  onUnmounted(() => {
    document.removeEventListener("keydown", handleKeyDown);
  });
}
