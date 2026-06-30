//! 压力测试仓储接口
//!
//! 定义压力测试相关数据的持久化接口，符合DDD依赖反转原则。

use crate::domain::models::{StressParamsConfig, StressTestResult, StressTestResultIndexEntry};

/// 压力测试仓储接口
pub trait StressTestRepository {
    /// 读取压测参数配置
    fn read_params(&self, workspace_id: &str, api_id: &str) -> Result<StressParamsConfig, String>;

    /// 保存压测参数配置
    fn save_params(
        &self,
        workspace_id: &str,
        api_id: &str,
        config: &StressParamsConfig,
    ) -> Result<(), String>;

    /// 保存压测结果
    fn save_result(&self, workspace_id: &str, result: &StressTestResult) -> Result<(), String>;

    /// 读取单个压测结果
    fn read_result(
        &self,
        workspace_id: &str,
        api_id: &str,
        id: &str,
    ) -> Result<Option<StressTestResult>, String>;

    /// 删除压测结果
    fn delete_result(&self, workspace_id: &str, api_id: &str, id: &str) -> Result<(), String>;

    /// 获取接口的压测结果列表
    fn get_api_results(
        &self,
        workspace_id: &str,
        api_id: &str,
    ) -> Result<Vec<StressTestResultIndexEntry>, String>;
}
