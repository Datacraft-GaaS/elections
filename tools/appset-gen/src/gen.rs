use crate::model::{all_sites, ServiceManifest};
use anyhow::Result;
use std::path::Path;

/// Site metadata used when rendering templates
struct SiteMeta {
    name: String,
    cluster_url: String,
    namespace: String,
    is_edge: bool,
    is_dr: bool,
}

fn site_meta(site: &str) -> SiteMeta {
    let is_edge = site.starts_with("edge-");
    let is_dr = site == "primary" || site == "mirror";
    let cluster_url = match site {
        "primary" => "https://k8s.primary.eel.internal:6443",
        "mirror" => "https://k8s.mirror.eel.internal:6443",
        "edge-nairobi" => "https://k8s.edge-nairobi.eel.internal:6443",
        "edge-mombasa" => "https://k8s.edge-mombasa.eel.internal:6443",
        "edge-kisumu" => "https://k8s.edge-kisumu.eel.internal:6443",
        "edge-eldoret" => "https://k8s.edge-eldoret.eel.internal:6443",
        "edge-garissa" => "https://k8s.edge-garissa.eel.internal:6443",
        _ => "https://k8s.unknown.eel.internal:6443",
    };
    SiteMeta {
        name: site.to_string(),
        cluster_url: cluster_url.to_string(),
        namespace: site.replace('-', "_").replace("edge_", "edge-"),
        is_edge,
        is_dr,
    }
}

/// Emit one ApplicationSet YAML (per-service, fan-out over its sites).
pub fn emit_appset(svc: &ServiceManifest, out: &Path) -> Result<()> {
    let sites = svc.effective_sites();
    let generators: Vec<String> = sites
        .iter()
        .map(|s| {
            let meta = site_meta(s);
            format!(
                "    - list:\n        elements:\n          - site: {site}\n            clusterURL: {url}\n            namespace: {svc_name}\n            image: {image}\n",
                site = s,
                url = meta.cluster_url,
                svc_name = svc.name,
                image = svc.image,
            )
        })
        .collect();

    let yaml = format!(
        r#"apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: {name}
  namespace: argocd
  labels:
    eel.internal/service: "{name}"
spec:
  goTemplate: true
  goTemplateOptions: ["missingkey=error"]
  generators:
{generators}
  template:
    metadata:
      name: "{{{{.name}}}}-{{{{.site}}}}"
      namespace: argocd
      labels:
        eel.internal/service: "{name}"
        eel.internal/site: "{{{{.site}}}}"
      annotations:
        notifications.argoproj.io/subscribe.on-sync-failed.slack: eel-ops-alerts
        notifications.argoproj.io/subscribe.on-sync-failed.email: ops@eel.internal
    spec:
      project: {name}
      source:
        repoURL: https://gitea.internal/eel/eel
        targetRevision: HEAD
        path: "environments/{{{{.site}}}}/services/{name}"
      destination:
        server: "{{{{.clusterURL}}}}"
        namespace: "{{{{.namespace}}}}"
      syncPolicy:
        automated:
          prune: true
          selfHeal: true
        syncOptions:
          - CreateNamespace=true
          - ServerSideApply=true
      revisionHistoryLimit: 3
"#,
        name = svc.name,
        generators = generators.join(""),
    );

    let path = out.join(format!("{}.yaml", svc.name));
    std::fs::write(&path, yaml)?;
    Ok(())
}

/// Emit per-site Kustomize overlay skeletons for a service.
pub fn emit_kustomize_overlays(svc: &ServiceManifest, out: &Path) -> Result<()> {
    let sites = svc.effective_sites();
    for site in &sites {
        let meta = site_meta(site);
        let overlay_dir = out
            .parent()
            .unwrap_or(out)
            .join("overlays")
            .join(&svc.name)
            .join(site);
        std::fs::create_dir_all(&overlay_dir)?;

        let kustomization = format!(
            r#"apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
namespace: {ns}
resources:
  - ../../../../services/{name}/deploy/base
patches:
  - path: replica-patch.yaml
    target:
      kind: Deployment
      name: {name}
images:
  - name: {name}
    newName: {image}
    newTag: "latest"
labels:
  - pairs:
      eel.internal/site: {site}
      eel.internal/service: {name}
    includeSelectors: false
"#,
            ns = svc.name,
            name = svc.name,
            image = svc.image,
            site = site,
        );

        let replicas = svc
            .replicas
            .get(site)
            .copied()
            .unwrap_or(if meta.is_edge { 2 } else { 3 });

        let replica_patch = format!(
            r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {name}
spec:
  replicas: {replicas}
"#,
            name = svc.name,
            replicas = replicas,
        );

        std::fs::write(overlay_dir.join("kustomization.yaml"), kustomization)?;
        std::fs::write(overlay_dir.join("replica-patch.yaml"), replica_patch)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ServiceManifest;
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    fn sample_svc() -> ServiceManifest {
        ServiceManifest {
            name: "rts-svc".to_string(),
            image: "harbor.eel.internal/eel/rts-svc".to_string(),
            artifacts: vec![],
            sites: vec!["primary".to_string(), "mirror".to_string()],
            strategy: BTreeMap::new(),
            replicas: BTreeMap::new(),
        }
    }

    #[test]
    fn appset_snapshot() {
        let svc = sample_svc();
        let dir = tempdir().unwrap();
        emit_appset(&svc, dir.path()).unwrap();
        let content = std::fs::read_to_string(dir.path().join("rts-svc.yaml")).unwrap();
        insta::assert_snapshot!(content);
    }

    #[test]
    fn overlay_creates_files() {
        let svc = sample_svc();
        let dir = tempdir().unwrap();
        emit_kustomize_overlays(&svc, dir.path()).unwrap();
        // Overlay dirs should exist for both sites
        for site in &["primary", "mirror"] {
            let kpath = dir
                .path()
                .parent()
                .unwrap_or(dir.path())
                .join("overlays")
                .join("rts-svc")
                .join(site)
                .join("kustomization.yaml");
            // tempdir parent is OS tmp; just verify the file was written somewhere
            // under the overlay path relative to out
            let alt = dir
                .path()
                .join("../overlays/rts-svc")
                .join(site)
                .join("kustomization.yaml");
            assert!(
                kpath.exists() || alt.exists(),
                "kustomization.yaml missing for site {site}"
            );
        }
    }

    #[test]
    fn default_sites_all_seven() {
        let svc = ServiceManifest {
            name: "portal-svc".to_string(),
            image: "harbor.eel.internal/eel/portal-svc".to_string(),
            artifacts: vec![],
            sites: vec![],
            strategy: BTreeMap::new(),
            replicas: BTreeMap::new(),
        };
        assert_eq!(svc.effective_sites().len(), 7);
    }
}
