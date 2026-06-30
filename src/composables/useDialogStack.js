import { ref, watch, onBeforeUnmount } from "vue";

const stack = ref([]);

export function useDialogStack() {
  const push = (closeFn) => {
    stack.value.push(closeFn);
    return () => {
      const idx = stack.value.indexOf(closeFn);
      if (idx >= 0) stack.value.splice(idx, 1);
    };
  };

  const closeTop = () => {
    const s = stack.value;
    if (s.length > 0) {
      s[s.length - 1]();
      return true;
    }
    return false;
  };

  return { push, closeTop };
}

export function useDialogEscape(visibleFn, closeFn) {
  const { push } = useDialogStack();
  let unregister = null;

  watch(
    visibleFn,
    (v) => {
      if (v) {
        if (!unregister) unregister = push(closeFn);
      } else {
        if (unregister) {
          unregister();
          unregister = null;
        }
      }
    },
    { immediate: true, flush: "sync" },
  );

  onBeforeUnmount(() => {
    if (unregister) {
      unregister();
      unregister = null;
    }
  });
}
