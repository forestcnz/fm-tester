use super::Header;
use serde::{Deserialize, Serialize};

/// 变量（值对象）
///
/// 用于表示环境变量或集合变量。
/// 通过 `enabled` 字段控制是否在替换时使用该变量。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Variable {
    pub fn validate(&self) -> Result<(), String> {
        if self.key.trim().is_empty() {
            return Err("变量名不能为空".to_string());
        }
        if self.key.contains(' ') {
            return Err("变量名不能包含空格".to_string());
        }
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// 变量信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    pub key: String,
    pub value: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 环境（聚合根 Aggregate Root）
///
/// 这是环境聚合的根实体，管理环境变量和公共请求头。
///
/// ## 聚合边界
/// - 一个 Environment 聚合包含：环境本身 + 所有变量 + 公共请求头
/// - 变量是值对象，不可独立存在
///
/// ## 业务规则
/// - 环境名称不能为空
/// - 变量名不能为空且不能包含空格
/// - 环境名称在同一工作区内不能重复
///
/// ## 生命周期
/// - 创建：通过 `EnvironmentDomainService::create_environment_entity()`
/// - 验证：通过 `Environment.validate()` 方法
/// - 持久化：通过 `EnvironmentRepository` 仓储接口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub variables: Vec<Variable>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub common_headers: Option<Vec<Header>>,
}

impl Environment {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("环境名称不能为空".to_string());
        }
        for var in &self.variables {
            var.validate()?;
        }
        Ok(())
    }

    pub fn get_variable(&self, key: &str) -> Option<&Variable> {
        self.variables.iter().find(|v| v.key == key && v.enabled)
    }

    pub fn get_enabled_variables(&self) -> Vec<&Variable> {
        self.variables.iter().filter(|v| v.enabled).collect()
    }
}

/// 环境配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentsConfig {
    pub environments: Vec<Environment>,
    pub active_environment_id: Option<String>,
}

/// 变量替换结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceResult {
    /// 替换后的文本
    pub text: String,
    /// 未替换的变量列表（未定义的变量名）
    pub undefined_variables: Vec<String>,
}

impl std::fmt::Display for ReplaceResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl EnvironmentsConfig {
    pub fn validate(&self) -> Result<(), String> {
        for env in &self.environments {
            env.validate()?;
        }

        for (i, env1) in self.environments.iter().enumerate() {
            for env2 in self.environments.iter().skip(i + 1) {
                if env1.name == env2.name {
                    return Err(format!("环境名称 '{}' 重复", env1.name));
                }
                if env1.id == env2.id {
                    return Err(format!("环境 ID '{}' 重复", env1.id));
                }
            }
        }

        if let Some(ref active_id) = self.active_environment_id {
            if !self.environments.iter().any(|e| e.id == *active_id) {
                return Err(format!("激活环境 ID '{}' 不存在", active_id));
            }
        }

        Ok(())
    }

    pub fn get_active_environment(&self) -> Option<&Environment> {
        if let Some(ref active_id) = self.active_environment_id {
            self.environments.iter().find(|e| e.id == *active_id)
        } else {
            None
        }
    }

    pub fn find_environment_by_id(&self, id: &str) -> Option<&Environment> {
        self.environments.iter().find(|e| e.id == id)
    }
}
