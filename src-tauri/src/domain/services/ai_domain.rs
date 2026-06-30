//! AI 领域服务
//!
//! 提供 AI 相关的纯业务逻辑（系统提示生成）。
//! 任务状态管理已移到 Application 层。

use crate::domain::models::{ToolDef, ToolFunctionDef};

/// AI 领域服务（纯业务逻辑）
pub struct AiDomainService;

impl AiDomainService {
    /// 获取脚本优化的系统提示（纯业务逻辑）
    pub fn get_script_system_prompt(script_type: &str) -> &'static str {
        if script_type == "pre" {
            "你是一个API测试脚本专家。请优化或完善用户提供的前置脚本（Pre-request Script）。

前置脚本在请求发送前执行，可以使用以下 fm API：
- fm.environment.get(key) / fm.environment.set(key, value) / fm.environment.getAll() - 环境变量操作
- fm.collection.get(key) / fm.collection.set(key, value) / fm.collection.getAll() - 集合变量操作
- fm.request.getUrl() / fm.request.setUrl(url) - 获取/设置请求URL
- fm.request.getBaseUrl() / fm.request.setBaseUrl(baseUrl) - 获取/设置baseUrl
- fm.request.getPath() / fm.request.setPath(path) - 获取/设置请求路径
- fm.request.getMethod() / fm.request.setMethod(method) - 获取/设置请求方法
- fm.request.getHeader(key) / fm.request.setHeader(key, value) / fm.request.removeHeader(key) / fm.request.getHeaders() - 请求头操作
- fm.request.getBody() / fm.request.setBody(body) - 获取/设置请求体
- fm.log(message) - 输出日志到Console
- fm.assert(condition, message) - 断言检查
- fm.sleep(ms) - 异步等待（毫秒）

请根据用户的需求或现有脚本，优化、完善或生成JavaScript脚本代码。
只返回纯JavaScript代码，不要包含任何解释或markdown格式。"
        } else {
            "你是一个API测试脚本专家。请优化或完善用户提供的后置脚本（Post-request Script）。

后置脚本在响应返回后执行，可以使用以下 fm API（包括前置脚本所有API）：
- fm.response.getStatus() - 获取响应状态码
- fm.response.getStatusText() - 获取响应状态文本
- fm.response.getHeader(key) / fm.response.getHeaders() - 获取响应头
- fm.response.getBody() - 获取响应体（字符串）
- fm.response.getJson() - 获取响应体（JSON对象）
- fm.response.getTime() - 获取响应时间（ms）
- fm.response.getSize() - 获取响应大小（bytes）
- fm.environment.get(key) / fm.environment.set(key, value) / fm.environment.getAll() - 环境变量操作
- fm.collection.get(key) / fm.collection.set(key, value) / fm.collection.getAll() - 集合变量操作
- fm.log(message) - 输出日志到Console
- fm.assert(condition, message) - 断言检查
- fm.sleep(ms) - 异步等待（毫秒）

请根据用户的需求或现有脚本，优化、完善或生成JavaScript脚本代码。
只返回纯JavaScript代码，不要包含任何解释或markdown格式。"
        }
    }

    /// 工作区上下文聊天的系统提示
    pub fn get_workspace_chat_system_prompt() -> &'static str {
        "你是 FM Tester 的 API 测试助手。用户正在当前工作区中进行 API 测试与开发。

你可以调用以下工具查询当前工作区的接口信息：
- list_workspace_apis：列出工作区内所有接口（名称、方法、URL、描述、所属集合路径）
- get_api_detail：获取某个接口的完整定义（请求头、请求体、查询参数、表单字段等）
- get_api_doc：获取某个接口的 Markdown 文档（若已编写）
- get_api_responses：获取某个接口已保存的响应示例（响应体内容）
- get_api_history：获取某个接口的请求历史（状态码、响应时间、响应体）
- search_apis：按关键词搜索接口

回答用户关于接口的问题时，请先调用合适的工具获取准确信息，再基于工具返回的数据作答。
不要凭空猜测接口细节。回答使用中文，技术术语可保留英文。如果工具返回的数据不足，可多次调用不同工具。"
    }

    /// 聊天会话标题总结的系统提示
    pub fn get_chat_summary_prompt() -> &'static str {
        "你是一个标题生成助手。请根据用户的对话内容生成一个简短、准确的中文标题。\n\n\
         要求：\n\
         - 长度控制在 4 到 12 个字之间\n\
         - 概括对话的核心主题或用户意图\n\
         - 不要使用任何标点符号（引号、句号、逗号等）\n\
         - 不要以“关于”、“询问”、“如何”等无意义前缀开头\n\
         - 直接输出标题文本，不要包含任何额外说明、引号或 markdown 格式"
    }

    /// 工作区上下文聊天的工具定义（OpenAI function calling 格式）
    pub fn get_workspace_tools() -> Vec<ToolDef> {
        vec![
            ToolDef {
                tool_type: "function".to_string(),
                function: ToolFunctionDef {
                    name: "list_workspace_apis".to_string(),
                    description:
                        "列出当前工作区的所有 API 接口，返回每个接口的 id、名称、HTTP 方法、URL、描述及所属集合路径。"
                            .to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {}
                    }),
                },
            },
            ToolDef {
                tool_type: "function".to_string(),
                function: ToolFunctionDef {
                    name: "get_api_detail".to_string(),
                    description:
                        "根据接口 id 获取单个接口的完整定义，包含请求头、请求体、查询参数、表单字段等。"
                            .to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "api_id": {
                                "type": "string",
                                "description": "接口 ID（可从 list_workspace_apis 获取）"
                            }
                        },
                        "required": ["api_id"]
                    }),
                },
            },
            ToolDef {
                tool_type: "function".to_string(),
                function: ToolFunctionDef {
                    name: "get_api_doc".to_string(),
                    description: "根据接口 id 获取其 Markdown 格式的 API 文档（若已编写）。"
                        .to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "api_id": {
                                "type": "string",
                                "description": "接口 ID"
                            }
                        },
                        "required": ["api_id"]
                    }),
                },
            },
            ToolDef {
                tool_type: "function".to_string(),
                function: ToolFunctionDef {
                    name: "get_api_responses".to_string(),
                    description: "根据接口 id 获取该接口已保存的响应示例（含响应体内容）。".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "api_id": {
                                "type": "string",
                                "description": "接口 ID"
                            }
                        },
                        "required": ["api_id"]
                    }),
                },
            },
            ToolDef {
                tool_type: "function".to_string(),
                function: ToolFunctionDef {
                    name: "get_api_history".to_string(),
                    description: "根据接口 id 获取该接口的请求历史记录（状态码、响应时间、响应体等）。".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "api_id": {
                                "type": "string",
                                "description": "接口 ID"
                            }
                        },
                        "required": ["api_id"]
                    }),
                },
            },
            ToolDef {
                tool_type: "function".to_string(),
                function: ToolFunctionDef {
                    name: "search_apis".to_string(),
                    description: "按关键词搜索接口（匹配名称、URL、描述），返回匹配的接口列表。"
                        .to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "keyword": {
                                "type": "string",
                                "description": "搜索关键词"
                            }
                        },
                        "required": ["keyword"]
                    }),
                },
            },
        ]
    }
}
