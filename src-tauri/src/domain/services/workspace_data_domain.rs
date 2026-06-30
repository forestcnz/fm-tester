//! 工作区数据领域服务
//!
//! environment / memory / cookie 的纯业务逻辑。
//!
//! ## 设计理由
//! - 三者的持久化操作在 WorkspaceDataRepository 中独立读写
//! - 领域服务合并，保持架构一致性
//! - 减少服务数量，简化代码结构

use crate::domain::models::common::generate_id;
use crate::domain::models::{Cookie, Environment, EnvironmentsConfig, MemoryConfig, ReplaceResult};
use regex::Regex;
use std::collections::{HashMap, HashSet};

// 全局缓存的变量引用正则：避免每次 replace_variables 调用都重新编译
// （该函数在 HTTP 请求 / 压测 / 每个 header / body 上都会调用）
lazy_static::lazy_static! {
    static ref VAR_REF_RE: Regex = Regex::new(r"\{\{([^}]+)\}\}").expect("invalid regex");
}

/// 工作区数据领域服务
pub struct WorkspaceDataDomainService;

// === Environment 相关 ===

impl WorkspaceDataDomainService {
    /// 验证环境
    pub fn validate_environment(env: &Environment) -> Result<(), String> {
        env.validate()
    }

    /// 验证变量名
    pub fn validate_variable_key(key: &str) -> Result<(), String> {
        if key.trim().is_empty() {
            return Err("变量名不能为空".to_string());
        }
        if key.contains(' ') {
            return Err("变量名不能包含空格".to_string());
        }
        Ok(())
    }

    /// 生成环境 ID
    pub fn generate_environment_id() -> String {
        generate_id("env")
    }

    /// 创建环境实体
    pub fn create_environment_entity(name: String) -> Environment {
        Environment {
            id: Self::generate_environment_id(),
            name,
            variables: Vec::new(),
            common_headers: None,
        }
    }

    /// 获取激活环境的变量映射
    pub fn get_active_variables_map(config: &EnvironmentsConfig) -> HashMap<String, String> {
        if let Some(env) = config.get_active_environment() {
            env.variables
                .iter()
                .filter(|v| v.enabled)
                .map(|v| (v.key.clone(), v.value.clone()))
                .collect()
        } else {
            HashMap::new()
        }
    }

    /// 替换字符串中的环境变量 {{变量名}}
    /// 返回替换结果，包含未定义变量的警告
    pub fn replace_variables(text: &str, variables: &HashMap<String, String>) -> ReplaceResult {
        // 使用全局缓存的正则（VAR_REF_RE），避免每次调用重新编译
        let re: &Regex = &VAR_REF_RE;

        let mut result = text.to_string();

        // 找出所有变量引用
        let mut matches: Vec<(usize, usize, String)> = Vec::new();
        for cap in re.captures_iter(text) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str().trim();
            matches.push((full_match.start(), full_match.end(), var_name.to_string()));
        }

        // 按变量名分组，记录未定义的变量
        let mut undefined_set = HashSet::new();

        // 替换已知变量
        for (_, _, var_name) in &matches {
            if !variables.contains_key(var_name) {
                undefined_set.insert(var_name.clone());
            }
        }

        // 实际替换
        for (key, value) in variables {
            let pattern = format!("{{{{{}}}}}", key);
            result = result.replace(&pattern, value);
        }

        // 收集未定义变量列表
        let mut undefined_variables: Vec<String> = undefined_set.into_iter().collect();
        undefined_variables.sort();

        ReplaceResult {
            text: result,
            undefined_variables,
        }
    }
}

// === Memory 相关 ===

impl WorkspaceDataDomainService {
    /// 验证记忆配置
    pub fn validate_memory_config(config: &MemoryConfig) -> Result<(), String> {
        config.validate()
    }

    /// 验证展开集合ID列表
    pub fn validate_expanded_ids(ids: &[String]) -> Result<(), String> {
        let mut seen = HashSet::new();
        for id in ids {
            if id.trim().is_empty() {
                return Err("展开集合ID不能为空".to_string());
            }
            if seen.contains(id) {
                return Err(format!("展开集合ID '{}' 重复", id));
            }
            seen.insert(id.clone());
        }
        Ok(())
    }

    /// 验证打开的标签页列表
    pub fn validate_open_tabs(tabs: &[String]) -> Result<(), String> {
        let mut seen = HashSet::new();
        for tab in tabs {
            if tab.trim().is_empty() {
                return Err("标签页ID不能为空".to_string());
            }
            if seen.contains(tab) {
                return Err(format!("标签页ID '{}' 重复", tab));
            }
            seen.insert(tab.clone());
        }
        Ok(())
    }

    /// 验证激活标签页索引
    pub fn validate_active_tab_index(tabs_len: usize, active_index: usize) -> Result<(), String> {
        if tabs_len > 0 && active_index >= tabs_len {
            return Err(format!(
                "激活标签页索引 {} 超出范围 (共 {} 个标签页)",
                active_index, tabs_len
            ));
        }
        Ok(())
    }
}

// === Cookie 相关 ===

impl WorkspaceDataDomainService {
    /// 验证 Cookie 数据有效性
    ///
    /// 检查：
    /// - name 不能为空
    /// - domain 不能为空
    /// - value 不能为空
    pub fn validate_cookie(cookie: &Cookie) -> Result<(), String> {
        if cookie.name.trim().is_empty() {
            return Err("Cookie 名称不能为空".to_string());
        }
        if cookie.domain.trim().is_empty() {
            return Err("Cookie domain 不能为空".to_string());
        }
        if cookie.value.trim().is_empty() {
            return Err("Cookie value 不能为空".to_string());
        }
        Ok(())
    }
}
