// ============================================================================
// Crate 级 Clippy 放行策略
//
// 以下 lint 在全 crate 范围放行，原因集中记录于此，便于后续定向重构清理：
// - type_complexity：SQLite 仓储读取、响应解析等存在深层嵌套的泛型返回类型，
//   抽取 type alias 需较大范围重构，暂整体放行（见工程化路线图阶段三）。
// - large_enum_variant：Postman 导入类型的 enum 变体大小差异较大，但导入为
//   低频场景，Box 化收益有限，暂放行。
// ============================================================================
#![allow(clippy::type_complexity)]
#![allow(clippy::large_enum_variant)]

mod application;
mod domain;
mod error_macro;
mod infrastructure;
mod interface;

// 使用显式导出避免 glob re-export 的命名冲突
// 只导出领域层的模型和仓储接口（遵循 DDD 原则）
pub use domain::models::*;
pub use domain::repositories::*;

// interface commands 显式导出（Tauri 命令注册需要）
pub use interface::commands::ai::*;
pub use interface::commands::app_config::*; // 合并 settings + workspace
pub use interface::commands::chat::*;
pub use interface::commands::collection::*;
pub use interface::commands::file_dialog::*;
pub use interface::commands::git_backup::*;
pub use interface::commands::history::*;
pub use interface::commands::http::*;
pub use interface::commands::import::*;
pub use interface::commands::md::*;
pub use interface::commands::orchestration::*;
pub use interface::commands::response::*;
pub use interface::commands::script::*;
pub use interface::commands::script_execution::*;
pub use interface::commands::sse::*;
pub use interface::commands::stress::*;
pub use interface::commands::workspace_data::*; // 合并 environment + memory + cookie
pub use interface::commands::workspace_io::*;
pub use interface::commands::ws::*; // 工作区导入导出

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    infrastructure::logging::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            tracing::info!("FM Tester 应用启动");

            let window = app.get_webview_window("main").expect("找不到主窗口");
            window.on_window_event(|event| {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    tracing::info!("应用关闭，正在清理资源...");
                    infrastructure::sqlite::connection::shutdown();
                }
            });

            // 注：定时任务恢复由前端 App.js 在 onMounted 中调用 restore_scheduled_tasks_cmd 完成。
            // 后端不在此处启动钩子里重复调用，避免与前端命令并发争抢 SQLite Mutex。

            // 启动自动备份后台任务（读取设置决定是否启用；循环首次 sleep 到下个备份时刻，不立即争抢 SQLite）
            infrastructure::auto_backup::AutoBackupScheduler::start(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 工作区
            get_workspaces,
            get_last_workspace,
            create_workspace,
            switch_workspace,
            delete_workspace,
            update_workspace,
            set_last_workspace,
            set_last_api,
            get_last_api,
            reorder_workspaces,
            // 集合
            get_collections,
            get_collections_tree,
            get_items_by_ids,
            create_collection,
            create_api,
            update_api,
            delete_collection_item,
            update_collection,
            update_collection_settings,
            move_api,
            move_collection,
            reorder_collection_items,
            duplicate_api,
            duplicate_collection,
            // 环境
            get_environments,
            save_environment,
            delete_environment,
            switch_environment,
            get_active_variables,
            reorder_environments,
            get_available_variables,
            replace_variables_text,
            // 记忆
            get_expanded_collections,
            save_expanded_collections,
            get_open_tabs,
            save_open_tabs,
            // HTTP
            send_http_request,
            export_as_curl,
            // Cookie
            get_cookies,
            clear_cookies,
            delete_cookie,
            add_cookie,
            // Saved Response
            save_response,
            get_saved_responses,
            get_saved_response,
            delete_saved_response,
            get_api_saved_responses,
            // History
            get_history_dates,
            get_history_by_date,
            get_history_entry,
            delete_history_entry,
            clear_history_by_date,
            clear_all_history,
            // Settings
            get_settings,
            update_settings,
            // AI
            get_ai_models,
            chat_ai,
            chat_ai_agent,
            optimize_script_ai,
            // Chat History
            save_chat_history,
            get_chat_history,
            clear_chat_history,
            get_chat_sessions,
            delete_chat_session,
            rename_chat_session,
            // Script
            save_script,
            get_script,
            delete_script,
            delete_target_scripts,
            get_all_scripts,
            execute_pre_scripts_cmd,
            execute_post_scripts_cmd,
            // API Doc
            get_api_doc,
            save_api_doc,
            generate_api_doc_with_ai,
            get_doc_generation_status,
            cancel_doc_generation,
            get_api_doc_metadata,
            // File Dialog
            safe_pick_directory,
            safe_save_file,
            // Import
            preview_openapi,
            import_openapi,
            parse_curl,
            preview_postman,
            import_postman,
            export_collection_postman,
            export_collection_postman_with_data,
            // Stress Test
            start_stress_test,
            get_stress_test_progress,
            stop_stress_test,
            get_api_stress_test_results,
            get_stress_test_result,
            delete_stress_test_result,
            get_stress_params,
            save_stress_params,
            // Orchestration
            get_orchestrations,
            get_orchestration,
            create_orchestration_cmd,
            update_orchestration_cmd,
            delete_orchestration_cmd,
            add_orchestration_step_cmd,
            update_orchestration_step_cmd,
            remove_orchestration_step_cmd,
            reorder_orchestration_steps_cmd,
            create_orchestration_run_cmd,
            update_orchestration_run_step_cmd,
            complete_orchestration_run_cmd,
            get_orchestration_runs,
            get_orchestration_run,
            delete_orchestration_run_cmd,
            clear_orchestration_runs_cmd,
            update_orchestration_schedule_cmd,
            restore_scheduled_tasks_cmd,
            get_scheduled_tasks_cmd,
            get_next_run_time_cmd,
            get_next_run_times_cmd,
            reorder_orchestrations_cmd,
            // SSE
            start_sse_cmd,
            stop_sse_cmd,
            // WebSocket
            connect_websocket,
            send_ws_message,
            disconnect_websocket,
            is_ws_connected,
            get_ws_configs,
            save_ws_config,
            delete_ws_config,
            // Domain Service 验证命令
            validate_collection_name,
            validate_environment_name,
            // 工作区导入导出
            export_workspace,
            preview_workspace_import,
            import_workspace,
            // Git 工作区备份
            get_git_backup_settings,
            update_git_backup_settings,
            update_auto_backup_settings,
            test_git_connection,
            backup_workspace,
            list_workspace_backups,
            restore_workspace_from_backup,
            restore_into_workspace,
            delete_backup
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
