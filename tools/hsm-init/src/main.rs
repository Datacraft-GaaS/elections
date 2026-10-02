// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! hsm-init: Bootstrap PKCS#11 keys in SoftHSM slots for eel local dev.
//!
//! Subcommands:
//!   init-slots      Generate a signing key in every known slot
//!   generate-key    Generate a key in a specific slot (--role)
//!   list-keys       List all objects in all known slots
//!   sign-blob       Sign a file with a slot key (--role, --input, --output)
//!   verify-blob     Verify a signature (--role, --input, --sig)

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use cryptoki::{
    context::{CInitializeArgs, Pkcs11},
    mechanism::Mechanism,
    object::{Attribute, AttributeType, KeyType, ObjectClass},
    session::UserType,
    types::AuthPin,
};
use std::{path::PathBuf, str::FromStr};

// ---------------------------------------------------------------------------
// Slot definitions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Sth,
    Declaration,
    KitCa,
    CredentialCa,
    CodeSigning,
    CycleKek,
}

impl FromStr for Role {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "sth" => Ok(Self::Sth),
            "declaration" => Ok(Self::Declaration),
            "kit-ca" => Ok(Self::KitCa),
            "credential-ca" => Ok(Self::CredentialCa),
            "code-signing" => Ok(Self::CodeSigning),
            "cycle-kek" => Ok(Self::CycleKek),
            other => Err(anyhow!("unknown role: {other}")),
        }
    }
}

struct SlotInfo {
    label: &'static str,
    user_pin: &'static str,
    key_label: &'static str,
}

fn slot_for_role(role: Role) -> SlotInfo {
    match role {
        Role::Sth => SlotInfo {
            label: "eel-sth",
            user_pin: "eel-sth-user-pin",
            key_label: "eel-sth-signing",
        },
        Role::Declaration => SlotInfo {
            label: "eel-declaration",
            user_pin: "eel-decl-user-pin",
            key_label: "eel-declaration-signing",
        },
        Role::KitCa => SlotInfo {
            label: "eel-kit-ca",
            user_pin: "eel-kitca-user-pin",
            key_label: "eel-kit-ca-signing",
        },
        Role::CredentialCa => SlotInfo {
            label: "eel-credential-ca",
            user_pin: "eel-credca-user-pin",
            key_label: "eel-credential-ca-signing",
        },
        Role::CodeSigning => SlotInfo {
            label: "eel-code-signing",
            user_pin: "eel-codesign-user-pin",
            key_label: "eel-code-signing-key",
        },
        Role::CycleKek => SlotInfo {
            label: "eel-cycle-kek",
            user_pin: "eel-kek-user-pin",
            key_label: "eel-cycle-kek",
        },
    }
}

