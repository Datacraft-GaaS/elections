mod model;
mod gen;
mod kafka;
mod initial;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "eel-appset-gen", version, about = "Generate ArgoCD ApplicationSets and platform manifests from service descriptors")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Emit ApplicationSet + Kustomize overlay manifests for one service
    Service {
        /// Service name (matches services/<name>/deploy/service.yaml)
        #[arg(long)]
        service: Option<String>,
        /// Process every service with a deploy/service.yaml
        #[arg(long, conflicts_with = "service")]
        all: bool,
        /// Output directory (default: platform/argocd/applicationsets)
        #[arg(long, default_value = "platform/argocd/applicationsets")]
        out: PathBuf,
    },
    /// Emit Cilium NetworkPolicies and Kafka ACLs from platform/kafka/topics/*.yaml
    Topics {
        /// Output directory for generated manifests
        #[arg(long, default_value = "platform/cilium/generated")]
        out: PathBuf,
    },
    /// Exit non-zero if any service lacks deploy/service.yaml
    Check,
    /// Write template deploy/service.yaml for each of the 18 canonical services
    InitialServices,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Service { service, all, out } => {
            let names = collect_services(service, all)?;
            std::fs::create_dir_all(&out)?;
            for name in &names {
                let svc = model::load_service(name)?;
                gen::emit_appset(&svc, &out)?;
                gen::emit_kustomize_overlays(&svc, &out)?;
            }
            eprintln!("Generated manifests for {} service(s) in {}", names.len(), out.display());
        }
        Cmd::Topics { out } => {
            std::fs::create_dir_all(&out)?;
            kafka::emit_all(&out)?;
        }
        Cmd::Check => {
            let missing = check_missing()?;
            if missing.is_empty() {
                eprintln!("All services have deploy/service.yaml");
            } else {
                eprintln!("Missing deploy/service.yaml for: {}", missing.join(", "));
                std::process::exit(1);
            }
        }
        Cmd::InitialServices => {
            initial::write_templates()?;
        }
    }
    Ok(())
}

fn collect_services(service: Option<String>, all: bool) -> Result<Vec<String>> {
    if let Some(name) = service {
        return Ok(vec![name]);
    }
    if all {
        return model::discover_services();
    }
    anyhow::bail!("Provide --service <name> or --all")
}

fn check_missing() -> Result<Vec<String>> {
    let all = model::all_canonical_services();
    let mut missing = Vec::new();
    for name in &all {
        let path = format!("services/{}/deploy/service.yaml", name);
        if !std::path::Path::new(&path).exists() {
            missing.push(name.clone());
        }
    }
    Ok(missing)
}
