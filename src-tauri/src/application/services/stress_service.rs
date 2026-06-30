//! 压力测试应用服务
//!
//! 处理压力测试相关的业务逻辑，通过仓储工厂动态获取仓储。

use crate::application::services::WorkspaceDataApplicationService;
use crate::domain::models::{
    Cookie, ScriptInfo, StressParamsConfig, StressTestConfig, StressTestResult,
    StressTestResultIndexEntry,
};
use crate::domain::services::stress::runner_domain::StressTestRunnerCore;
use crate::infrastructure::RepositoryFactory;
use std::collections::HashMap;
use std::sync::Arc;

/// 压力测试应用服务
///
/// 无状态服务，每次调用时通过工厂根据 workspace_id 获取对应仓储
pub struct StressApplicationService;

impl StressApplicationService {
    /// 创建默认实例
    pub fn new() -> Self {
        Self
    }

    /// 创建压力测试运行器
    pub fn create_runner(
        config: StressTestConfig,
        variables: HashMap<String, String>,
        cookies: Vec<Cookie>,
        post_scripts: Vec<ScriptInfo>,
    ) -> Arc<StressTestRunnerCore> {
        Arc::new(StressTestRunnerCore::new(
            config,
            variables,
            cookies,
            post_scripts,
        ))
    }

    /// 获取工作区变量和Cookie（用于压力测试）
    pub fn get_variables_and_cookies(
        workspace_id: &str,
    ) -> Result<(HashMap<String, String>, Vec<Cookie>), String> {
        let config = WorkspaceDataApplicationService::read_environments(workspace_id)?;
        let variables = WorkspaceDataApplicationService::get_active_variables_map(&config);

        let cookie_repo = RepositoryFactory::get_workspace_data_repository();
        let cookies = cookie_repo.get_all_cookies(workspace_id)?;

        Ok((variables, cookies))
    }
}

impl Default for StressApplicationService {
    fn default() -> Self {
        Self::new()
    }
}

impl StressApplicationService {
    /// 获取压测参数配置
    pub fn get_params(workspace_id: &str, api_id: &str) -> Result<StressParamsConfig, String> {
        let repo = RepositoryFactory::get_stress_repository();
        repo.read_params(workspace_id, api_id)
    }

    /// 保存压测参数配置
    pub fn save_params(
        workspace_id: &str,
        api_id: &str,
        config: &StressParamsConfig,
    ) -> Result<(), String> {
        let repo = RepositoryFactory::get_stress_repository();
        repo.save_params(workspace_id, api_id, config)
    }

    /// 获取接口的压测结果列表
    pub fn get_api_results(
        workspace_id: &str,
        api_id: &str,
    ) -> Result<Vec<StressTestResultIndexEntry>, String> {
        let repo = RepositoryFactory::get_stress_repository();
        repo.get_api_results(workspace_id, api_id)
    }

    /// 获取单个压测结果
    pub fn get_result(
        workspace_id: &str,
        api_id: &str,
        id: &str,
    ) -> Result<Option<StressTestResult>, String> {
        let repo = RepositoryFactory::get_stress_repository();
        repo.read_result(workspace_id, api_id, id)
    }

    /// 保存压测结果
    pub fn save_result(workspace_id: &str, result: &StressTestResult) -> Result<(), String> {
        let repo = RepositoryFactory::get_stress_repository();
        repo.save_result(workspace_id, result)
    }

    /// 删除压测结果
    pub fn delete_result(workspace_id: &str, api_id: &str, id: &str) -> Result<(), String> {
        let repo = RepositoryFactory::get_stress_repository();
        repo.delete_result(workspace_id, api_id, id)
    }
}
