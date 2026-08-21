use crate::models::AuditManifest;

pub struct ManifestLoader;

impl ManifestLoader {
    pub fn load(_path: &str) -> AuditManifest {
        AuditManifest::default()
    }
}
