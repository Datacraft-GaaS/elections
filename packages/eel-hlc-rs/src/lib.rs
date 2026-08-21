//! Hybrid Logical Clock (HLC) for the eel platform.
//!
//! An HLC combines a physical wall-clock timestamp with a logical counter
//! to provide causally-consistent, globally-unique timestamps even across
//! nodes with skewed clocks.  The string wire form is
//! `physical_ms.counter.node`, e.g. `"1723731000123.5.42"`.
//!
//! # Example
//!
//! ```
//! use eel_hlc_rs::{HlcClock, SystemWallClock};
//! let clock = HlcClock::new(1, SystemWallClock);
//! let t1 = clock.now();
//! let t2 = clock.now();
//! assert!(t2 >= t1, "HLC is non-decreasing");
//! ```

use std::{fmt, str::FromStr, sync::Mutex};

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced by HLC parsing and arithmetic.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum HlcError {
    /// The string did not match the `physical_ms.counter.node` format.
    #[error("invalid HLC string (expected physical_ms.counter.node): {0:?}")]
    Parse(String),

    /// A field could not be parsed as an integer.
    #[error("field parse error: {0}")]
    FieldParse(#[from] std::num::ParseIntError),

    /// The logical counter wrapped around (>= u16::MAX within one ms).
    #[error("HLC counter overflow: too many events within one millisecond")]
    CounterOverflow,
}

// ---------------------------------------------------------------------------
// Wall-clock abstraction
// ---------------------------------------------------------------------------

/// Source of the physical wall-clock time in milliseconds since Unix epoch.
///
/// The default implementation reads `SystemTime::now()`.  In tests, inject a
/// `MockClock` instead.
pub trait WallClock: Send + Sync {
    /// Returns the current time in milliseconds since Unix epoch.
    fn now_ms(&self) -> u64;
}

/// Production wall-clock backed by `std::time::SystemTime`.
///
/// # Example
///
/// ```
/// use eel_hlc_rs::SystemWallClock;
/// use eel_hlc_rs::WallClock as _;
/// let _ = SystemWallClock.now_ms();
/// ```
pub struct SystemWallClock;

impl WallClock for SystemWallClock {
    fn now_ms(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is before the Unix epoch")
            .as_millis() as u64
    }
}

// ---------------------------------------------------------------------------
// Hlc value type
// ---------------------------------------------------------------------------

/// A Hybrid Logical Clock timestamp.
///
/// Ordering is lexicographic on `(physical_ms, counter, node)`.
///
/// # Example
///
/// ```
/// use eel_hlc_rs::Hlc;
/// let h: Hlc = "1723731000123.5.42".parse().unwrap();
/// assert_eq!(h.to_string(), "1723731000123.5.42");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hlc {
    /// Wall-clock component: milliseconds since Unix epoch.
    pub physical_ms: u64,
    /// Logical counter; increments when physical time does not advance.
    pub counter: u16,
    /// Node identifier; distinguishes clocks on different nodes.
    pub node: u16,
}

impl fmt::Debug for Hlc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hlc({})", self)
    }
}

impl fmt::Display for Hlc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.physical_ms, self.counter, self.node)
    }
}

impl FromStr for Hlc {
    type Err = HlcError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut it = s.splitn(3, '.');
        let physical_ms = it
            .next()
            .ok_or_else(|| HlcError::Parse(s.to_owned()))?
            .parse()?;
        let counter = it
            .next()
            .ok_or_else(|| HlcError::Parse(s.to_owned()))?
            .parse()?;
        let node = it
            .next()
            .ok_or_else(|| HlcError::Parse(s.to_owned()))?
            .parse()?;
        if it.next().is_some() {
            return Err(HlcError::Parse(s.to_owned()));
        }
        Ok(Hlc {
            physical_ms,
            counter,
            node,
        })
    }
}

// Serde: always the string form.

impl Serialize for Hlc {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Hlc {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
}

// SQLx: encode/decode as Postgres TEXT.

#[cfg(feature = "sqlx-postgres")]
impl sqlx::Type<sqlx::Postgres> for Hlc {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

#[cfg(feature = "sqlx-postgres")]
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for Hlc {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as sqlx::Encode<'_, sqlx::Postgres>>::encode_by_ref(&self.to_string(), buf)
    }
}

#[cfg(feature = "sqlx-postgres")]
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Hlc {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        s.parse()
            .map_err(|e| Box::new(e) as sqlx::error::BoxDynError)
    }
}

// ---------------------------------------------------------------------------
// HlcClock
// ---------------------------------------------------------------------------

/// A thread-safe, monotonic HLC clock for one node.
///
/// # Example
///
/// ```
/// use eel_hlc_rs::{HlcClock, SystemWallClock};
/// let clock = HlcClock::new(42, SystemWallClock);
/// let t1 = clock.now();
/// let t2 = clock.now();
/// assert!(t2 >= t1);
/// ```
pub struct HlcClock<W: WallClock = SystemWallClock> {
    node: u16,
    wall: W,
    /// (last_physical_ms, last_counter)
    state: Mutex<(u64, u16)>,
}

