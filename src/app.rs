#[derive(Debug, Clone)]
pub struct AppState {
    pub password_hash: String,
}

impl AppState {
    pub async fn init(password_hash: String) -> Self {
        AppState { password_hash }
    }
}
