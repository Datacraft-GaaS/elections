//! PII classification wrappers for the eel platform.
//!
//! Implements the five-tier data-class model from ADR-0016.  Every field
//! that holds personal data must be wrapped in the appropriate type.
//! Accessing Tier-A/B values requires an explicit [`Purpose`] and a
//! [`PIIAuditSink`], which records the access in the audit trail.
//!
//! | Type | ADR tier | Encryption | Serde |
//! |---|---|---|---|
//! | [`PIIRestricted<T>`] | A | Column-level required | errors |
//! | [`PIISensitive<T>`]  | B | TDE at DB level | errors |
//! | [`PIIPublic<T>`]     | C | Integrity only | transparent |
//! | [`PIIOperational<T>`]| D | At rest | transparent |
//! | [`PIIEphemeral<T>`]  | E | TTL-managed | transparent |
//!
//! # Example
//!
//! ```
//! use eel_pii_rs::{PIIRestricted, Purpose, NoopAuditSink};
//! let id = PIIRestricted::new("KE12345678".to_string());
//! let sink = NoopAuditSink;
//! let value = id.reveal(Purpose::Identification, &sink);
//! assert_eq!(value, "KE12345678");
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// Purpose
// ---------------------------------------------------------------------------

/// The stated processing purpose, required when revealing restricted data.
///
/// Every call to `.reveal()` on a Tier-A or Tier-B wrapper must supply one
/// of these variants.  The purpose is forwarded to the [`PIIAuditSink`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Purpose {
    /// Enrolling a new voter or updating existing registration.
    Enrolment,
    /// Identifying a voter at a polling station.
    Identification,
    /// Adjudicating a registration dispute or appeal.
    Adjudication,
    /// Publishing statutory results or the voter register in gazette form.
    Publication,
    /// Producing a statutory report required by law.
    StatutoryReport,
    /// Receiving or processing a grievance or petition.
    GrievanceIntake,
    /// Support ticket handling by authorised staff.
    SupportTicket,
    /// Internal or external audit of system activity.
    Audit,
    /// Fulfilling a data-portability request under DPA s.26.
    DataPortability,
    /// Processing an erasure request under DPA s.40.
    ErasureRequest,
    /// Security investigation by authorised security personnel.
    SecurityInvestigation,
    /// Internal system monitoring (e.g. health checks, reconciliation).
    InternalMonitoring,
}

// ---------------------------------------------------------------------------
// PIIDataClass
// ---------------------------------------------------------------------------

/// Data classification tier per ADR-0016.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PIIDataClass {
    /// Tier A: Voter PII (maximum restriction).
    A,
    /// Tier B: Voter operational data.
    B,
    /// Tier C: Publicly available data.
    C,
    /// Tier D: Operational metadata.
    D,
    /// Tier E: Ephemeral data.
    E,
}

// ---------------------------------------------------------------------------
// Classified trait
// ---------------------------------------------------------------------------

/// Marker trait for schema-annotation types: each implementing type declares
/// its data classification statically.
///
/// # Example
///
/// ```
/// use eel_pii_rs::{Classified, PIIDataClass, PIIRestricted};
/// assert_eq!(<PIIRestricted<String>>::data_class(), PIIDataClass::A);
/// ```
pub trait Classified {
    /// Returns the ADR-0016 data class of this type.
    fn data_class() -> PIIDataClass;
}

// ---------------------------------------------------------------------------
// PIIAuditSink
// ---------------------------------------------------------------------------

/// Receiver for PII access events.
///
/// Implement this trait to wire reveal calls into your audit pipeline.  The
/// provided [`NoopAuditSink`] discards all events (suitable for tests).
pub trait PIIAuditSink: Send + Sync {
    /// Record that `type_name` data of the given `data_class` was accessed
    /// under `purpose`.
    fn record_access(&self, data_class: PIIDataClass, purpose: Purpose, type_name: &'static str);
}

/// A no-op audit sink that discards all events.
///
/// Use in unit tests or when audit infrastructure is not yet initialised.
///
/// # Example
///
/// ```
/// use eel_pii_rs::{NoopAuditSink, PIIAuditSink, PIIDataClass, Purpose};
/// let s = NoopAuditSink;
/// s.record_access(PIIDataClass::A, Purpose::Audit, "String");
/// ```
pub struct NoopAuditSink;

impl PIIAuditSink for NoopAuditSink {
    fn record_access(&self, _: PIIDataClass, _: Purpose, _: &'static str) {}
}

/// A tracing-backed audit sink that emits `tracing::info!` events.
///
/// # Example
///
/// ```
/// use eel_pii_rs::TracingAuditSink;
/// let _ = TracingAuditSink;
/// ```
pub struct TracingAuditSink;

impl PIIAuditSink for TracingAuditSink {
    fn record_access(&self, data_class: PIIDataClass, purpose: Purpose, type_name: &'static str) {
        tracing::info!(
            data_class = ?data_class,
            purpose = ?purpose,
            type_name,
            "PII access"
        );
    }
}

// ---------------------------------------------------------------------------
// SerdeRedact
// ---------------------------------------------------------------------------

/// Serialises any value as the literal string `"***REDACTED***"`.
///
/// Use this when you must include a PII field in a serialisable struct but
/// must not expose its actual value in logs or API responses.
///
/// # Example
///
/// ```
/// use eel_pii_rs::SerdeRedact;
/// let json = serde_json::to_string(&SerdeRedact).unwrap();
/// assert_eq!(json, r#""***REDACTED***""#);
/// ```
#[derive(Clone, Copy)]
pub struct SerdeRedact;

impl Serialize for SerdeRedact {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str("***REDACTED***")
    }
}

impl fmt::Debug for SerdeRedact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***REDACTED***")
    }
}

