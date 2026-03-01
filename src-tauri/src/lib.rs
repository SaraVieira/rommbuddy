#![allow(clippy::too_many_lines)]

mod commands;
mod db;
mod dedup;
pub mod entity;
mod error;
mod hash;
mod metadata;
mod models;
pub mod platform_registry;
mod retroachievements;
mod saves;
mod sources;

use directories::ProjectDirs;
use tauri::Manager;

/// Run the Tauri application.
///
/// # Panics
///
/// Panics if the Tauri application fails to start.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_log::Builder::new()
          .target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::Webview,
          ))
          .build());

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .setup(|app| {
            let db_path = if let Some(proj_dirs) =
                ProjectDirs::from("com", "romm-buddy", "romm-buddy")
            {
                let data_dir = proj_dirs.data_dir();
                std::fs::create_dir_all(data_dir)?;
                format!("sqlite:{}/romm-buddy.db", data_dir.display())
            } else {
                "sqlite:romm-buddy.db".to_string()
            };

            let db = tauri::async_runtime::block_on(db::create_pool(&db_path))?;
            app.manage(db);
            app.manage(commands::CancelTokenMap(
                tokio::sync::Mutex::new(std::collections::HashMap::new()),
            ));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_platforms,
            commands::get_sources,
            commands::test_romm_connection,
            commands::test_local_path,
            commands::add_source,
            commands::update_source,
            commands::get_source_credentials,
            commands::remove_source,
            commands::sync_source,
            commands::cancel_sync,
            commands::get_library_roms,
            commands::get_platforms_with_counts,
            commands::proxy_image,
            commands::get_retroarch_path,
            commands::set_retroarch_path,
            commands::detect_cores,
            commands::get_core_mappings,
            commands::has_core_mapping,
            commands::set_core_mapping,
            commands::download_and_launch,
            commands::get_available_cores,
            commands::install_core,
            commands::get_emulators,
            commands::get_emulator_paths,
            commands::set_emulator_path,
            commands::detect_emulators,
            commands::update_launchbox_db,
            commands::fetch_metadata,
            commands::cancel_metadata,
            commands::has_launchbox_db,
            commands::compute_rom_hash,
            commands::enrich_single_rom,
            commands::get_rom,
            commands::get_rom_screenshots,
            commands::get_ra_credentials,
            commands::set_ra_credentials,
            commands::test_ra_connection,
            commands::get_achievements,
            commands::toggle_favorite,
            commands::get_favorites_count,
            commands::get_rom_sources,
            commands::deduplicate_roms,
            commands::import_dat_file,
            commands::get_dat_files,
            commands::remove_dat_file,
            commands::detect_dat_platform,
            commands::verify_library,
            commands::cancel_verification,
            commands::get_verification_stats,
            commands::get_igdb_credentials,
            commands::set_igdb_credentials,
            commands::test_igdb_connection,
            commands::get_ss_credentials,
            commands::set_ss_credentials,
            commands::test_ss_connection,
            commands::get_rom_saves,
            commands::get_save_paths,
            commands::set_save_path,
            commands::delete_save_file,
            commands::export_save_file,
            commands::import_save_file,
            commands::read_file_base64,
            commands::get_all_registry_platforms,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
