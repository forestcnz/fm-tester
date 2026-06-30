//! Git 工作区备份服务
//!
//! 将工作区数据导出为 JSON，通过 git2 推送到备份仓库；
//! 支持列举备份文件与从备份恢复（导入为新工作区）。
//!
//! 本地缓存目录：./data/.git_backup_cache （复用，避免每次全量克隆）
//! 远程目录结构：workspaces/<工作区名>/<工作区名>_<时间戳>.json

use crate::application::services::WorkspaceIOService;
use crate::domain::models::{GitBackupFile, GitBackupSettings, Workspace};
use crate::domain::services::EncryptionService;
use crate::infrastructure::{data_dir, get_encryption_service, RepositoryFactory};
use crate::repo_error;
use chrono::Local;
use git2::{Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository, ResetType, Signature};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const CACHE_DIR_NAME: &str = ".git_backup_cache";
const BACKUPS_SUBDIR: &str = "workspaces";
const BACKUP_AUTHOR_NAME: &str = "FM-Tester";
const BACKUP_AUTHOR_EMAIL: &str = "backup@fm-tester.local";

/// 缓存目录路径：./data/.git_backup_cache
fn cache_dir() -> PathBuf {
    data_dir::get_data_dir().join(CACHE_DIR_NAME)
}

/// 读取 Git 备份配置并解密密码
fn load_config_with_credentials() -> Result<(GitBackupSettings, String), String> {
    let config = RepositoryFactory::get_app_config_repository().read()?;
    let settings = config.settings.git_backup.clone();
    let password = if settings.encrypted_password.is_empty() {
        String::new()
    } else {
        get_encryption_service().decrypt(&settings.encrypted_password)?
    };
    Ok((settings, password))
}

/// 构造带认证的回调（含密码错误防重入）
fn make_callbacks(username: &str, password: &str) -> RemoteCallbacks<'static> {
    let tried = Arc::new(AtomicBool::new(false));
    let u = username.to_string();
    let p = password.to_string();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, _username, _allowed_types| {
        // git2 在认证失败时会反复调用回调，用标志位阻止无限循环
        if tried.swap(true, Ordering::SeqCst) {
            return Err(git2::Error::from_str("Git 认证失败：用户名或密码错误"));
        }
        Cred::userpass_plaintext(&u, &p)
    });
    callbacks
}

/// 测试连接：返回远程分支列表
pub fn test_connection() -> Result<Vec<String>, String> {
    let (settings, password) = load_config_with_credentials()?;
    if !settings.is_configured() {
        return Err(repo_error!("尚未配置 Git 仓库地址"));
    }
    let url = settings.repo_url.trim();
    let username = settings.username.trim();

    let mut remote =
        git2::Remote::create_detached(url).map_err(|e| repo_error!("创建远程连接失败: {}", e))?;
    let callbacks = make_callbacks(username, &password);
    remote
        .connect_auth(git2::Direction::Fetch, Some(callbacks), None)
        .map_err(|e| repo_error!("连接远程仓库失败: {}", e))?;
    let list = remote
        .list()
        .map_err(|e| repo_error!("读取远程引用失败: {}", e))?;
    let branches: Vec<String> = list
        .iter()
        .map(|h| h.name().to_string())
        .filter(|n| n.starts_with("refs/heads/"))
        .map(|n| n.trim_start_matches("refs/heads/").to_string())
        .collect();
    let _ = remote.disconnect();
    Ok(branches)
}

