//! 编排仓储接口
//!
//! 定义编排数据持久化的抽象接口，遵循 DDD 依赖反转原则。
//! 领域层通过此接口访问数据，具体实现在基础设施层。

use crate::domain::models::{
    Orchestration, OrchestrationIndex, OrchestrationRun, OrchestrationRunIndex,
};

/// 编排仓储接口
///
/// 负责编排数据的持久化操作，包括：
/// - 编排索引读写
/// - 单个编排文件读写
/// - 编排执行记录读写
pub trait OrchestrationRepository {
    /// 读取编排索引
    fn read_index(&self, workspace_id: &str) -> Result<OrchestrationIndex, String>;

    /// 写入编排索引
    fn write_index(&self, workspace_id: &str, index: &OrchestrationIndex) -> Result<(), String>;

    /// 读取单个编排
    fn read_orchestration(&self, workspace_id: &str, id: &str) -> Result<Orchestration, String>;

    /// 写入单个编排
    fn write_orchestration(
        &self,
        workspace_id: &str,
        orchestration: &Orchestration,
    ) -> Result<(), String>;

    /// 删除编排文件及其执行记录
    fn delete_orchestration(&self, workspace_id: &str, id: &str) -> Result<(), String>;

    /// 读取编排执行索引
    fn read_runs_index(
        &self,
        workspace_id: &str,
        orchestration_id: &str,
    ) -> Result<OrchestrationRunIndex, String>;

    /// 写入编排执行索引
    fn write_runs_index(
        &self,
        workspace_id: &str,
        orchestration_id: &str,
        index: &OrchestrationRunIndex,
    ) -> Result<(), String>;

    /// 读取单个编排执行记录
    fn read_run(
        &self,
        workspace_id: &str,
        orchestration_id: &str,
        run_id: &str,
    ) -> Result<OrchestrationRun, String>;

    /// 写入单个编排执行记录
    fn write_run(
        &self,
        workspace_id: &str,
        orchestration_id: &str,
        run: &OrchestrationRun,
    ) -> Result<(), String>;

    /// 删除编排执行记录
    fn delete_run(
        &self,
        workspace_id: &str,
        orchestration_id: &str,
        run_id: &str,
    ) -> Result<(), String>;

    /// 清空所有执行记录
    fn clear_all_runs(&self, workspace_id: &str, orchestration_id: &str) -> Result<(), String>;

    /// 获取编排目录路径
    fn get_orchestrations_dir(&self, workspace_id: &str) -> std::path::PathBuf;

    /// 获取编排索引文件路径
    fn get_index_path(&self, workspace_id: &str) -> std::path::PathBuf;

    /// 获取单个编排文件路径
    fn get_orchestration_path(&self, workspace_id: &str, id: &str) -> std::path::PathBuf;

    /// 获取执行记录目录路径
    fn get_runs_dir(&self, workspace_id: &str, orchestration_id: &str) -> std::path::PathBuf;

    /// 获取执行记录索引文件路径
    fn get_runs_index_path(&self, workspace_id: &str, orchestration_id: &str)
        -> std::path::PathBuf;

    /// 获取单个执行记录文件路径
    fn get_run_path(
        &self,
        workspace_id: &str,
        orchestration_id: &str,
        run_id: &str,
    ) -> std::path::PathBuf;
}
