use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppState {
    pub password_hash: String,
}

impl AppState {
    pub async fn init(password_hash: String) -> Self {
        AppState { password_hash }
    }
}

#[derive(Debug, Clone)]
pub struct AppData {
    pub data_dir: PathBuf,
}

impl AppData {
    pub async fn init(data_dir: PathBuf) -> Self {
        AppData { data_dir }
    }
}