/// 准备缓存仓库：已存在则打开并拉取更新，否则克隆
fn prepare_cache_repo(
    url: &str,
    branch: &str,
    username: &str,
    password: &str,
) -> Result<Repository, String> {
    let cache = cache_dir();
    let git_dir = cache.join(".git");

    if git_dir.exists() {
        let repo = Repository::open(&cache).map_err(|e| repo_error!("打开缓存仓库失败: {}", e))?;

        // 若仓库地址已变更，清空缓存重新克隆
        let origin_url = repo
            .find_remote("origin")
            .ok()
            .and_then(|r| r.url().map(|s| s.to_string()));
        if origin_url.as_deref() != Some(url) {
            drop(repo);
            std::fs::remove_dir_all(&cache).map_err(|e| repo_error!("清理缓存目录失败: {}", e))?;
            return clone_fresh(url, branch, username, password, &cache);
        }

        fetch_and_ff(&repo, url, branch, username, password)?;
        Ok(repo)
    } else {
        clone_fresh(url, branch, username, password, &cache)
    }
}

/// 全新克隆并切换到目标分支
fn clone_fresh(
    url: &str,
    branch: &str,
    username: &str,
    password: &str,
    cache: &Path,
) -> Result<Repository, String> {
    if cache.exists() {
        std::fs::remove_dir_all(cache).map_err(|e| repo_error!("清理缓存目录失败: {}", e))?;
    }
    let mut fetch_options = FetchOptions::new();
    let callbacks = make_callbacks(username, password);
    fetch_options.remote_callbacks(callbacks);
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);
    let repo = builder
        .clone(url, cache)
        .map_err(|e| repo_error!("克隆远程仓库失败: {}", e))?;
    setup_branch(&repo, branch)?;
    Ok(repo)
}

/// 克隆后切换到目标分支（兼容空仓库与远程默认分支不同的情况）
fn setup_branch(repo: &Repository, branch: &str) -> Result<(), String> {
    let local_ref = format!("refs/heads/{}", branch);
    let remote_ref = format!("refs/remotes/origin/{}", branch);

    if let Ok(remote_oid) = repo.refname_to_id(&remote_ref) {
        // 远程存在该分支：建立本地跟踪分支并检出
        repo.reference(&local_ref, remote_oid, true, "checkout backup branch")
            .map_err(|e| repo_error!("创建本地分支失败: {}", e))?;
        repo.set_head(&local_ref)
            .map_err(|e| repo_error!("切换 HEAD 失败: {}", e))?;
        let commit = repo
            .find_commit(remote_oid)
            .map_err(|e| repo_error!("查找提交失败: {}", e))?;
        repo.reset(commit.as_object(), ResetType::Hard, None)
            .map_err(|e| repo_error!("重置工作区失败: {}", e))?;
    } else {
        // 远程无此分支（多为空仓库）：仅设置 HEAD 指向目标分支（首次提交时创建）
        repo.set_head(&local_ref)
            .map_err(|e| repo_error!("设置 HEAD 失败: {}", e))?;
    }
    Ok(())
}

/// 拉取远程并同步本地分支
fn fetch_and_ff(
    repo: &Repository,
    url: &str,
    branch: &str,
    username: &str,
    password: &str,
) -> Result<(), String> {
    if repo.find_remote("origin").is_err() {
        repo.remote("origin", url)
            .map_err(|e| repo_error!("创建 origin 失败: {}", e))?;
    } else {
        let _ = repo.remote_set_url("origin", url);
    }

    let refspec = format!("refs/heads/{}:refs/remotes/origin/{}", branch, branch);
    {
        let mut remote = repo
            .find_remote("origin")
            .map_err(|e| repo_error!("查找 origin 失败: {}", e))?;
        let mut fetch_options = FetchOptions::new();
        let callbacks = make_callbacks(username, password);
        fetch_options.remote_callbacks(callbacks);
        fetch_options.prune(git2::FetchPrune::On);
        remote
            .fetch(&[&refspec], Some(&mut fetch_options), None)
            .map_err(|e| repo_error!("拉取远程失败: {}", e))?;
    }

    let remote_ref = format!("refs/remotes/origin/{}", branch);
    let remote_oid = match repo.refname_to_id(&remote_ref) {
        Ok(oid) => oid,
        Err(_) => return Ok(()), // 远程无此分支
    };

    // 强制本地分支指向远程，并重置工作目录
    let local_ref = format!("refs/heads/{}", branch);
    repo.reference(&local_ref, remote_oid, true, "sync to remote")
        .map_err(|e| repo_error!("更新本地分支失败: {}", e))?;
    repo.set_head(&local_ref)
        .map_err(|e| repo_error!("切换 HEAD 失败: {}", e))?;
    let commit = repo
        .find_commit(remote_oid)
        .map_err(|e| repo_error!("查找提交失败: {}", e))?;
    repo.reset(commit.as_object(), ResetType::Hard, None)
        .map_err(|e| repo_error!("重置工作区失败: {}", e))?;
    Ok(())
}

