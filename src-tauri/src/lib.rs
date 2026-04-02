mod diagnostics;

use diagnostics::result::DiagnosticReport;

#[tauri::command]
async fn run_diagnosis(target: String) -> Result<DiagnosticReport, String> {
    Ok(diagnostics::run_diagnostics(&target).await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![run_diagnosis])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
