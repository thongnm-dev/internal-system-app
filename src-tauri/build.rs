use std::fs;
use std::path::Path;

fn main() {
    // Tạo config/config.ini từ config.ini.example nếu chưa có (production build).
    let config_dir = Path::new("config");
    let config_file = config_dir.join("config.ini");
    let template = Path::new("config.ini.example");

    if !config_file.exists() && template.exists() {
        fs::create_dir_all(config_dir).expect("Failed to create config/ directory");
        fs::copy(template, &config_file).expect("Failed to copy config.ini.example to config/config.ini");
    }

    tauri_build::build()
}
