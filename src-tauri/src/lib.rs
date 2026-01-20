// ShotAuto - YouTube Shorts Automation Desktop App

mod db;

use db::{Config, Database, DashboardStats};
use std::sync::Mutex;
use tauri::State;

/// Application state managed by Tauri
pub struct AppState {
    pub db: Mutex<Database>,
}

// ==================== Tauri Commands ====================

/// Get current configuration
#[tauri::command]
fn get_config(state: State<AppState>) -> Result<Config, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_config().map_err(|e| e.to_string())
}

/// Save configuration
#[tauri::command]
fn save_config(state: State<AppState>, config: Config) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_config(&config).map_err(|e| e.to_string())
}

/// Get dashboard statistics
#[tauri::command]
fn get_stats(state: State<AppState>) -> Result<DashboardStats, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_stats().map_err(|e| e.to_string())
}

/// Test YouTube API key
#[tauri::command]
async fn test_youtube_api(api_key: String) -> Result<bool, String> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/videos?part=id&id=dQw4w9WgXcQ&key={}",
        api_key
    );
    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    Ok(response.status().is_success())
}

/// Test Telegram bot token
#[tauri::command]
async fn test_telegram_bot(token: String) -> Result<bool, String> {
    let url = format!("https://api.telegram.org/bot{}/getMe", token);
    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    Ok(response.status().is_success())
}

/// Test Ollama endpoint
#[tauri::command]
async fn test_ollama(endpoint: String) -> Result<bool, String> {
    let url = format!("{}/api/tags", endpoint);
    let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    Ok(response.status().is_success())
}

// ==================== App Entry Point ====================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Get app data directory
    let app_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("shotauto");
    
    std::fs::create_dir_all(&app_dir).expect("Failed to create app data directory");
    
    let db_path = app_dir.join("shotauto.db");
    let db = Database::new(db_path).expect("Failed to initialize database");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { db: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_stats,
            test_youtube_api,
            test_telegram_bot,
            test_ollama,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
