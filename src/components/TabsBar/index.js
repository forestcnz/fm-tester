// TabsBar composable
export function useTabsBarSetup(props, emit) {
  const selectTab = (index) => {
    emit("update:activeTab", index);
    // 通知父组件选中对应的项，让侧边栏展开父集合
    const tab = props.tabs[index];
    if (tab?.tabType === "collection") {
      emit("selectCollection", tab.id);
    } else if (tab?.tabType === "api") {
      emit("selectApi", tab.id);
    }
  };

  const closeTab = (index, event) => {
    if (event) {
      event.stopPropagation();
    }
    emit("closeTab", index);
  };

  const closeAllTabs = () => {
    emit("closeAllTabs");
  };

  const closeOtherTabs = (keepIndex) => {
    emit("closeOtherTabs", keepIndex);
  };

  return {
    selectTab,
    closeTab,
    closeAllTabs,
    closeOtherTabs,
  };
}
