import { ref } from "vue";

/**
 * ContextMenu composable
 * 提供右键菜单的状态管理
 */
export function useContextMenuSetup() {
  const visible = ref(false);
  const x = ref(0);
  const y = ref(0);

  /**
   * 打开右键菜单
   * @param {MouseEvent} event - 鼠标事件
   */
  const open = (event) => {
    event.preventDefault();
    event.stopPropagation();
    x.value = event.clientX;
    y.value = event.clientY;
    visible.value = true;
  };

  /**
   * 关闭右键菜单
   */
  const close = () => {
    visible.value = false;
  };

  return {
    visible,
    x,
    y,
    open,
    close,
  };
}
