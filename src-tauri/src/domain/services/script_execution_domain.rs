//! 脚本执行领域服务
//!
//! 定义脚本执行的业务规则，不依赖具体的 JS runtime 实现。

use crate::domain::models::{ScriptInfo, ScriptKind};

/// 脚本执行领域服务
pub struct ScriptExecutionDomainService;

impl ScriptExecutionDomainService {
    /// 对脚本链进行排序（根据脚本类型）
    ///
    /// 前置脚本顺序：工作区 → 环境 → 集合（父 → 子） → 接口
    /// 后置脚本顺序：接口 → 集合（子 → 父） → 环境 → 工作区（反向）
    pub fn order_script_chain(scripts: Vec<ScriptInfo>, kind: ScriptKind) -> Vec<ScriptInfo> {
        if kind == ScriptKind::Post {
            // 后置脚本反向执行
            scripts.into_iter().rev().collect()
        } else {
            scripts
        }
    }

    /// 检查脚本是否为空
    pub fn is_empty_script(content: &str) -> bool {
        content.trim().is_empty()
    }

    /// 生成脚本来源标识
    pub fn generate_source_identifier(
        target_type: crate::domain::models::ScriptTargetType,
        target_id: Option<&str>,
        target_name: Option<&str>,
    ) -> String {
        match target_type {
            crate::domain::models::ScriptTargetType::Workspace => "workspace".to_string(),
            crate::domain::models::ScriptTargetType::Environment => "environment".to_string(),
            crate::domain::models::ScriptTargetType::Collection => {
                if let Some(name) = target_name {
                    format!("collection:{}", name)
                } else {
                    format!("collection:{}", target_id.unwrap_or("unknown"))
                }
            }
            crate::domain::models::ScriptTargetType::Api => "api".to_string(),
        }
    }
}
