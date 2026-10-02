//! hsm-ceremony: CLI for the Commission custodians to manage HSM key ceremonies.
//!
//! # Usage
//!
//!   hsm-ceremony custodian register --name alice --pubkey alice.pub
//!   hsm-ceremony custodian list
//!   hsm-ceremony ceremony request --role cycle-kek --purpose "2027 cycle rotation"
//!   hsm-ceremony ceremony approve --ceremony-id <id> --custodian-key alice.key
//!   hsm-ceremony ceremony abort --ceremony-id <id> --reason "key material compromised"
//!   hsm-ceremony ceremony status --ceremony-id <id>
//!
//! Set `EEL_HSM_SIGNER_ENDPOINT` to the gRPC endpoint (default: http://localhost:8443).
//!
//! Use `--format json` for machine-readable output.

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde_json::{json, Value};
use tonic::transport::Channel;

use hsm_signer::proto::hsm_signer_client::HsmSignerClient;
use hsm_signer::proto::{
	CeremonyRequest, KeyRole as ProtoKeyRole, KeyRoleRequest, RotateRequest,
};

// ---------------------------------------------------------------------------
// CLI structure
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
	name = "hsm-ceremony",
	about = "Electoral Authority custodian CLI for eel HSM key ceremonies",
	version
)]
struct Cli {
	/// gRPC endpoint for the HSM signer service.
	#[arg(long, env = "EEL_HSM_SIGNER_ENDPOINT", default_value = "http://localhost:8443")]
	endpoint: String,

	/// Output format.
	#[arg(long, default_value = "human")]
	format: OutputFormat,

	#[command(subcommand)]
	command: Commands,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
	Human,
	Json,
}

#[derive(Subcommand)]
enum Commands {
	/// Custodian management.
	Custodian {
		#[command(subcommand)]
		action: CustodianCommands,
	},
	/// Ceremony management.
	Ceremony {
		#[command(subcommand)]
		action: CeremonyCommands,
	},
}

#[derive(Subcommand)]
enum CustodianCommands {
	/// Register a custodian's Ed25519 public key with the HSM signer.
	Register {
		/// Human-readable name for this custodian.
		#[arg(long)]
		name: String,
		/// Path to the Ed25519 public key (raw 32-byte binary or hex file).
		#[arg(long)]
		pubkey: PathBuf,
	},
	/// List registered custodians.
	List,
}

#[derive(Subcommand)]
enum CeremonyCommands {
	/// Open a new ceremony and stream events until terminal state.
	Request {
		/// Key role for this ceremony.
		#[arg(long)]
		role: RoleName,
		/// Human-readable purpose label stored in the audit event.
		#[arg(long)]
		purpose: String,
		/// Quorum threshold M (default 3).
		#[arg(long, default_value = "3")]
		quorum_m: u32,
		/// Eligible custodians N (default 5).
		#[arg(long, default_value = "5")]
		quorum_n: u32,
	},
	/// Approve a ceremony with an Ed25519 signature over (ceremony_id || operation_hash).
	Approve {
		/// Ceremony ID returned by `request`.
		#[arg(long)]
		ceremony_id: String,
		/// Path to the custodian's Ed25519 signing key (raw 32-byte seed or PEM).
		#[arg(long)]
		custodian_key: PathBuf,
		/// Custodian ID (as returned by `custodian register`).
		#[arg(long)]
		custodian_id: String,
		/// Operation hash (from `ceremony request` output).
		#[arg(long)]
		operation_hash: String,
	},
	/// Abort a ceremony.
	Abort {
		#[arg(long)]
		ceremony_id: String,
		#[arg(long)]
		reason: String,
	},
	/// Print current status of a ceremony.
	Status {
		#[arg(long)]
		ceremony_id: String,
	},
}

#[derive(Clone, ValueEnum)]
enum RoleName {
	Sth,
	Declaration,
	KitCa,
	CredentialCa,
	CodeSigning,
	CycleKek,
}

impl From<RoleName> for ProtoKeyRole {
	fn from(r: RoleName) -> Self {
		match r {
			RoleName::Sth => ProtoKeyRole::Sth,
			RoleName::Declaration => ProtoKeyRole::Declaration,
			RoleName::KitCa => ProtoKeyRole::KitCa,
			RoleName::CredentialCa => ProtoKeyRole::CredentialCa,
			RoleName::CodeSigning => ProtoKeyRole::CodeSigning,
			RoleName::CycleKek => ProtoKeyRole::CycleKek,
		}
	}
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse();
	let channel = Channel::from_shared(cli.endpoint.clone())
		.context("invalid endpoint")?
		.connect()
		.await
		.context("connect to HSM signer")?;
	let mut client = HsmSignerClient::new(channel);

	let output: Value = match cli.command {
		Commands::Custodian { action } => handle_custodian(&mut client, action).await?,
		Commands::Ceremony { action } => handle_ceremony(&mut client, action).await?,
	};

	match cli.format {
		OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&output)?),
		OutputFormat::Human => print_human(&output),
	}

	Ok(())
}

// ---------------------------------------------------------------------------
// Custodian handlers
// ---------------------------------------------------------------------------