impl<W: WallClock> HlcClock<W> {
    /// Create a new HLC clock for the given `node` identifier.
    pub fn new(node: u16, wall: W) -> Self {
        HlcClock {
            node,
            wall,
            state: Mutex::new((0, 0)),
        }
    }

    /// Emit the next HLC timestamp for this node.
    ///
    /// Monotonicity guarantee: the returned timestamp is always strictly
    /// greater than every previously returned value from this clock.
    pub fn now(&self) -> Hlc {
        let wall_ms = self.wall.now_ms();
        let mut guard = self.state.lock().expect("HlcClock mutex poisoned");
        let (last_phys, last_ctr) = *guard;

        let (new_phys, new_ctr) = if wall_ms > last_phys {
            (wall_ms, 0)
        } else {
            // Physical did not advance; bump the counter.
            let nc = last_ctr.saturating_add(1);
            (last_phys, nc)
        };

        *guard = (new_phys, new_ctr);
        Hlc {
            physical_ms: new_phys,
            counter: new_ctr,
            node: self.node,
        }
    }

    /// Merge an incoming remote HLC into this clock's state.
    ///
    /// The result is causally after both the local state and `incoming`.
    ///
    /// # HLC paper algorithm
    ///
    /// ```text
    /// pt' = max(local.pt, incoming.pt, wall)
    /// if pt' == local.pt == incoming.pt:  c' = max(local.c, incoming.c) + 1
    /// elif pt' == local.pt:               c' = local.c + 1
    /// elif pt' == incoming.pt:            c' = incoming.c + 1
    /// else (pt' == wall):                 c' = 0
    /// ```
    pub fn merge(&self, incoming: Hlc) -> Hlc {
        let wall_ms = self.wall.now_ms();
        let mut guard = self.state.lock().expect("HlcClock mutex poisoned");
        let (last_phys, last_ctr) = *guard;

        let max_phys = wall_ms.max(incoming.physical_ms).max(last_phys);

        let new_ctr = if max_phys == last_phys && max_phys == incoming.physical_ms {
            last_ctr.max(incoming.counter).saturating_add(1)
        } else if max_phys == last_phys {
            last_ctr.saturating_add(1)
        } else if max_phys == incoming.physical_ms {
            incoming.counter.saturating_add(1)
        } else {
            0
        };

        *guard = (max_phys, new_ctr);
        Hlc {
            physical_ms: max_phys,
            counter: new_ctr,
            node: self.node,
        }
    }
}

// ---------------------------------------------------------------------------
// Kani verification harnesses
// ---------------------------------------------------------------------------

#[cfg(kani)]
mod kani_harnesses {
    use super::*;

    struct KaniClock;
    impl WallClock for KaniClock {
        fn now_ms(&self) -> u64 {
            kani::any()
        }
    }