// ---------------------------------------------------------------------------
// Tier A: PIIRestricted
// ---------------------------------------------------------------------------

/// Tier-A PII wrapper (Voter PII, maximum restriction).
///
/// The inner value is inaccessible without an explicit [`Purpose`] and a
/// [`PIIAuditSink`].  The `Debug` and `Serialize` impls never expose the
/// value.
///
/// # Example
///
/// ```
/// use eel_pii_rs::{PIIRestricted, Purpose, NoopAuditSink};
/// let id = PIIRestricted::new("KE12345678".to_string());
/// assert!(!format!("{id:?}").contains("KE12345678"), "Debug must redact");
/// let value = id.reveal(Purpose::Identification, &NoopAuditSink);
/// assert_eq!(value, "KE12345678");
/// ```
pub struct PIIRestricted<T>(T);

impl<T> PIIRestricted<T> {
    /// Wrap `value` as Tier-A restricted PII.
    pub fn new(value: T) -> Self {
        PIIRestricted(value)
    }

    /// Reveal the inner value, recording the access in `sink`.
    pub fn reveal<S: PIIAuditSink>(&self, purpose: Purpose, sink: &S) -> &T {
        sink.record_access(PIIDataClass::A, purpose, std::any::type_name::<T>());
        &self.0
    }

    /// Consume `self`, returning the inner value and recording the access.
    pub fn into_inner<S: PIIAuditSink>(self, purpose: Purpose, sink: &S) -> T {
        sink.record_access(PIIDataClass::A, purpose, std::any::type_name::<T>());
        self.0
    }

    /// Return a [`SerdeRedact`] placeholder suitable for logging pipelines.
    pub fn as_redacted(&self) -> SerdeRedact {
        SerdeRedact
    }
}

impl<T> Classified for PIIRestricted<T> {
    fn data_class() -> PIIDataClass {
        PIIDataClass::A
    }
}

impl<T> fmt::Debug for PIIRestricted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PIIRestricted(***REDACTED***)")
    }
}

impl<T> fmt::Display for PIIRestricted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***REDACTED***")
    }
}

/// Serialising Tier-A data is refused at runtime to prevent accidental
/// exposure.  Use `.as_redacted()` or `.reveal()` explicitly.
impl<T: Serialize> Serialize for PIIRestricted<T> {
    fn serialize<S: serde::Serializer>(&self, _s: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(
            "PIIRestricted (Tier A) cannot be serialised directly; \
             use .as_redacted() or .reveal(purpose, sink)",
        ))
    }
}

/// Deserialising into `PIIRestricted<T>` is allowed (e.g. reading from DB).
impl<'de, T: Deserialize<'de>> Deserialize<'de> for PIIRestricted<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        T::deserialize(d).map(PIIRestricted)
    }
}

// SQLx: Tier A has no direct Encode impl; the caller must apply column-level
// encryption first.  Decode is allowed (reading ciphertext from the DB into
// the application, where it is then decrypted).
#[cfg(feature = "sqlx-postgres")]
impl<'r, T> sqlx::Decode<'r, sqlx::Postgres> for PIIRestricted<T>
where
    T: sqlx::Decode<'r, sqlx::Postgres>,
{
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        T::decode(value).map(PIIRestricted)
    }
}

#[cfg(feature = "sqlx-postgres")]
impl<T> sqlx::Type<sqlx::Postgres> for PIIRestricted<T>
where
    T: sqlx::Type<sqlx::Postgres>,
{
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        T::type_info()
    }
}

// ---------------------------------------------------------------------------
// Tier B: PIISensitive
// ---------------------------------------------------------------------------