const ALL_ROLES: &[Role] = &[
    Role::Sth,
    Role::Declaration,
    Role::KitCa,
    Role::CredentialCa,
    Role::CodeSigning,
    Role::CycleKek,
];

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "hsm-init", about = "Bootstrap PKCS#11 keys for eel local dev")]
struct Cli {
    /// Path to PKCS#11 module (libsofthsm2.so). Defaults to PKCS11_MODULE_PATH env var.
    #[arg(long, env = "PKCS11_MODULE_PATH")]
    module: PathBuf,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate signing keys in all known slots
    InitSlots,
    /// Generate a key in a specific slot
    GenerateKey {
        #[arg(long)]
        role: Role,
    },
    /// List all objects in all known slots
    ListKeys,
    /// Sign a blob with the slot key for --role
    SignBlob {
        #[arg(long)]
        role: Role,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Verify a blob signature with the slot key for --role
    VerifyBlob {
        #[arg(long)]
        role: Role,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        sig: PathBuf,
    },
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    let cli = Cli::parse();
    let pkcs11 = Pkcs11::new(&cli.module).context("failed to load PKCS#11 module")?;
    pkcs11
        .initialize(CInitializeArgs::OsThreads)
        .context("failed to initialize PKCS#11")?;

    match cli.cmd {
        Cmd::InitSlots => {
            for &role in ALL_ROLES {
                generate_key(&pkcs11, role)?;
            }
            println!("All slots initialized.");
        }
        Cmd::GenerateKey { role } => {
            generate_key(&pkcs11, role)?;
        }
        Cmd::ListKeys => {
            list_keys(&pkcs11)?;
        }
        Cmd::SignBlob {
            role,
            input,
            output,
        } => {
            let data = std::fs::read(&input).context("reading input file")?;
            let sig = sign_blob(&pkcs11, role, &data)?;
            std::fs::write(&output, &sig).context("writing signature file")?;
            println!("Signature written to {}", output.display());
        }
        Cmd::VerifyBlob { role, input, sig } => {
            let data = std::fs::read(&input).context("reading input file")?;
            let signature = std::fs::read(&sig).context("reading signature file")?;
            verify_blob(&pkcs11, role, &data, &signature)?;
            println!("Signature valid.");
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// PKCS#11 helpers
// ---------------------------------------------------------------------------

fn find_slot(pkcs11: &Pkcs11, label: &str) -> Result<cryptoki::slot::Slot> {
    let slots = pkcs11.get_slots_with_initialized_token()?;
    for slot in slots {
        let info = pkcs11.get_token_info(slot)?;
        if info.label().trim() == label {
            return Ok(slot);
        }
    }
    Err(anyhow!(
        "slot with label '{}' not found; run tools/softhsm/init.sh first",
        label
    ))
}

fn generate_key(pkcs11: &Pkcs11, role: Role) -> Result<()> {
    let info = slot_for_role(role);
    let slot = find_slot(pkcs11, info.label)?;
    let session = pkcs11.open_rw_session(slot)?;
    session.login(UserType::User, Some(&AuthPin::new(info.user_pin.into())))?;

    // Check if key already exists
    let existing = session.find_objects(&[
        Attribute::Label(info.key_label.as_bytes().to_vec()),
        Attribute::Class(ObjectClass::PRIVATE_KEY),
    ])?;
    if !existing.is_empty() {
        println!(
            "  Key '{}' already exists in slot '{}'.",
            info.key_label, info.label
        );
        return Ok(());
    }

    // Generate EC (Ed25519) key pair for signing roles; AES-256 for KEK
    match role {
        Role::CycleKek => {
            // AES-256 symmetric key for envelope encryption
            let template = vec![
                Attribute::Token(true),
                Attribute::Private(true),
                Attribute::Sensitive(true),
                Attribute::Extractable(false),
                Attribute::Label(info.key_label.as_bytes().to_vec()),
                Attribute::Class(ObjectClass::SECRET_KEY),
                Attribute::KeyType(KeyType::AES),
                Attribute::ValueLen(32.into()),
                Attribute::Encrypt(true),
                Attribute::Decrypt(true),
                Attribute::Wrap(true),
                Attribute::Unwrap(true),
            ];
            session.generate_key(&Mechanism::AesKeyGen, &template)?;
            println!(
                "  Generated AES-256 KEK '{}' in slot '{}'.",
                info.key_label, info.label
            );
        }
        _ => {
            // EC P-256 key pair (Ed25519 not universally supported in SoftHSM PKCS#11)
            let ec_params = vec![0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07];
            let pub_template = vec![
                Attribute::Token(true),
                Attribute::Label(info.key_label.as_bytes().to_vec()),
                Attribute::Class(ObjectClass::PUBLIC_KEY),
                Attribute::KeyType(KeyType::EC),
                Attribute::Verify(true),
                Attribute::EcParams(ec_params),
            ];
            let priv_template = vec![
                Attribute::Token(true),
                Attribute::Private(true),
                Attribute::Sensitive(true),
                Attribute::Extractable(false),
                Attribute::Label(info.key_label.as_bytes().to_vec()),
                Attribute::Class(ObjectClass::PRIVATE_KEY),
                Attribute::KeyType(KeyType::EC),
                Attribute::Sign(true),
            ];
            session.generate_key_pair(&Mechanism::EccKeyPairGen, &pub_template, &priv_template)?;
            println!(
                "  Generated EC P-256 key pair '{}' in slot '{}'.",
                info.key_label, info.label
            );
        }
    }

    Ok(())
}

fn list_keys(pkcs11: &Pkcs11) -> Result<()> {
    for &role in ALL_ROLES {
        let info = slot_for_role(role);
        match find_slot(pkcs11, info.label) {
            Ok(slot) => {
                let session = pkcs11.open_ro_session(slot)?;
                if session
                    .login(UserType::User, Some(&AuthPin::new(info.user_pin.into())))
                    .is_ok()
                {
                    let objects = session.find_objects(&[])?;
                    println!("Slot '{}': {} object(s)", info.label, objects.len());
                    for obj in objects {
                        if let Ok(attrs) = session
                            .get_attributes(obj, &[AttributeType::Label, AttributeType::Class])
                        {
                            let label = attrs
                                .iter()
                                .find_map(|a| {
                                    if let Attribute::Label(v) = a {
                                        Some(String::from_utf8_lossy(v).into_owned())
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or_default();
                            println!("  {label}");
                        }
                    }
                }
            }
            Err(e) => println!("Slot '{}': not found ({})", info.label, e),
        }
    }
    Ok(())
}

fn sign_blob(pkcs11: &Pkcs11, role: Role, data: &[u8]) -> Result<Vec<u8>> {
    let info = slot_for_role(role);
    let slot = find_slot(pkcs11, info.label)?;
    let session = pkcs11.open_rw_session(slot)?;
    session.login(UserType::User, Some(&AuthPin::new(info.user_pin.into())))?;

    let keys = session.find_objects(&[
        Attribute::Label(info.key_label.as_bytes().to_vec()),
        Attribute::Class(ObjectClass::PRIVATE_KEY),
    ])?;
    let key = keys.into_iter().next().ok_or_else(|| {
        anyhow!(
            "private key '{}' not found; run init-slots first",
            info.key_label
        )
    })?;
    let sig = session.sign(&Mechanism::Ecdsa, key, data)?;
    Ok(sig)
}

fn verify_blob(pkcs11: &Pkcs11, role: Role, data: &[u8], signature: &[u8]) -> Result<()> {
    let info = slot_for_role(role);
    let slot = find_slot(pkcs11, info.label)?;
    let session = pkcs11.open_ro_session(slot)?;
    session.login(UserType::User, Some(&AuthPin::new(info.user_pin.into())))?;

    let keys = session.find_objects(&[
        Attribute::Label(info.key_label.as_bytes().to_vec()),
        Attribute::Class(ObjectClass::PUBLIC_KEY),
    ])?;
    let key = keys
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("public key '{}' not found", info.key_label))?;
    session
        .verify(&Mechanism::Ecdsa, key, data, signature)
        .map_err(|e| anyhow!("verification failed: {e}"))?;
    Ok(())
}