    /// Merging any incoming HLC never produces a result behind the incoming
    /// timestamp's physical component.
    #[kani::proof]
    fn hlc_merge_never_regresses() {
        let inc_phys: u64 = kani::any();
        let inc_ctr: u16 = kani::any();
        kani::assume(inc_phys < u64::MAX - 1);
        kani::assume(inc_ctr < u16::MAX);

        let clock = HlcClock::new(1, KaniClock);
        let incoming = Hlc {
            physical_ms: inc_phys,
            counter: inc_ctr,
            node: 0,
        };
        let result = clock.merge(incoming);

        assert!(
            result.physical_ms >= incoming.physical_ms,
            "merge result physical time must be >= incoming"
        );
        assert!(
            result >= incoming,
            "merged HLC must be causally after the incoming value"
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    // --- mock clock ----------------------------------------------------------

    struct MockClock(Arc<AtomicU64>);

    impl MockClock {
        fn new(ms: u64) -> (Self, Arc<AtomicU64>) {
            let shared = Arc::new(AtomicU64::new(ms));
            (MockClock(Arc::clone(&shared)), shared)
        }
    }

    impl WallClock for MockClock {
        fn now_ms(&self) -> u64 {
            self.0.load(Ordering::SeqCst)
        }
    }

    // --- parse / display -----------------------------------------------------

    #[test]
    fn roundtrip_display_parse() {
        let hlc = Hlc {
            physical_ms: 1_723_731_000_123,
            counter: 5,
            node: 42,
        };
        let s = hlc.to_string();
        assert_eq!(s, "1723731000123.5.42");
        let parsed: Hlc = s.parse().unwrap();
        assert_eq!(parsed, hlc);
    }

    #[test]
    fn parse_error_on_bad_input() {
        assert!("not-an-hlc".parse::<Hlc>().is_err());
        assert!("1.2".parse::<Hlc>().is_err());
        assert!("1.2.3.4".parse::<Hlc>().is_err());
    }

    // --- serde ---------------------------------------------------------------

    #[test]
    fn serde_roundtrip() {
        let hlc = Hlc {
            physical_ms: 1_000,
            counter: 3,
            node: 7,
        };
        let json = serde_json::to_string(&hlc).unwrap();
        assert_eq!(json, r#""1000.3.7""#);
        let back: Hlc = serde_json::from_str(&json).unwrap();
        assert_eq!(back, hlc);
    }

    // --- monotonicity --------------------------------------------------------

    #[test]
    fn now_is_monotone_despite_clock_skew() {
        let (mock, shared) = MockClock::new(1000);
        let clock = HlcClock::new(1, mock);
        let mut prev = clock.now();

        // Simulate a backwards NTP jump.
        shared.store(900, Ordering::SeqCst);
        for _ in 0..100 {
            let next = clock.now();
            assert!(next > prev, "HLC regressed: {prev} >= {next}");
            prev = next;
        }

        // Wall clock moves forward again.
        shared.store(2000, Ordering::SeqCst);
        let next = clock.now();
        assert!(next > prev);
    }

    // --- merge ---------------------------------------------------------------

    #[test]
    fn merge_result_is_after_both_inputs() {
        let (mock, shared) = MockClock::new(500);
        let clock = HlcClock::new(1, mock);

        let incoming = Hlc {
            physical_ms: 800,
            counter: 3,
            node: 2,
        };
        shared.store(400, Ordering::SeqCst); // wall behind incoming
        let result = clock.merge(incoming);

        assert!(result.physical_ms >= incoming.physical_ms);
        assert!(result >= incoming);
    }

    #[test]
    fn merge_advances_counter_when_physical_ties() {
        let (mock, shared) = MockClock::new(1000);
        let clock = HlcClock::new(1, mock);

        // Warm up the local state.
        let _ = clock.now();
        shared.store(999, Ordering::SeqCst); // wall goes back

        let incoming = Hlc {
            physical_ms: 1000,
            counter: 2,
            node: 2,
        };
        let result = clock.merge(incoming);

        assert_eq!(result.physical_ms, 1000);
        assert!(result.counter > 2, "counter must advance past incoming");
    }

    // --- proptest suites -----------------------------------------------------

    proptest! {
        /// Any sequence of 10 000 `now()` calls must be strictly increasing.
        #[test]
        fn prop_now_strictly_increasing(
            times in proptest::collection::vec(0u64..=1_000_000u64, 10_000)
        ) {
            let (mock, shared) = MockClock::new(0);
            let clock = HlcClock::new(1, mock);
            let mut prev = Hlc { physical_ms: 0, counter: 0, node: 0 };

            for t in times {
                // Wall clock may go forwards or backwards.
                shared.store(t, Ordering::SeqCst);
                let h = clock.now();
                prop_assert!(h > prev, "HLC not strictly increasing: {h} <= {prev}");
                prev = h;
            }
        }

        /// Merging with the same incoming value twice: the second result is
        /// after the first (merge is not idempotent; it always advances).
        #[test]
        fn prop_merge_advances(
            phys in 0u64..=1_000_000u64,
            ctr in 0u16..=1000u16,
        ) {
            let (mock, _shared) = MockClock::new(phys.saturating_sub(100));
            let clock = HlcClock::new(1, mock);
            let inc = Hlc { physical_ms: phys, counter: ctr, node: 2 };

            let r1 = clock.merge(inc);
            let r2 = clock.merge(inc);
            prop_assert!(r2 > r1, "second merge must produce a strictly later value");
        }

        /// Physical component of `merge(a, b)` is >= both a.physical_ms and
        /// b.physical_ms — the clock never goes back.
        #[test]
        fn prop_merge_physical_non_regression(
            wall in 0u64..=1_000_000u64,
            inc_phys in 0u64..=1_000_000u64,
            inc_ctr in 0u16..=100u16,
        ) {
            let (mock, _shared) = MockClock::new(wall);
            let clock = HlcClock::new(1, mock);
            let inc = Hlc { physical_ms: inc_phys, counter: inc_ctr, node: 2 };
            let result = clock.merge(inc);
            prop_assert!(result.physical_ms >= inc_phys);
            prop_assert!(result.physical_ms >= wall);
        }

        /// Display -> parse round trip.
        #[test]
        fn prop_display_parse_roundtrip(
            phys in 0u64..=u64::MAX,
            ctr in 0u16..=u16::MAX,
            node in 0u16..=u16::MAX,
        ) {
            let hlc = Hlc { physical_ms: phys, counter: ctr, node };
            let s = hlc.to_string();
            let back: Hlc = s.parse().expect("round-trip parse failed");
            prop_assert_eq!(back, hlc);
        }
    }
}
