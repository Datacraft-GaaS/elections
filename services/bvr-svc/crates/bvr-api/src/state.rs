#[derive(Clone)]
pub struct AppState {
    pub iprs_base_url: String,
}

impl AppState {
    pub fn new(iprs_base_url: impl Into<String>) -> Self {
        Self {
            iprs_base_url: iprs_base_url.into(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(std::env::var("IPRS_URL").unwrap_or_else(|_| "http://localhost:8090".into()))
    }
}
