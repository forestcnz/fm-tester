use crate::domain::models::{ScriptKind, ScriptTargetType};

/// 脚本领域服务
///
/// 包含纯业务逻辑，不涉及文件操作。
pub struct ScriptDomainService;

impl ScriptDomainService {
    /// 生成脚本文件名
    ///
    /// 根据目标类型、目标 ID 和脚本种类生成唯一的文件名。
    pub fn generate_script_filename(
        target_type: ScriptTargetType,
        target_id: Option<&str>,
        script_kind: ScriptKind,
    ) -> String {
        let kind_str = if script_kind == ScriptKind::Pre {
            "pre"
        } else {
            "post"
        };

        match target_type {
            ScriptTargetType::Workspace => format!("workspace_{}.js", kind_str),
            ScriptTargetType::Environment => format!(
                "environment_{}_{}.js",
                target_id.unwrap_or("unknown"),
                kind_str
            ),
            ScriptTargetType::Collection => format!(
                "collection_{}_{}.js",
                target_id.unwrap_or("unknown"),
                kind_str
            ),
            ScriptTargetType::Api => {
                format!("api_{}_{}.js", target_id.unwrap_or("unknown"), kind_str)
            }
        }
    }
}
