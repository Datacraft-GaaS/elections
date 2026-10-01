use crate::store::SharedBvrStore;

#[derive(Clone)]
pub struct AppState {
    pub iprs_base_url: String,
    /// The register.
    ///
    /// `AppState` was an IPRS URL and nothing else, so the voter, status, Merkle
    /// and SAR handlers had no register to read and built their answers. The
    /// tables have been in `services/bvr-svc/migrations/` the whole time.
    pub store: SharedBvrStore,
}

impl AppState {
    pub fn new(iprs_base_url: impl Into<String>, store: SharedBvrStore) -> Self {
        Self { iprs_base_url: iprs_base_url.into(), store }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(
            std::env::var("IPRS_URL").unwrap_or_else(|_| "http://localhost:8090".into()),
            crate::store::BvrStore::in_memory(),
        )
    }
}
