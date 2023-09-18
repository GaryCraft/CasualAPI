#[tauri::command]
async fn greet(name: &str) -> Result<String, String> {
    println!("Hello from Rust!");
    Ok(format!("Hello from Rust! {}", name))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
