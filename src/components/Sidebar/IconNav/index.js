import { useI18n } from "vue-i18n";
import { navItems } from "../index.js";

// 导出 composable 函数
export function useIconNavSetup(props, emit) {
  const { t } = useI18n();

  // 选择导航项
  const selectNav = (key) => {
    emit("navChange", key);
  };

  // 获取导航项名称（国际化）
  const getNavName = (item) => {
    return t(item.nameKey);
  };

  return {
    navItems,
    selectNav,
    getNavName,
  };
}
