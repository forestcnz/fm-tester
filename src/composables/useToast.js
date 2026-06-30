import { ref } from "vue";

// Toast 消息列表
const toasts = ref([]);

// Toast 定时器映射（id -> timer），用于在清除时取消未触发的定时器
const timers = new Map();

let toastId = 0;

/**
 * 显示 Toast 提示
 * @param {string} message - 提示消息
 * @param {string} type - 类型：'success' | 'error' | 'warning' | 'info'
 * @param {number} duration - 显示时长（毫秒），默认 3000
 */
export function showToast(message, type = "info", duration = 3000) {
  const id = ++toastId;

  toasts.value.push({
    id,
    message,
    type,
    visible: true,
  });

  // 自动关闭
  if (duration > 0) {
    const timer = setTimeout(() => {
      removeToast(id);
    }, duration);
    timers.set(id, timer);
  }

  return id;
}

/**
 * 移除 Toast
 * @param {number} id - Toast ID
 */
export function removeToast(id) {
  // 取消未触发的定时器，避免 removeToast 重复调用
  const timer = timers.get(id);
  if (timer) {
    clearTimeout(timer);
    timers.delete(id);
  }
  const index = toasts.value.findIndex((t) => t.id === id);
  if (index >= 0) {
    toasts.value.splice(index, 1);
  }
}

/**
 * 清除所有 Toast
 */
export function clearAllToasts() {
  // 取消所有未触发的定时器，避免定时器堆积
  for (const timer of timers.values()) {
    clearTimeout(timer);
  }
  timers.clear();
  toasts.value = [];
}

export function useToast() {
  return {
    toasts,
    showToast,
    removeToast,
    clearAllToasts,
  };
}