/// 提交并推送单个文件
fn commit_and_push(
    repo: &Repository,
    branch: &str,
    username: &str,
    password: &str,
    rel_path: &Path,
    message: &str,
    is_removal: bool,
) -> Result<(), String> {
    let mut index = repo
        .index()
        .map_err(|e| repo_error!("读取索引失败: {}", e))?;
    if is_removal {
        index
            .remove_path(rel_path)
            .map_err(|e| repo_error!("git rm 失败: {}", e))?;
    } else {
        index
            .add_path(rel_path)
            .map_err(|e| repo_error!("git add 失败: {}", e))?;
    }
    index
        .write()
        .map_err(|e| repo_error!("写入索引失败: {}", e))?;
    let tree_id = index
        .write_tree()
        .map_err(|e| repo_error!("生成树失败: {}", e))?;
    let tree = repo
        .find_tree(tree_id)
        .map_err(|e| repo_error!("查找树失败: {}", e))?;
    let signature = Signature::now(BACKUP_AUTHOR_NAME, BACKUP_AUTHOR_EMAIL)
        .map_err(|e| repo_error!("创建签名失败: {}", e))?;

    let parent = repo
        .head()
        .ok()
        .and_then(|h| h.target())
        .and_then(|oid| repo.find_commit(oid).ok());

    match &parent {
        Some(p) => {
            repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[p])
                .map_err(|e| repo_error!("提交失败: {}", e))?;
        }
        None => {
            repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])
                .map_err(|e| repo_error!("初始提交失败: {}", e))?;
        }
    }

    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| repo_error!("查找 origin 失败: {}", e))?;
    let mut push_options = PushOptions::new();
    let callbacks = make_callbacks(username, password);
    push_options.remote_callbacks(callbacks);
    let push_ref = format!("refs/heads/{}:refs/heads/{}", branch, branch);
    remote
        .push(&[&push_ref], Some(&mut push_options))
        .map_err(|e| repo_error!("git push 失败: {}", e))?;
    Ok(())
}

/// 工作区名清洗为合法的文件/目录名
fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .trim_end_matches('.')
        .to_string();
    if sanitized.is_empty() {
        "workspace".to_string()
    } else {
        sanitized
    }
}

/// 更新工作区的上次备份时间
fn update_last_backup_at(workspace_id: &str) -> Result<(), String> {
    let repo = RepositoryFactory::get_app_config_repository();
    let mut config = repo.read()?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    for ws in &mut config.workspaces {
        if ws.id == workspace_id {
            ws.last_backup_at = Some(now);
            break;
        }
    }
    repo.write(&config)?;
    Ok(())
}