/// Tier-B PII wrapper (Voter operational data, TDE at database level).
///
/// # Example
///
/// ```
/// use eel_pii_rs::{PIISensitive, Purpose, NoopAuditSink};
/// let assignment = PIISensitive::new("PS-001".to_string());
/// let val = assignment.reveal(Purpose::Identification, &NoopAuditSink);
/// assert_eq!(val, "PS-001");
/// ```
pub struct PIISensitive<T>(T);

impl<T> PIISensitive<T> {
    /// Wrap `value` as Tier-B sensitive PII.
    pub fn new(value: T) -> Self {
        PIISensitive(value)
    }

    /// Reveal the inner value, recording the access in `sink`.
    pub fn reveal<S: PIIAuditSink>(&self, purpose: Purpose, sink: &S) -> &T {
        sink.record_access(PIIDataClass::B, purpose, std::any::type_name::<T>());
        &self.0
    }

    /// Consume `self`, returning the inner value and recording the access.
    pub fn into_inner<S: PIIAuditSink>(self, purpose: Purpose, sink: &S) -> T {
        sink.record_access(PIIDataClass::B, purpose, std::any::type_name::<T>());
        self.0
    }

    /// Return a [`SerdeRedact`] placeholder.
    pub fn as_redacted(&self) -> SerdeRedact {
        SerdeRedact
    }
}

impl<T> Classified for PIISensitive<T> {
    fn data_class() -> PIIDataClass {
        PIIDataClass::B
    }
}

impl<T> fmt::Debug for PIISensitive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PIISensitive(***REDACTED***)")
    }
}

impl<T> fmt::Display for PIISensitive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***REDACTED***")
    }
}

impl<T: Serialize> Serialize for PIISensitive<T> {
    fn serialize<S: serde::Serializer>(&self, _s: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(
            "PIISensitive (Tier B) cannot be serialised directly; \
             use .as_redacted() or .reveal(purpose, sink)",
        ))
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for PIISensitive<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        T::deserialize(d).map(PIISensitive)
    }
}

// Tier B: TDE is at DB level, so encoding is transparent.
#[cfg(feature = "sqlx-postgres")]
impl<T> sqlx::Type<sqlx::Postgres> for PIISensitive<T>
where
    T: sqlx::Type<sqlx::Postgres>,
{
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        T::type_info()
    }
}

#[cfg(feature = "sqlx-postgres")]
impl<'q, T> sqlx::Encode<'q, sqlx::Postgres> for PIISensitive<T>
where
    T: sqlx::Encode<'q, sqlx::Postgres>,
{
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        self.0.encode_by_ref(buf)
    }
}

#[cfg(feature = "sqlx-postgres")]
impl<'r, T> sqlx::Decode<'r, sqlx::Postgres> for PIISensitive<T>
where
    T: sqlx::Decode<'r, sqlx::Postgres>,
{
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        T::decode(value).map(PIISensitive)
    }
}

// ---------------------------------------------------------------------------
// Macro for transparent tiers C / D / E
// ---------------------------------------------------------------------------

macro_rules! transparent_pii_tier {
    (
        $name:ident,
        $class:expr,
        $doc_tier:literal,
        $doc_example_val:expr,
        $doc_purpose:expr
    ) => {
        #[doc = concat!("Tier-", $doc_tier, " PII wrapper.")]
        ///
        /// This tier is transparent for serialisation purposes: `Debug`,
        /// `Display`, and `Serialize` all delegate to the inner type.
        ///
        /// # Example
        ///
        /// ```
        #[doc = concat!("use eel_pii_rs::", stringify!($name), ";")]
        #[doc = concat!("let v = ", stringify!($name), "::new(", stringify!($doc_example_val), ");")]
        #[doc = concat!("assert_eq!(format!(\"{v:?}\"), format!(\"{:?}\", ", stringify!($doc_example_val), "));")]
        /// ```
        #[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name<T>(pub T);

        impl<T> $name<T> {
            /// Wrap `value`.
            pub fn new(value: T) -> Self {
                $name(value)
            }

            /// Access the inner value directly (no audit needed for this tier).
            pub fn inner(&self) -> &T {
                &self.0
            }

            /// Consume `self`, returning the inner value.
            pub fn into_inner(self) -> T {
                self.0
            }
        }

        impl<T> Classified for $name<T> {
            fn data_class() -> PIIDataClass {
                $class
            }
        }

        impl<T: fmt::Debug> fmt::Debug for $name<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<T: fmt::Display> fmt::Display for $name<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        #[cfg(feature = "sqlx-postgres")]
        impl<T> sqlx::Type<sqlx::Postgres> for $name<T>
        where
            T: sqlx::Type<sqlx::Postgres>,
        {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                T::type_info()
            }
        }

        #[cfg(feature = "sqlx-postgres")]
        impl<'q, T> sqlx::Encode<'q, sqlx::Postgres> for $name<T>
        where
            T: sqlx::Encode<'q, sqlx::Postgres>,
        {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer,
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                self.0.encode_by_ref(buf)
            }
        }

        #[cfg(feature = "sqlx-postgres")]
        impl<'r, T> sqlx::Decode<'r, sqlx::Postgres> for $name<T>
        where
            T: sqlx::Decode<'r, sqlx::Postgres>,
        {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                T::decode(value).map($name)
            }
        }
    };
}

