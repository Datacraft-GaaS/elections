// eel-synth: deterministic synthetic data generator.
// Every public module is re-exported for use in tests and integration harnesses.

pub mod embassies;
pub mod forms;
pub mod geo;
pub mod kits;
pub mod names;
pub mod output;
pub mod personnel;
pub mod rng;
pub mod verify;
pub mod voters;

// ---------------------------------------------------------------------------
// Shared scale profiles
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleProfile {
    pub voters: u64,
    pub stations: u64,
    pub forms: u64,
    pub kits: u64,
    pub personnel: u64,
    pub embassies: u64,
}

impl ScaleProfile {
    /// Canonical "default" profile used by `all --scale default` and `verify`.
    pub const DEFAULT: Self = Self {
        voters: 100_000,
        stations: 4_600,
        forms: 200,
        kits: 6_500,
        personnel: 25_000,
        embassies: 22,
    };

    /// Tiny profile for fast smoke tests.
    pub const SMOKE: Self = Self {
        voters: 100,
        stations: 10,
        forms: 5,
        kits: 20,
        personnel: 50,
        embassies: 5,
    };
}

impl std::str::FromStr for ScaleProfile {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "default" => Ok(Self::DEFAULT),
            "smoke" | "test" => Ok(Self::SMOKE),
            _ => anyhow::bail!(
                "unknown scale profile '{}'. Valid values: default, smoke",
                s
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Output format
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Csv,
    Parquet,
    Sql,
}

impl std::str::FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "csv" => Ok(Self::Csv),
            "parquet" => Ok(Self::Parquet),
            "sql" => Ok(Self::Sql),
            _ => anyhow::bail!(
                "unknown format '{}'. Valid values: csv, parquet, sql",
                s
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Degradation flags for Form 34A images
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DegradationFlags {
    pub torn: bool,
    pub wet: bool,
    pub angled: bool,
    pub low_light: bool,
}

impl Default for DegradationFlags {
    fn default() -> Self {
        Self { torn: false, wet: false, angled: false, low_light: false }
    }
}

impl std::str::FromStr for DegradationFlags {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let mut flags = Self::default();
        for token in s.split(',') {
            match token.trim().to_ascii_lowercase().as_str() {
                "torn" => flags.torn = true,
                "wet" => flags.wet = true,
                "angled" => flags.angled = true,
                "low-light" | "lowlight" | "low_light" => flags.low_light = true,
                "" => {}
                other => anyhow::bail!("unknown degradation flag '{}'", other),
            }
        }
        Ok(flags)
    }
}