/// 备份工作区：导出 JSON → 写入缓存仓库 → 提交 → 推送
pub fn backup_workspace(workspace_id: &str) -> Result<String, String> {
    tracing::info!("开始备份工作区: {}", workspace_id);
    let (settings, password) = load_config_with_credentials()?;
    if !settings.is_configured() {
        return Err(repo_error!(
            "尚未配置备份设备，请先在设置中配置 Git 仓库地址"
        ));
    }
    let url = settings.repo_url.trim().to_string();
    let branch = settings.branch.trim().to_string();
    let username = settings.username.trim().to_string();

    let repo = prepare_cache_repo(&url, &branch, &username, &password)?;

    // 读取工作区元信息
    let config = RepositoryFactory::get_app_config_repository().read()?;
    let workspace = config
        .workspaces
        .iter()
        .find(|w| w.id == workspace_id)
        .ok_or_else(|| repo_error!("工作区不存在: {}", workspace_id))?;
    let ws_name = workspace.name.clone();

    // 导出工作区数据
    let io_service = WorkspaceIOService::new();
    let json = io_service.export_workspace(workspace_id)?;

    // 写入备份文件
    let safe_name = sanitize_filename(&ws_name);
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let rel_dir = format!("{}/{}", BACKUPS_SUBDIR, safe_name);
    let file_name = format!("{}_{}.json", safe_name, timestamp);
    let abs_dir = cache_dir().join(&rel_dir);
    std::fs::create_dir_all(&abs_dir).map_err(|e| repo_error!("创建备份目录失败: {}", e))?;
    let abs_file = abs_dir.join(&file_name);
    std::fs::write(&abs_file, &json).map_err(|e| repo_error!("写入备份文件失败: {}", e))?;

    // 提交并推送
    let rel_path = Path::new(&rel_dir).join(&file_name);
    let message = format!("backup: {} ({})", ws_name, timestamp);
    commit_and_push(
        &repo, &branch, &username, &password, &rel_path, &message, false,
    )?;

    // 更新该工作区的上次备份时间
    update_last_backup_at(workspace_id)?;

    Ok(file_name)
}

/// 列举备份仓库中的所有备份文件（从 Git 树读取，确保与远程一致）
pub fn list_backups() -> Result<Vec<GitBackupFile>, String> {
    let (settings, password) = load_config_with_credentials()?;
    if !settings.is_configured() {
        return Err(repo_error!("尚未配置备份设备"));
    }
    let repo = prepare_cache_repo(
        settings.repo_url.trim(),
        settings.branch.trim(),
        settings.username.trim(),
        &password,
    )?;

    let remote_ref = format!("refs/remotes/origin/{}", settings.branch.trim());
    let commit_oid = match repo.refname_to_id(&remote_ref) {
        Ok(oid) => oid,
        Err(_) => return Ok(Vec::new()), // 远程无此分支，返回空列表
    };
    let commit = repo
        .find_commit(commit_oid)
        .map_err(|e| repo_error!("查找提交失败: {}", e))?;
    let tree = commit
        .tree()
        .map_err(|e| repo_error!("读取树失败: {}", e))?;

    let mut files = Vec::new();
    let re = Regex::new(r"(\d{8}_\d{6})\.json$").map_err(|e| repo_error!("编译正则失败: {}", e))?;

    // 遍历 workspaces 目录下的子目录
    if let Some(ws_tree_entry) = tree.get_name(BACKUPS_SUBDIR) {
        let ws_tree_obj = ws_tree_entry
            .to_object(&repo)
            .map_err(|e| repo_error!("读取 workspaces 目录失败: {}", e))?;
        if let Some(ws_tree) = ws_tree_obj.as_tree() {
            for ws_entry in ws_tree.iter() {
                let ws_name = ws_entry.name().unwrap_or("").to_string();
                if ws_entry.kind() != Some(git2::ObjectType::Tree) {
                    continue;
                }
                let ws_subtree_obj = ws_entry
                    .to_object(&repo)
                    .map_err(|e| repo_error!("读取工作区目录失败: {}", e))?;
                if let Some(ws_subtree) = ws_subtree_obj.as_tree() {
                    for file_entry in ws_subtree.iter() {
                        let fname = file_entry.name().unwrap_or("").to_string();
                        if !fname.ends_with(".json") {
                            continue;
                        }
                        let timestamp = re
                            .captures(&fname)
                            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
                            .unwrap_or_default();
                        let size = file_entry
                            .to_object(&repo)
                            .ok()
                            .and_then(|obj| obj.as_blob().map(|b| b.size()))
                            .unwrap_or(0);
                        files.push(GitBackupFile {
                            workspace_name: ws_name.clone(),
                            file_name: fname,
                            timestamp,
                            size: size as u64,
                        });
                    }
                }
            }
        }
    }

    files.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(files)
}

