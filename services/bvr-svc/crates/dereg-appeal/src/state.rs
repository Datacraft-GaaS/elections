#[derive(Clone)]
pub struct AppState {
    pub db_url: String,
}

impl AppState {
    pub fn new(db_url: impl Into<String>) -> Self {
        Self {
            db_url: db_url.into(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(
            std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/dereg_appeal".into()),
        )
    }
}
