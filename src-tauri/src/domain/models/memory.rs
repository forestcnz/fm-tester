use serde::{Deserialize, Serialize};

/// 工作区记忆配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryConfig {
    pub expanded_ids: Vec<String>,
    #[serde(default)]
    pub open_tabs: Vec<String>,
    #[serde(default)]
    pub open_tab_types: Vec<String>,
    #[serde(default)]
    pub active_tab_index: usize,
    #[serde(default)]
    pub request_tabs: std::collections::HashMap<String, String>,
}

impl MemoryConfig {
    pub fn validate(&self) -> Result<(), String> {
        let mut seen_ids = std::collections::HashSet::new();
        for id in &self.expanded_ids {
            if seen_ids.contains(id) {
                return Err(format!("展开集合ID '{}' 重复", id));
            }
            seen_ids.insert(id.clone());
        }

        let mut seen_tabs = std::collections::HashSet::new();
        for tab in &self.open_tabs {
            if seen_tabs.contains(tab) {
                return Err(format!("标签页ID '{}' 重复", tab));
            }
            seen_tabs.insert(tab.clone());
        }

        if !self.open_tabs.is_empty() {
            if self.active_tab_index >= self.open_tabs.len() {
                return Err("激活标签页索引超出范围".to_string());
            }

            if self.open_tab_types.len() != self.open_tabs.len() {
                return Err("标签页类型数量与标签页数量不匹配".to_string());
            }
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.expanded_ids.is_empty() && self.open_tabs.is_empty()
    }
}