/// 从备份恢复：读取 JSON 并导入为新工作区
pub fn restore_from_backup(
    workspace_name: &str,
    file_name: &str,
    new_name: Option<String>,
) -> Result<Workspace, String> {
    tracing::info!(
        "从备份恢复（新建）: {}/{} -> {:?}",
        workspace_name,
        file_name,
        new_name
    );
    let (settings, password) = load_config_with_credentials()?;
    if !settings.is_configured() {
        return Err(repo_error!("尚未配置备份设备"));
    }
    // 确保缓存仓库为最新
    let _repo = prepare_cache_repo(
        settings.repo_url.trim(),
        settings.branch.trim(),
        settings.username.trim(),
        &password,
    )?;

    let path = cache_dir()
        .join(BACKUPS_SUBDIR)
        .join(workspace_name)
        .join(file_name);
    let json =
        std::fs::read_to_string(&path).map_err(|e| repo_error!("读取备份文件失败: {}", e))?;

    let io_service = WorkspaceIOService::new();
    io_service.import_workspace(&json, new_name)
}

/// 从备份覆盖恢复到指定工作区（保留目标工作区 id 与名称，替换全部数据）
pub fn restore_into_workspace(
    target_workspace_id: &str,
    workspace_name: &str,
    file_name: &str,
) -> Result<Workspace, String> {
    tracing::info!(
        "从备份覆盖恢复: {}/{} -> workspace {}",
        workspace_name,
        file_name,
        target_workspace_id
    );
    let (settings, password) = load_config_with_credentials()?;
    if !settings.is_configured() {
        return Err(repo_error!("尚未配置备份设备"));
    }
    // 确保缓存仓库为最新
    let _repo = prepare_cache_repo(
        settings.repo_url.trim(),
        settings.branch.trim(),
        settings.username.trim(),
        &password,
    )?;

    let path = cache_dir()
        .join(BACKUPS_SUBDIR)
        .join(workspace_name)
        .join(file_name);
    let json =
        std::fs::read_to_string(&path).map_err(|e| repo_error!("读取备份文件失败: {}", e))?;

    let io_service = WorkspaceIOService::new();
    io_service.restore_into_workspace(target_workspace_id, &json)
}

/// 删除指定备份文件（从 Git 仓库移除并推送）
pub fn delete_backup(workspace_name: &str, file_name: &str) -> Result<(), String> {
    tracing::info!("删除备份: {}/{}", workspace_name, file_name);
    let (settings, password) = load_config_with_credentials()?;
    if !settings.is_configured() {
        return Err(repo_error!("尚未配置备份设备"));
    }
    let url = settings.repo_url.trim().to_string();
    let branch = settings.branch.trim().to_string();
    let username = settings.username.trim().to_string();

    let repo = prepare_cache_repo(&url, &branch, &username, &password)?;

    let rel_dir = format!("{}/{}", BACKUPS_SUBDIR, workspace_name);
    let rel_path = Path::new(&rel_dir).join(file_name);
    let abs_path = cache_dir().join(&rel_path);

    // 删除工作树中的文件
    std::fs::remove_file(&abs_path).map_err(|e| repo_error!("删除备份文件失败: {}", e))?;

    // 从 Git 索引移除并提交推送
    let message = format!("delete: {}/{}", workspace_name, file_name);
    commit_and_push(
        &repo, &branch, &username, &password, &rel_path, &message, true,
    )?;

    Ok(())
}