transparent_pii_tier!(
    PIIPublic,
    PIIDataClass::C,
    "C",
    "aggregate-results-2027".to_string(),
    Purpose::Publication
);

transparent_pii_tier!(
    PIIOperational,
    PIIDataClass::D,
    "D",
    42u64,
    Purpose::InternalMonitoring
);

transparent_pii_tier!(
    PIIEphemeral,
    PIIDataClass::E,
    "E",
    "session-token".to_string(),
    Purpose::InternalMonitoring
);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // --- redaction -----------------------------------------------------------

    #[test]
    fn restricted_debug_never_leaks() {
        let v = PIIRestricted::new("secret-id-number".to_string());
        let s = format!("{v:?}");
        assert!(
            !s.contains("secret-id-number"),
            "Debug must not contain the value"
        );
        assert!(s.contains("REDACTED"));
    }

    #[test]
    fn sensitive_debug_never_leaks() {
        let v = PIISensitive::new("polling-station-001".to_string());
        let s = format!("{v:?}");
        assert!(
            !s.contains("polling-station-001"),
            "Debug must not contain the value"
        );
    }

    // --- serde refuses tier A/B ----------------------------------------------

    #[test]
    fn restricted_serde_serialize_errors() {
        let v = PIIRestricted::new("value".to_string());
        let result = serde_json::to_string(&v);
        assert!(result.is_err(), "Tier A serialisation must fail");
    }

    #[test]
    fn sensitive_serde_serialize_errors() {
        let v = PIISensitive::new("value".to_string());
        let result = serde_json::to_string(&v);
        assert!(result.is_err(), "Tier B serialisation must fail");
    }

    // --- serde redact --------------------------------------------------------

    #[test]
    fn serde_redact_serializes_as_placeholder() {
        let v = PIIRestricted::new("very-secret".to_string());
        let json = serde_json::to_string(&v.as_redacted()).unwrap();
        assert_eq!(json, r#""***REDACTED***""#);
        assert!(!json.contains("very-secret"));
    }

    // --- reveal and audit ----------------------------------------------------

    #[derive(Default)]
    struct CountingSink(std::sync::Mutex<u32>);

    impl PIIAuditSink for CountingSink {
        fn record_access(&self, _: PIIDataClass, _: Purpose, _: &'static str) {
            *self.0.lock().unwrap() += 1;
        }
    }

    #[test]
    fn reveal_emits_audit_event() {
        let v = PIIRestricted::new(42u32);
        let sink = CountingSink::default();
        let _ = v.reveal(Purpose::Audit, &sink);
        assert_eq!(*sink.0.lock().unwrap(), 1);
    }

    #[test]
    fn reveal_returns_correct_value() {
        let v = PIIRestricted::new("KE99887766".to_string());
        let _ = v.reveal(Purpose::Identification, &NoopAuditSink);
        assert_eq!(
            v.reveal(Purpose::Identification, &NoopAuditSink),
            "KE99887766"
        );
    }

    // --- public / operational / ephemeral are transparent --------------------

    #[test]
    fn public_debug_transparent() {
        let v = PIIPublic::new(42u32);
        assert_eq!(format!("{v:?}"), "42");
    }

    #[test]
    fn operational_serde_roundtrip() {
        let v = PIIOperational::new(99u64);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "99");
        let back: PIIOperational<u64> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.into_inner(), 99);
    }

    // --- proptest: Debug and Display never leak for Tier A/B ----------------

    proptest! {
        #[test]
        fn prop_restricted_debug_never_leaks(s in ".*") {
            let v = PIIRestricted::new(s.clone());
            let debug = format!("{v:?}");
            // Only check for leakage if the value is non-trivial.
            if !s.is_empty() && !s.contains("REDACTED") && s.len() > 2 {
                // The debug must not contain the raw value.
                prop_assert!(
                    !debug.contains(&s),
                    "Debug leaked: {debug:?} contains {s:?}"
                );
            }
        }

        #[test]
        fn prop_sensitive_serialize_always_errors(s in ".*") {
            let v = PIISensitive::new(s.clone());
            let result = serde_json::to_string(&v);
            prop_assert!(result.is_err(), "Tier B serialize must always error");
        }
    }
}
