import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { confirm } from "@tauri-apps/plugin-dialog";
import { useI18n } from "vue-i18n";
import { useDialogEscape } from "../../composables/useDialogStack.js";
import { showToast } from "../../composables/useToast.js";

export function useGitBackupRestoreSetup(props, emit) {
  const { t } = useI18n();
  const loading = ref(false);
  const restoring = ref(false);
  const deleting = ref(false);
  const backups = ref([]);
  const selected = ref(null);
  const newName = ref("");
  const error = ref("");

  // 是否为覆盖模式（targetWorkspace 存在则覆盖该工作区，否则新建）
  const isOverwrite = computed(() => !!props.targetWorkspace);

  const formatSize = (bytes) => {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / 1024 / 1024).toFixed(2) + " MB";
  };

  const formatTimestamp = (ts) => {
    if (!ts || ts.length < 15) return ts;
    return (
      ts.slice(0, 4) +
      "-" +
      ts.slice(4, 6) +
      "-" +
      ts.slice(6, 8) +
      " " +
      ts.slice(9, 11) +
      ":" +
      ts.slice(11, 13) +
      ":" +
      ts.slice(13, 15)
    );
  };

  const loadBackups = async () => {
    try {
      loading.value = true;
      error.value = "";
      selected.value = null;
      newName.value = "";
      const list = await invoke("list_workspace_backups");
      backups.value = list || [];
    } catch (e) {
      console.error(e);
      error.value = String(e);
      backups.value = [];
    } finally {
      loading.value = false;
    }
  };

  const selectBackup = (item) => {
    selected.value = item;
  };

  const confirmRestore = async () => {
    if (!selected.value) return;
    try {
      restoring.value = true;
      let ws;
      if (isOverwrite.value) {
        // 覆盖模式：恢复到目标工作区（保留 id 与名称，替换全部数据）
        ws = await invoke("restore_into_workspace", {
          targetWorkspaceId: props.targetWorkspace.id,
          workspaceName: selected.value.workspace_name,
          fileName: selected.value.file_name,
        });
      } else {
        // 新建模式：导入为新工作区
        ws = await invoke("restore_workspace_from_backup", {
          workspaceName: selected.value.workspace_name,
          fileName: selected.value.file_name,
          newName: newName.value.trim() || null,
        });
      }
      const successMsg = isOverwrite.value
        ? t("gitBackup.restoreSuccessOverwrite")
        : t("gitBackup.restoreSuccess");
      showToast(successMsg, "success");
      emit("imported", ws);
      emit("close");
    } catch (e) {
      console.error(e);
      error.value = String(e);
      showToast(`${t("gitBackup.restoreFailed")}: ${e}`, "error");
    } finally {
      restoring.value = false;
    }
  };

  const close = () => {
    if (restoring.value || deleting.value) return;
    emit("close");
  };

  const deleteBackup = async (item) => {
    if (deleting.value) return;
    const confirmed = await confirm(t("gitBackup.deleteConfirm"));
    if (!confirmed) return;
    try {
      deleting.value = true;
      await invoke("delete_backup", {
        workspaceName: item.workspace_name,
        fileName: item.file_name,
      });
      showToast(t("gitBackup.deleteSuccess"), "success");
      if (selected.value && selected.value.file_name === item.file_name) {
        selected.value = null;
      }
      await loadBackups();
    } catch (e) {
      console.error(e);
      showToast(`${t("gitBackup.deleteFailed")}: ${e}`, "error");
    } finally {
      deleting.value = false;
    }
  };

  useDialogEscape(() => props.visible, close);
  watch(
    () => props.visible,
    (v) => {
      if (v) loadBackups();
    },
  );

  return {
    t,
    loading,
    restoring,
    deleting,
    backups,
    selected,
    newName,
    error,
    isOverwrite,
    formatSize,
    formatTimestamp,
    loadBackups,
    selectBackup,
    confirmRestore,
    deleteBackup,
    close,
  };
}
