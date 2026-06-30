import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save, open } from "@tauri-apps/plugin-dialog";
import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
import { showToast } from "./useToast.js";
import { useI18n } from "vue-i18n";

const exporting = ref(false);
const importing = ref(false);

export function useWorkspaceIO() {
  const { t } = useI18n();

  const exportWorkspace = async (workspaceId) => {
    if (exporting.value) return;

    exporting.value = true;
    try {
      const jsonContent = await invoke("export_workspace", { workspaceId });

      const filePath = await save({
        filters: [{ name: "JSON", extensions: ["json"] }],
        defaultPath: `workspace-export.json`,
        title: t("workspace.export"),
      });

      if (filePath) {
        await writeTextFile(filePath, jsonContent);
        showToast(t("workspace.exportSuccess"), "success");
      }
    } catch (error) {
      console.error("导出工作区失败:", error);
      showToast(t("workspace.exportFailed") + ": " + error, "error");
    } finally {
      exporting.value = false;
    }
  };

  const importWorkspace = async () => {
    if (importing.value) return;

    importing.value = true;
    try {
      const filePath = await open({
        filters: [{ name: "JSON", extensions: ["json"] }],
        multiple: false,
        title: t("workspace.import"),
      });

      if (!filePath) {
        importing.value = false;
        return null;
      }

      const content = await readTextFile(filePath);
      return content;
    } catch (error) {
      console.error("读取导入文件失败:", error);
      showToast(t("workspace.importFailed") + ": " + error, "error");
      importing.value = false;
      return null;
    }
  };

  const previewImport = async (content) => {
    try {
      const preview = await invoke("preview_workspace_import", { content });
      return preview;
    } catch (error) {
      console.error("预览导入失败:", error);
      throw error;
    }
  };

  const executeImport = async (content, newName = null) => {
    try {
      const workspace = await invoke("import_workspace", {
        content,
        newName,
      });
      showToast(t("workspace.importSuccess"), "success");
      return workspace;
    } catch (error) {
      console.error("导入工作区失败:", error);
      showToast(t("workspace.importFailed") + ": " + error, "error");
      throw error;
    } finally {
      importing.value = false;
    }
  };

  return {
    exporting,
    importing,
    exportWorkspace,
    importWorkspace,
    previewImport,
    executeImport,
  };
}