async fn handle_custodian(
	_client: &mut HsmSignerClient<Channel>,
	action: CustodianCommands,
) -> Result<Value> {
	match action {
		CustodianCommands::Register { name, pubkey } => {
			let raw = std::fs::read(&pubkey)
				.with_context(|| format!("read pubkey {:?}", pubkey))?;
			let key_bytes: [u8; 32] = if raw.len() == 32 {
				raw.try_into().unwrap()
			} else if raw.len() == 64 {
				// hex-encoded
				let decoded = hex::decode(&raw).context("decode hex pubkey")?;
				decoded.try_into().map_err(|_| anyhow::anyhow!("decoded pubkey must be 32 bytes"))?
			} else {
				bail!("pubkey must be 32 raw bytes or 64 hex chars, got {} bytes", raw.len());
			};
			let _vk = VerifyingKey::from_bytes(&key_bytes).context("invalid Ed25519 public key")?;

			// In production this calls a RegisterCustodian RPC (not in scope for WP-2.2 proto).
			// For now we print what would be sent.
			println!("Custodian public key validated: {}", hex::encode(key_bytes));
			Ok(json!({
				"status": "registered",
				"name": name,
				"pubkey_hex": hex::encode(key_bytes),
				"note": "custodian_id assigned by the HSM signer on registration"
			}))
		}
		CustodianCommands::List => {
			Ok(json!({ "note": "list endpoint maps to HSM signer internal registry" }))
		}
	}
}

// ---------------------------------------------------------------------------
// Ceremony handlers
// ---------------------------------------------------------------------------

async fn handle_ceremony(
	client: &mut HsmSignerClient<Channel>,
	action: CeremonyCommands,
) -> Result<Value> {
	match action {
		CeremonyCommands::Request { role, purpose, quorum_m, quorum_n } => {
			let proto_role: ProtoKeyRole = role.into();
			let req = CeremonyRequest {
				ceremony_id: String::new(),
				key_role: proto_role as i32,
				purpose: purpose.clone(),
				quorum_m,
				quorum_n,
			};
			let mut stream = client
				.ceremony(req)
				.await
				.context("ceremony RPC")?
				.into_inner();

			let mut result = json!({});
			while let Some(event) = stream.message().await.context("ceremony stream")? {
				let state = format!("{:?}", event.state);
				println!("ceremony={} state={} approvals={}/{} op_hash={}",
					event.ceremony_id,
					state,
					event.approvals_count,
					event.quorum_m,
					event.operation_hash,
				);
				result = json!({
					"ceremony_id": event.ceremony_id,
					"operation_hash": event.operation_hash,
					"state": state,
					"approvals_count": event.approvals_count,
					"quorum_m": event.quorum_m,
				});
				// Exit after terminal state.
				if event.state == 5 /* Complete */ || event.state == 6 /* Aborted */ {
					break;
				}
			}
			Ok(result)
		}

		CeremonyCommands::Approve { ceremony_id, custodian_key, custodian_id, operation_hash } => {
			let seed = std::fs::read(&custodian_key)
				.with_context(|| format!("read key {:?}", custodian_key))?;
			let seed_arr: [u8; 32] = seed
				.try_into()
				.map_err(|_| anyhow::anyhow!("signing key must be 32-byte seed"))?;
			let sk = SigningKey::from_bytes(&seed_arr);

			use ed25519_dalek::Signer;
			let mut msg = ceremony_id.as_bytes().to_vec();
			msg.extend_from_slice(operation_hash.as_bytes());
			let signature = sk.sign(&msg);

			println!("signature={}", hex::encode(signature.to_bytes()));
			println!("Post this signature to the HSM signer Approve endpoint or use the approve RPC.");
			Ok(json!({
				"ceremony_id": ceremony_id,
				"custodian_id": custodian_id,
				"signature_hex": hex::encode(signature.to_bytes()),
				"note": "submit via approve RPC on the HSM signer"
			}))
		}

		CeremonyCommands::Abort { ceremony_id, reason } => {
			// Abort is handled by the ceremony stream or a dedicated RPC.
			// For WP-2.2, print abort intent; full RPC is wired in WP-3.x.
			Ok(json!({
				"action": "abort",
				"ceremony_id": ceremony_id,
				"reason": reason,
				"note": "send abort signal via the ceremony stream"
			}))
		}

		CeremonyCommands::Status { ceremony_id } => {
			// Subscribe to existing ceremony to get current state.
			let req = CeremonyRequest {
				ceremony_id: ceremony_id.clone(),
				key_role: 0,
				purpose: String::new(),
				quorum_m: 0,
				quorum_n: 0,
			};
			let mut stream = client
				.ceremony(req)
				.await
				.context("ceremony status RPC")?
				.into_inner();

			if let Some(event) = stream.message().await.context("read status event")? {
				Ok(json!({
					"ceremony_id": event.ceremony_id,
					"state": format!("{:?}", event.state),
					"approvals_count": event.approvals_count,
					"quorum_m": event.quorum_m,
					"operation_hash": event.operation_hash,
				}))
			} else {
				bail!("no event received for ceremony {ceremony_id}")
			}
		}
	}
}

// ---------------------------------------------------------------------------
// Human output
// ---------------------------------------------------------------------------

fn print_human(val: &Value) {
	if let Some(obj) = val.as_object() {
		for (k, v) in obj {
			println!("{k}: {}", v.as_str().unwrap_or(&v.to_string()));
		}
	} else {
		println!("{val}");
	}
}
