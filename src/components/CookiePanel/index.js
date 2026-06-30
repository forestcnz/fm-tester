import { invoke } from "@tauri-apps/api/core";

export function useCookiePanel(props, emit) {
  const deleteCookie = async (name, domain) => {
    try {
      await invoke("delete_cookie", {
        workspaceId: props.workspaceId,
        name,
        domain,
      });
      emit("refresh");
    } catch (e) {
      console.error("删除 Cookie 失败:", e);
    }
  };

  const clearCookies = async () => {
    try {
      await invoke("clear_cookies", {
        workspaceId: props.workspaceId,
      });
      emit("refresh");
    } catch (e) {
      console.error("清空 Cookie 失败:", e);
    }
  };

  return { deleteCookie, clearCookies };
}
