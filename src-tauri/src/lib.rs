mod commands;
mod error;
mod models;
mod services;
mod state;
mod utils;

use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let st_guard = handle.state::<AppState>();
                let st = st_guard.inner();
                if let Err(e) = services::refresh_school_cache(st).await {
                    eprintln!("刷新学校缓存失败: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::status,
            commands::quit_app,
            commands::open_external,
            commands::upload_run_face_info,
            commands::get_run_face_status,
            commands::run_face_info_comparison,
            commands::login,
            commands::save_session,
            commands::load_session_info,
            commands::has_cli_session,
            commands::clear_cli_session,
            commands::session_accounts,
            commands::session_get,
            commands::session_remove,
            commands::get_home_run_info,
            commands::run,
            commands::finish_run,
            commands::discard_run,
            commands::simulate,
            commands::get_task,
            commands::get_run_record_detail,
            commands::get_sport_info,
            commands::have_yd_info,
            commands::get_schools,
            commands::get_config,
            commands::save_config,
            commands::face_add,
            commands::face_list,
            commands::face_get,
            commands::face_delete,
            commands::face_next,
            commands::track_status,
            commands::track_query,
            commands::track_list_local,
            commands::track_refine,
            commands::track_save_local,
            commands::record_track,
            commands::validate,
            commands::upload,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
