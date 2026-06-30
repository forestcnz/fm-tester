import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { showToast } from "./useToast.js";

/**
 * 编排定时任务管理 composable
 * 支持定时配置、启用/禁用、执行历史显示
 */
export function useOrchestrationSchedule(workspaceId, orchestrationId) {
  const { t } = useI18n();

  // 状态管理
  const schedule = ref({
    enabled: false,
    cron_expression: "",
    next_run_at: null,
    last_run_at: null,
    run_count: 0,
  });

  const isRunning = ref(false);
  const cronError = ref("");
  const isLoading = ref(false);

  // Cron 表达式描述缓存
  const cronDescription = computed(() => {
    if (!schedule.value.cron_expression) return "";
    return getCronDescription(schedule.value.cron_expression);
  });

  /**
   * 加载编排的定时配置
   */
  const loadSchedule = async () => {
    if (!workspaceId.value || !orchestrationId.value) return;

    isLoading.value = true;
    try {
      // 直接从编排数据中获取 schedule 字段
      // 使用现有的 get_orchestration 命令
      const orchestration = await invoke("get_orchestration", {
        workspaceId: workspaceId.value,
        orchestrationId: orchestrationId.value,
      });

      if (orchestration && orchestration.schedule) {
        schedule.value = {
          enabled: orchestration.schedule.enabled || false,
          cron_expression: orchestration.schedule.cron_expression || "",
          next_run_at: orchestration.schedule.next_run_at || null,
          last_run_at: orchestration.schedule.last_run_at || null,
          run_count: orchestration.schedule.run_count || 0,
        };
      }

      // 验证 cron 表达式
      if (schedule.value.cron_expression) {
        validateCron(schedule.value.cron_expression);
      }
    } catch (e) {
      console.error("加载定时配置失败:", e);
      showToast(t("toast.scheduleLoadFailed"), "error");
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * 更新定时配置
   */
  const updateSchedule = async (newSchedule) => {
    if (!workspaceId.value || !orchestrationId.value) return;

    // 先验证 cron 表达式
    if (newSchedule.cron_expression) {
      const error = validateCron(newSchedule.cron_expression);
      if (error) {
        showToast(t("toast.invalidCronExpression"), "error");
        return false;
      }
    }

    isLoading.value = true;
    try {
      await invoke("update_orchestration_schedule_cmd", {
        workspaceId: workspaceId.value,
        orchestrationId: orchestrationId.value,
        schedule: {
          enabled: newSchedule.enabled,
          cron_expression: newSchedule.cron_expression,
        },
      });

      // 更新本地状态
      schedule.value = {
        ...schedule.value,
        enabled: newSchedule.enabled,
        cron_expression: newSchedule.cron_expression,
      };

      showToast(t("toast.scheduleUpdated"), "success");
      return true;
    } catch (e) {
      console.error("更新定时配置失败:", e);
      showToast(t("toast.scheduleUpdateFailed"), "error");
      return false;
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * 启用/禁用定时任务
   */
  const toggleEnabled = async () => {
    const newEnabled = !schedule.value.enabled;

    // 如果要启用，必须先验证 cron 表达式
    if (newEnabled && !schedule.value.cron_expression) {
      showToast(t("toast.cronExpressionRequired"), "warning");
      return false;
    }

    if (newEnabled && schedule.value.cron_expression) {
      const error = validateCron(schedule.value.cron_expression);
      if (error) {
        showToast(t("toast.invalidCronExpression"), "error");
        return false;
      }
    }

    return await updateSchedule({
      ...schedule.value,
      enabled: newEnabled,
    });
  };

  /**
   * 验证 cron 表达式格式（Quartz 格式）
   * 支持 6 位 cron 表达式：秒 分 时 日 月 周
   * Quartz 特性：
   *   - 周几：1-7（周日=1，周六=7）或 SUN-SAT
   *   - '?' 符号：日和周互斥使用（一个指定，另一个用 ?）
   */
  const validateCron = (expression) => {
    if (!expression) {
      cronError.value = "";
      return "";
    }

    // 基础格式验证：检查是否为 6 位
    const parts = expression.trim().split(/\s+/);
    if (parts.length !== 6) {
      cronError.value = t("schedule.invalidCronFormat");
      return cronError.value;
    }

    // 检查每个部分的格式
    const [second, minute, hour, day, month, weekday] = parts;

    // Quartz 格式：日和周必须有一个是 '?'（互斥）
    // 但 '*' 也被接受作为通配符

    // 如果都指定了具体值（都不是 * 或 ?），给出提示但不报错
    // Quartz 规范建议一个用 ?，但 croner 也接受 *

    // 验证秒 (0-59 或 *)
    if (!isValidCronPart(second, 0, 59, true)) {
      cronError.value = t("schedule.invalidSecond");
      return cronError.value;
    }

    // 验证分 (0-59 或 *)
    if (!isValidCronPart(minute, 0, 59, true)) {
      cronError.value = t("schedule.invalidMinute");
      return cronError.value;
    }

    // 验证时 (0-23 或 *)
    if (!isValidCronPart(hour, 0, 23, true)) {
      cronError.value = t("schedule.invalidHour");
      return cronError.value;
    }

    // 验证日 (1-31 或 * 或 ?)
    if (!isValidCronPart(day, 1, 31, true)) {
      cronError.value = t("schedule.invalidDay");
      return cronError.value;
    }

    // 验证月 (1-12 或 JAN-DEC 或 *)
    if (!isValidMonthPart(month)) {
      cronError.value = t("schedule.invalidMonth");
      return cronError.value;
    }

    // 验证周 (1-7 或 SUN-SAT 或 * 或 ?)
    if (!isValidWeekdayPart(weekday)) {
      cronError.value = t("schedule.invalidWeekday");
      return cronError.value;
    }

    cronError.value = "";
    return "";
  };

  /**
   * 验证单个 cron 部分（支持 ? 符号）
   */
  const isValidCronPart = (part, min, max, allowQuestion = false) => {
    if (part === "*") return true;
    if (allowQuestion && part === "?") return true;

    // 支持数字
    if (/^\d+$/.test(part)) {
      const num = parseInt(part, 10);
      return num >= min && num <= max;
    }

    // 支持范围 (如 1-5)
    if (/^\d+-\d+$/.test(part)) {
      const [start, end] = part.split("-").map(Number);
      return (
        start >= min && start <= max && end >= min && end <= max && start <= end
      );
    }

    // 支持步长 (如 */5 或 0/5)
    if (/^(\*|\d+)\/\d+$/.test(part)) {
      const [base, step] = part.split("/");
      const stepNum = parseInt(step, 10);
      if (stepNum <= 0) return false;
      if (base === "*") return true;
      const baseNum = parseInt(base, 10);
      return baseNum >= min && baseNum <= max;
    }

    // 支持列表 (如 1,3,5)
    if (/^\d+(,\d+)*$/.test(part)) {
      const nums = part.split(",").map(Number);
      return nums.every((n) => n >= min && n <= max);
    }

    return false;
  };

  /**
   * 验证月部分（支持 JAN-DEC）
   */
  const isValidMonthPart = (part) => {
    if (part === "*") return true;
    if (part === "?") return true;

    // 数字格式
    if (/^\d+$/.test(part)) {
      const num = parseInt(part, 10);
      return num >= 1 && num <= 12;
    }

    // 英文月份名称
    const monthNames = [
      "JAN",
      "FEB",
      "MAR",
      "APR",
      "MAY",
      "JUN",
      "JUL",
      "AUG",
      "SEP",
      "OCT",
      "NOV",
      "DEC",
    ];
    if (monthNames.includes(part.toUpperCase())) return true;

    // 范围格式 (如 JAN-MAR)
    if (/^[A-Z]{3}-[A-Z]{3}$/i.test(part)) {
      const [start, end] = part.toUpperCase().split("-");
      const startIdx = monthNames.indexOf(start);
      const endIdx = monthNames.indexOf(end);
      return startIdx !== -1 && endIdx !== -1 && startIdx <= endIdx;
    }

    // 其他格式（步长、列表）
    return isValidCronPart(part, 1, 12, false);
  };

  /**
   * 验证周部分（Quartz 格式：1-7 或 SUN-SAT）
   * Quartz: 1=周日, 2=周一, ..., 7=周六
   */
  const isValidWeekdayPart = (part) => {
    if (part === "*") return true;
    if (part === "?") return true;

    // 数字格式 (Quartz: 1-7)
    if (/^\d+$/.test(part)) {
      const num = parseInt(part, 10);
      return num >= 1 && num <= 7; // Quartz 格式：1-7
    }

    // 英文星期名称
    const weekdayNames = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
    if (weekdayNames.includes(part.toUpperCase())) return true;

    // 范围格式 (如 MON-FRI 或 1-5)
    if (/^[A-Z]{3}-[A-Z]{3}$/i.test(part)) {
      const [start, end] = part.toUpperCase().split("-");
      const startIdx = weekdayNames.indexOf(start);
      const endIdx = weekdayNames.indexOf(end);
      return startIdx !== -1 && endIdx !== -1 && startIdx <= endIdx;
    }

    if (/^\d+-\d+$/.test(part)) {
      const [start, end] = part.split("-").map(Number);
      return start >= 1 && start <= 7 && end >= 1 && end <= 7 && start <= end;
    }

    // 其他格式（步长、列表）
    return isValidCronPart(part, 1, 7, false);
  };

  /**
   * 获取 cron 表达式的中文描述（Quartz 格式）
   */
  const getCronDescription = (expression) => {
    if (!expression) return "";

    const parts = expression.trim().split(/\s+/);
    if (parts.length !== 6) return "";

    const [second, minute, hour, day, month, weekday] = parts;

    // Quartz 格式星期名称映射（1=周日, 7=周六）
    const weekdaysQuartz = [
      "周日",
      "周一",
      "周二",
      "周三",
      "周四",
      "周五",
      "周六",
    ];
    const weekdaysNames = {
      SUN: 0,
      MON: 1,
      TUE: 2,
      WED: 3,
      THU: 4,
      FRI: 5,
      SAT: 6,
    };

    // 解析星期值
    const parseWeekday = (w) => {
      if (w === "*" || w === "?") return null;
      if (/^\d+$/.test(w)) {
        const num = parseInt(w, 10);
        if (num >= 1 && num <= 7) return num - 1; // Quartz 1-7 转 0-6
      }
      if (weekdaysNames[w.toUpperCase()] !== undefined) {
        return weekdaysNames[w.toUpperCase()];
      }
      return null;
    };

    // 常见模式匹配
    // 每分钟
    if (second === "0" && minute === "*" && hour === "*") {
      return t("schedule.everyMinute");
    }

    // 每小时
    if (second === "0" && minute === "0" && hour === "*") {
      return t("schedule.everyHour");
    }

    // 每天特定时间
    if (
      second === "0" &&
      minute !== "*" &&
      hour !== "*" &&
      (day === "*" || day === "?") &&
      (month === "*" || month === "?") &&
      (weekday === "*" || weekday === "?")
    ) {
      const h = parseInt(hour, 10);
      const m = parseInt(minute, 10);
      return t("schedule.everyDayAt", {
        hour: h.toString().padStart(2, "0"),
        minute: m.toString().padStart(2, "0"),
      });
    }

    // 每周特定时间（Quartz 格式）
    if (
      second === "0" &&
      minute !== "*" &&
      hour !== "*" &&
      (day === "?" || day === "*") &&
      (month === "*" || month === "?") &&
      weekday !== "*" &&
      weekday !== "?"
    ) {
      const w = parseWeekday(weekday);
      if (w !== null) {
        const h = parseInt(hour, 10);
        const m = parseInt(minute, 10);
        return t("schedule.everyWeekAt", {
          weekday: weekdaysQuartz[w],
          hour: h.toString().padStart(2, "0"),
          minute: m.toString().padStart(2, "0"),
        });
      }
    }

    // 每月特定日期和时间
    if (
      second === "0" &&
      minute !== "*" &&
      hour !== "*" &&
      day !== "*" &&
      day !== "?" &&
      (month === "*" || month === "?") &&
      (weekday === "?" || weekday === "*")
    ) {
      const h = parseInt(hour, 10);
      const m = parseInt(minute, 10);
      const d = parseInt(day, 10);
      return t("schedule.everyMonthAt", {
        day: d,
        hour: h.toString().padStart(2, "0"),
        minute: m.toString().padStart(2, "0"),
      });
    }

    // 默认返回原始表达式
    return t("schedule.customExpression", { expression });
  };

  /**
   * 计算接下来的 N 次执行时间（调用后端）
   */
  const getNextRunTimes = async (expression, count = 5) => {
    if (!expression) return [];

    const error = validateCron(expression);
    if (error) return [];

    try {
      const times = await invoke("get_next_run_times_cmd", {
        cronExpression: expression,
        count: count,
      });
      return times;
    } catch (e) {
      console.error("获取下次执行时间失败:", e);
      return [];
    }
  };

  /**
   * 格式化时间显示
   */
  const formatTime = (timestamp) => {
    if (!timestamp) return "";
    const date = new Date(timestamp);
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    const hour = String(date.getHours()).padStart(2, "0");
    const minute = String(date.getMinutes()).padStart(2, "0");
    const second = String(date.getSeconds()).padStart(2, "0");
    return `${year}-${month}-${day} ${hour}:${minute}:${second}`;
  };

  // 监听编排 ID 变化，自动加载配置
  watch(
    orchestrationId,
    async (newId, _oldId) => {
      if (newId) {
        await loadSchedule();
      } else {
        // 清空状态
        schedule.value = {
          enabled: false,
          cron_expression: "",
          next_run_at: null,
          last_run_at: null,
          run_count: 0,
        };
        cronError.value = "";
      }
    },
    { immediate: true },
  );

  return {
    schedule,
    isRunning,
    cronError,
    isLoading,
    cronDescription,
    loadSchedule,
    updateSchedule,
    toggleEnabled,
    validateCron,
    getCronDescription,
    getNextRunTimes,
    formatTime,
  };
}
