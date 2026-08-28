//! Error types for the rfid-cover-custody crate.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RfidError {
    // ---- Tag / pairing ----
    #[error("EPC mismatch: tag EPC does not match box body serial binding")]
    EpcMismatch,

    #[error("lot mismatch: tag lot {tag_lot} does not match box lot {box_lot}")]
    LotMismatch { tag_lot: String, box_lot: String },

    #[error("tag {tag_id} is already paired to box {box_id}")]
    AlreadyPaired { tag_id: String, box_id: String },

    #[error("tag {0} is retired and cannot be used")]
    TagRetired(String),

    // ---- Reader / HMAC ----
    #[error("reader SVID missing or rejected: {0}")]
    ReaderSvidRejected(String),

    #[error("HMAC challenge verification failed for tag {tag_id}")]
    HmacFailure { tag_id: String },

    #[error("invalid HMAC key")]
    InvalidHmacKey,

    // ---- Custody officer signature enforcement ----
    #[error("custody transition to {to_state} requires a valid officer signature")]
    OfficerSignatureRequired { to_state: String },

    #[error("officer {officer_id} has no registered verification key")]
    OfficerUnregistered { officer_id: String },

    #[error("officer signature does not verify over this_hash")]
    OfficerSignatureInvalid,

    #[error("reader signature verification failed")]
    ReaderSignatureInvalid,

    // ---- State machine ----
    #[error("transition {from} -> {to} is not allowed from reader type {reader_type}")]
    TransitionNotAllowed {
        from: String,
        to: String,
        reader_type: String,
    },

    #[error("transition from terminal state {0} is not permitted (target must be QUARANTINE)")]
    TransitionFromTerminal(String),

    #[error("four-eyes requirement not met for transition {from} -> {to}")]
    FourEyesRequired { from: String, to: String },

    // ---- Hash chain ----
    #[error(
        "hash chain broken at transition {transition_id}: expected {expected}, computed {computed}"
    )]
    ChainBroken {
        transition_id: String,
        expected: String,
        computed: String,
    },

    #[error("hash chain is empty")]
    ChainEmpty,

    // ---- Replay defence ----
    #[error("replay detected: nonce already seen for tag {tag_id} within the 5-minute window")]
    ReplayDetected { tag_id: String },

    // ---- Serialization ----
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    // ---- Database ----
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    // ---- ECC dispatch ----
    #[error("ECC dispatch failed: {0}")]
    EccDispatch(String),

    // ---- Generic ----
    #[error("internal error: {0}")]
    Internal(String),
}
