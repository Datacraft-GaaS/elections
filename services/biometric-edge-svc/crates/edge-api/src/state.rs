/// Shared application state injected into axum handlers via Extension.
#[derive(Clone, Debug)]
pub struct AppState {
    /// Base URL of bvr-svc for enrolment proxying.
    pub bvr_svc_url: String,
    /// Base URL of bvi-svc for config bundle download.
    pub bvi_svc_url: String,
    /// Base URL of ECC for distress alert routing.
    pub ecc_svc_url: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            bvr_svc_url: std::env::var("BVR_SVC_URL")
                .unwrap_or_else(|_| "http://bvr-svc.eel.local:8080".into()),
            bvi_svc_url: std::env::var("BVI_SVC_URL")
                .unwrap_or_else(|_| "http://bvi-svc.eel.local:8081".into()),
            ecc_svc_url: std::env::var("ECC_SVC_URL")
                .unwrap_or_else(|_| "http://ecc-svc.eel.local:8082".into()),
        }
    }
}
