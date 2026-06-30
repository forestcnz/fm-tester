//! 编排应用服务
//!
//! 处理编排相关的 UI 交互，协调 Repository 和 DomainService。
//! 通过 DomainService 进行业务逻辑处理，Repository 调用集中化。

use crate::domain::models::{
    Orchestration, OrchestrationIndex, OrchestrationRun, OrchestrationRunIndex, OrchestrationStep,
    StepRunResult,
};
use crate::domain::services::OrchestrationDomainService;
use crate::infrastructure::RepositoryFactory;

/// 编排应用服务
///
/// 无状态服务，通过 Domain Service 进行业务逻辑处理，
/// Repository 通过工厂函数获取，遵循 DDD 架构规范。
/// 编排应用服务
///
/// 无状态服务，每次调用时通过工厂根据 workspace_id 获取对应仓储
pub struct OrchestrationApplicationService;

impl OrchestrationApplicationService {
    /// 创建默认实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for OrchestrationApplicationService {
    fn default() -> Self {
        Self::new()
    }
}

impl OrchestrationApplicationService {
    /// 获取编排索引
    pub fn get_orchestrations(workspace_id: &str) -> Result<OrchestrationIndex, String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        repo.read_index(workspace_id)
    }

    /// 获取单个编排
    pub fn get_orchestration(
        workspace_id: &str,
        orchestration_id: &str,
    ) -> Result<Orchestration, String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        repo.read_orchestration(workspace_id, orchestration_id)
    }

    /// 创建编排
    pub fn create_orchestration(
        workspace_id: &str,
        name: String,
        description: Option<String>,
    ) -> Result<Orchestration, String> {
        // Domain Service: 业务验证和实体创建
        OrchestrationDomainService::validate_name(&name)?;
        let orchestration =
            OrchestrationDomainService::create_orchestration_entity(name, description);

        // 持久化
        let repo = RepositoryFactory::get_orchestration_repository();
        repo.write_orchestration(workspace_id, &orchestration)?;

        // 更新索引（使用 Domain Service 创建索引条目）
        let index_entry = OrchestrationDomainService::create_index_entry(&orchestration);
        let mut index = repo.read_index(workspace_id)?;
        index.orchestrations.push(index_entry);
        repo.write_index(workspace_id, &index)?;

        Ok(orchestration)
    }

    /// 更新编排
    pub fn update_orchestration(
        workspace_id: &str,
        orchestration_id: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Orchestration, String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        let mut orchestration = repo.read_orchestration(workspace_id, orchestration_id)?;

        // Domain Service: 业务验证和更新
        if let Some(n) = &name {
            OrchestrationDomainService::validate_name(n)?;
            orchestration.name = n.clone();
        }
        if let Some(d) = description {
            orchestration.description = Some(d);
        }
        OrchestrationDomainService::update_timestamp(&mut orchestration);

        // 持久化
        repo.write_orchestration(workspace_id, &orchestration)?;

        // 更新索引
        let mut index = repo.read_index(workspace_id)?;
        if let Some(entry) = index
            .orchestrations
            .iter_mut()
            .find(|e| e.id == orchestration_id)
        {
            entry.name = orchestration.name.clone();
            entry.description = orchestration.description.clone();
            entry.updated_at = orchestration.updated_at.clone();
            repo.write_index(workspace_id, &index)?;
        }

        Ok(orchestration)
    }

    /// 删除编排
    pub fn delete_orchestration(workspace_id: &str, orchestration_id: &str) -> Result<(), String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        repo.delete_orchestration(workspace_id, orchestration_id)?;

        // 更新索引
        let mut index = repo.read_index(workspace_id)?;
        index.orchestrations.retain(|e| e.id != orchestration_id);
        repo.write_index(workspace_id, &index)?;

        Ok(())
    }

    /// 添加步骤
    pub fn add_orchestration_step(
        workspace_id: &str,
        orchestration_id: &str,
        api_id: String,
        name: Option<String>,
        enabled: bool,
        wait_before: u64,
        retry_count: u32,
        retry_delay: u64,
        on_failure: String,
    ) -> Result<OrchestrationStep, String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        let mut orchestration = repo.read_orchestration(workspace_id, orchestration_id)?;

        // Domain Service: 创建步骤并添加
        let step = OrchestrationDomainService::create_step_entity(
            api_id,
            name,
            enabled,
            wait_before,
            retry_count,
            retry_delay,
            on_failure,
        );
        OrchestrationDomainService::add_step(&mut orchestration, step.clone());
        OrchestrationDomainService::update_timestamp(&mut orchestration);

        // 持久化
        repo.write_orchestration(workspace_id, &orchestration)?;

        // 更新索引
        let mut index = repo.read_index(workspace_id)?;
        if let Some(entry) = index
            .orchestrations
            .iter_mut()
            .find(|e| e.id == orchestration_id)
        {
            entry.step_count = orchestration.steps.len();
            entry.updated_at = orchestration.updated_at.clone();
            repo.write_index(workspace_id, &index)?;
        }

        Ok(step)
    }

    /// 更新步骤
    pub fn update_orchestration_step(
        workspace_id: &str,
        orchestration_id: &str,
        step_id: &str,
        name: Option<String>,
        enabled: Option<bool>,
        wait_before: Option<u64>,
        retry_count: Option<u32>,
        retry_delay: Option<u64>,
        on_failure: Option<String>,
    ) -> Result<OrchestrationStep, String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        let mut orchestration = repo.read_orchestration(workspace_id, orchestration_id)?;

        // Domain Service: 查找并更新步骤
        let step = OrchestrationDomainService::find_step(&mut orchestration, step_id)
            .ok_or("步骤不存在".to_string())?;
        OrchestrationDomainService::update_step(
            step,
            name,
            enabled,
            wait_before,
            retry_count,
            retry_delay,
            on_failure,
        );
        let result_step = step.clone();

        OrchestrationDomainService::update_timestamp(&mut orchestration);
        repo.write_orchestration(workspace_id, &orchestration)?;

        // 更新索引
        let mut index = repo.read_index(workspace_id)?;
        if let Some(entry) = index
            .orchestrations
            .iter_mut()
            .find(|e| e.id == orchestration_id)
        {
            entry.updated_at = orchestration.updated_at.clone();
            repo.write_index(workspace_id, &index)?;
        }

        Ok(result_step)
    }

    /// 移除步骤
    pub fn remove_orchestration_step(
        workspace_id: &str,
        orchestration_id: &str,
        step_id: &str,
    ) -> Result<(), String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        let mut orchestration = repo.read_orchestration(workspace_id, orchestration_id)?;

        // Domain Service: 移除步骤
        OrchestrationDomainService::remove_step(&mut orchestration, step_id);
        OrchestrationDomainService::update_timestamp(&mut orchestration);

        repo.write_orchestration(workspace_id, &orchestration)?;

        // 更新索引
        let mut index = repo.read_index(workspace_id)?;
        if let Some(entry) = index
            .orchestrations
            .iter_mut()
            .find(|e| e.id == orchestration_id)
        {
            entry.step_count = orchestration.steps.len();
            entry.updated_at = orchestration.updated_at.clone();
            repo.write_index(workspace_id, &index)?;
        }

        Ok(())
    }

    /// 重排步骤顺序
    pub fn reorder_orchestration_steps(
        workspace_id: &str,
        orchestration_id: &str,
        step_ids: Vec<String>,
    ) -> Result<(), String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        let mut orchestration = repo.read_orchestration(workspace_id, orchestration_id)?;

        // Domain Service: 重排步骤
        OrchestrationDomainService::reorder_steps(&mut orchestration, &step_ids)?;
        OrchestrationDomainService::update_timestamp(&mut orchestration);

        repo.write_orchestration(workspace_id, &orchestration)?;

        // 更新索引
        let mut index = repo.read_index(workspace_id)?;
        if let Some(entry) = index
            .orchestrations
            .iter_mut()
            .find(|e| e.id == orchestration_id)
        {
            entry.updated_at = orchestration.updated_at.clone();
            repo.write_index(workspace_id, &index)?;
        }

        Ok(())
    }

    /// 获取执行记录索引
    pub fn get_orchestration_runs(
        workspace_id: &str,
        orchestration_id: &str,
    ) -> Result<OrchestrationRunIndex, String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        repo.read_runs_index(workspace_id, orchestration_id)
    }

    /// 获取单个执行记录
    pub fn get_orchestration_run(
        workspace_id: &str,
        orchestration_id: &str,
        run_id: &str,
    ) -> Result<OrchestrationRun, String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        repo.read_run(workspace_id, orchestration_id, run_id)
    }

    /// 创建执行记录
    pub fn create_orchestration_run(
        workspace_id: &str,
        orchestration_id: &str,
    ) -> Result<OrchestrationRun, String> {
        // Domain Service: 创建执行记录实体
        let run = OrchestrationDomainService::create_run_entity(orchestration_id);

        let repo = RepositoryFactory::get_orchestration_repository();
        // write_run 已经完成写入，索引是从 runs 文件动态生成的，不需要单独操作索引
        repo.write_run(workspace_id, orchestration_id, &run)?;

        Ok(run)
    }

    /// 更新执行步骤结果
    pub fn update_orchestration_run_step(
        workspace_id: &str,
        orchestration_id: &str,
        run_id: &str,
        step_result: StepRunResult,
    ) -> Result<(), String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        let mut run = repo.read_run(workspace_id, orchestration_id, run_id)?;

        // Domain Service: 更新统计和步骤结果
        OrchestrationDomainService::update_run_statistics(&mut run, &step_result);
        OrchestrationDomainService::add_or_update_step_result(&mut run, step_result);

        // 直接写入数据库，不需要单独操作索引
        // read_runs_index 直接从数据库表查询，会自动反映最新数据
        repo.write_run(workspace_id, orchestration_id, &run)?;

        Ok(())
    }

    /// 完成执行记录
    pub fn complete_orchestration_run(
        workspace_id: &str,
        orchestration_id: &str,
        run_id: &str,
        status: String,
        end_time: String,
        total_time: u64,
    ) -> Result<OrchestrationRun, String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        let mut run = repo.read_run(workspace_id, orchestration_id, run_id)?;

        // Domain Service: 完成执行记录
        OrchestrationDomainService::complete_run(&mut run, status, end_time, total_time);

        // 直接写入数据库，不需要单独操作索引
        repo.write_run(workspace_id, orchestration_id, &run)?;

        Ok(run)
    }

    /// 删除执行记录
    pub fn delete_orchestration_run(
        workspace_id: &str,
        orchestration_id: &str,
        run_id: &str,
    ) -> Result<(), String> {
        let repo = RepositoryFactory::get_orchestration_repository();
        repo.delete_run(workspace_id, orchestration_id, run_id)?;

        Ok(())
    }

    /// 清空所有执行记录
    pub fn clear_orchestration_runs(
        workspace_id: &str,
        orchestration_id: &str,
    ) -> Result<(), String> {
        let repo = RepositoryFactory::get_orchestration_repository();

        // 直接清空数据库表
        repo.clear_all_runs(workspace_id, orchestration_id)?;

        Ok(())
    }

    /// 更新编排调度配置
    ///
    /// # 参数
    /// - `workspace_id`: 工作区路径
    /// - `orchestration_id`: 编排 ID
    /// - `schedule`: 调度配置
    ///
    /// # 返回
    /// - `Ok(Orchestration)`: 更新后的编排
    /// - `Err(String)`: 更新失败
    pub fn update_orchestration_schedule(
        workspace_id: &str,
        orchestration_id: &str,
        schedule: crate::domain::models::OrchestrationSchedule,
    ) -> Result<Orchestration, String> {
        // 验证调度配置
        schedule.validate()?;

        let repo = RepositoryFactory::get_orchestration_repository();
        let mut orchestration = repo.read_orchestration(workspace_id, orchestration_id)?;

        // 更新调度配置
        orchestration.schedule = Some(schedule.clone());
        OrchestrationDomainService::update_timestamp(&mut orchestration);

        // 持久化
        repo.write_orchestration(workspace_id, &orchestration)?;

        // 更新索引
        let mut index = repo.read_index(workspace_id)?;
        if let Some(entry) = index
            .orchestrations
            .iter_mut()
            .find(|e| e.id == orchestration_id)
        {
            entry.updated_at = orchestration.updated_at.clone();
            repo.write_index(workspace_id, &index)?;
        }

        Ok(orchestration)
    }

    /// 获取下次执行时间
    ///
    /// # 参数
    /// - `cron_expression`: Cron 表达式（Quartz 格式）
    ///
    /// # 返回
    /// - `Ok(String)`: 下次执行时间（格式：YYYY-MM-DD HH:MM:SS）
    /// - `Err(String)`: 计算失败
    pub fn get_next_run_time(cron_expression: &str) -> Result<String, String> {
        use chrono::{Local, TimeZone};
        use croner::Cron;

        // 解析 cron 表达式（Quartz 格式）
        let cron = Cron::new(cron_expression)
            .with_alternative_weekdays() // Quartz 格式：周几 1-7（周日=1）
            .with_seconds_required() // 要求秒字段（6位格式）
            .parse()
            .map_err(|e| format!("Cron 表达式解析失败 (Quartz 格式): {}", e))?;

        // 计算下次执行时间
        let now = Local::now();
        let next_run = cron
            .find_next_occurrence(&now, false)
            .map_err(|_| "无法计算下次执行时间")?;

        // 转换为本地时间并格式化
        let local_time = Local
            .from_local_datetime(&next_run.naive_local())
            .single()
            .unwrap_or(next_run);
        Ok(local_time.format("%Y-%m-%d %H:%M:%S").to_string())
    }

    /// 计算最近 N 次执行时间
    ///
    /// # 参数
    /// - `cron_expression`: Cron 表达式（Quartz 格式，6位）
    /// - `count`: 要计算的次数
    ///
    /// # 返回
    /// - `Ok(Vec<String>)`: 最近 N 次执行时间列表（格式：YYYY-MM-DD HH:MM:SS）
    /// - `Err(String)`: 计算失败
    pub fn get_next_run_times(cron_expression: &str, count: usize) -> Result<Vec<String>, String> {
        use chrono::{Local, TimeZone};
        use croner::Cron;

        // 解析 cron 表达式（Quartz 格式）
        let cron = Cron::new(cron_expression)
            .with_alternative_weekdays()
            .with_seconds_required()
            .parse()
            .map_err(|e| format!("Cron 表达式解析失败 (Quartz 格式): {}", e))?;

        let now = Local::now();
        let mut times = Vec::new();
        let mut current_time = now;

        for _ in 0..count {
            let next_run = cron
                .find_next_occurrence(&current_time, false)
                .map_err(|_| "无法计算下次执行时间")?;

            let local_time = Local
                .from_local_datetime(&next_run.naive_local())
                .single()
                .unwrap_or(next_run);
            times.push(local_time.format("%Y-%m-%d %H:%M:%S").to_string());

            // 移动到下一个时间点，继续查找
            current_time = next_run + chrono::Duration::seconds(1);
        }

        Ok(times)
    }

    /// 重排编排列表顺序
    ///
    /// # 参数
    /// - `workspace_id`: 工作区路径
    /// - `orchestration_ids`: 新的编排 ID 顺序列表
    ///
    /// # 返回
    /// - `Ok(())`: 排序成功
    /// - `Err(String)`: 排序失败
    pub fn reorder_orchestrations(
        workspace_id: &str,
        orchestration_ids: Vec<String>,
    ) -> Result<(), String> {
        let repo = RepositoryFactory::get_orchestration_repository();

        // 读取现有索引
        let mut index = repo.read_index(workspace_id)?;

        // 验证所有 ID 都存在
        let existing_ids: Vec<String> = index.orchestrations.iter().map(|e| e.id.clone()).collect();
        if orchestration_ids.len() != existing_ids.len() {
            return Err("编排列表长度不匹配".to_string());
        }

        for id in &orchestration_ids {
            if !existing_ids.contains(id) {
                return Err(format!("编排 ID {} 不存在", id));
            }
        }

        // 按新顺序重新排列索引条目
        let mut new_entries = Vec::with_capacity(orchestration_ids.len());
        for id in &orchestration_ids {
            if let Some(entry) = index.orchestrations.iter().find(|e| &e.id == id) {
                new_entries.push(entry.clone());
            }
        }

        // 更新索引
        index.orchestrations = new_entries;
        repo.write_index(workspace_id, &index)?;

        Ok(())
    }
}
