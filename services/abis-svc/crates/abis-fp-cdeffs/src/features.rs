//! Extended Feature Set structs.
//!
//! Reference: NIST SP 500-290 e3 Annex A; ANSI/NIST-ITL Type-9 fields 300-999.
//!
//! Field mapping (non-exhaustive):
//!   9.300  Pattern class
//!   9.301  Singular points (cores + deltas)
//!   9.302  Ridge quality map
//!   9.303  Ridge flow map (orientation field)
//!   9.304  Ridge wavelength (inter-ridge spacing)
//!   9.320  Minutia extended attributes (ridge counts, feature type)
//!   9.330  Dots (isolated ridge units)
//!   9.331  Incipient ridges
//!   9.350  Creases, scars, warts, skin conditions
//!   9.360  Ridge edge shapes

use serde::{Deserialize, Serialize};

/// ANSI/NIST-ITL Type-9 field 9.300 -- Level-1 pattern class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternClass {
    Arch,
    TentedArch,
    LeftLoop,
    RightLoop,
    Whorl,
    TwinLoop,
    CentralPocketLoop,
    Accidental,
    Unknown,
}

impl PatternClass {
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => Self::Arch,
            1 => Self::TentedArch,
            2 => Self::LeftLoop,
            3 => Self::RightLoop,
            4 => Self::Whorl,
            5 => Self::TwinLoop,
            6 => Self::CentralPocketLoop,
            7 => Self::Accidental,
            _ => Self::Unknown,
        }
    }

    pub fn to_code(self) -> u8 {
        match self {
            Self::Arch => 0,
            Self::TentedArch => 1,
            Self::LeftLoop => 2,
            Self::RightLoop => 3,
            Self::Whorl => 4,
            Self::TwinLoop => 5,
            Self::CentralPocketLoop => 6,
            Self::Accidental => 7,
            Self::Unknown => 255,
        }
    }
}

/// Kind of singular point (Type-9 field 9.301).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SingularPointKind {
    Core,
    Delta,
}

/// One singular point with position and orientation (Type-9 field 9.301).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingularPoint {
    pub kind: SingularPointKind,
    /// X coordinate in pixels.
    pub x: u16,
    /// Y coordinate in pixels.
    pub y: u16,
    /// Orientation in degrees [0, 360).
    pub orientation: f32,
}

/// Per-pixel ridge quality map (Type-9 field 9.302).
/// Values 0-100; 0 = no ridge, 100 = perfect ridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RidgeQualityMap {
    pub width: u16,
    pub height: u16,
    /// Row-major, one byte per pixel, value 0-100.
    pub data: Vec<u8>,
}

/// Per-pixel orientation field (Type-9 field 9.303).
/// Values 0-179 degrees (half-plane, ridge direction).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RidgeFlowMap {
    pub width: u16,
    pub height: u16,
    /// Row-major, one byte per pixel in [0, 179].
    pub data: Vec<u8>,
}

/// Per-pixel ridge wavelength map (Type-9 field 9.304).
/// In units of 1/100 mm (fixed-point).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RidgeWavelengthMap {
    pub width: u16,
    pub height: u16,
    /// Row-major, u16 per pixel.
    pub data: Vec<u16>,
}

/// Binary foreground/background segmentation mask.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentationMask {
    pub width: u16,
    pub height: u16,
    /// Packed bits, row-major. Bit 1 = foreground.
    pub data: Vec<u8>,
}

/// Extended minutia attribute (Type-9 field 9.320).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinutiaExtended {
    /// Index into the base minutiae list.
    pub minutia_idx: usize,
    /// Ridge counts to K nearest neighbours: (neighbour_idx, count).
    pub ridge_counts_to_neighbours: Vec<(usize, u8)>,
    /// Feature type (e.g. 1=ending, 2=bifurcation, 3=dot, 4=short ridge).
    pub feature_type: u8,
    /// True if this minutia is actually an isolated dot.
    pub is_dot: bool,
    /// True if this minutia is on an incipient (nascent) ridge.
    pub is_incipient: bool,
}

/// An isolated ridge dot (Type-9 field 9.330).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dot {
    pub x: u16,
    pub y: u16,
    pub quality: u8,
}

/// An incipient (nascent) ridge segment (Type-9 field 9.331).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncipientRidge {
    pub start_x: u16,
    pub start_y: u16,
    pub end_x: u16,
    pub end_y: u16,
    pub quality: u8,
}

/// Skin condition or scar/crease/wart feature (Type-9 field 9.350).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkinConditionKind {
    Crease,
    Scar,
    Wart,
    OtherSkinCondition,
}

/// One skin condition region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinCondition {
    pub kind: SkinConditionKind,
    /// Bounding box (top-left corner + dimensions).
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Ridge edge shape at a minutia (Type-9 field 9.360).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RidgeEdgeShape {
    pub minutia_idx: usize,
    /// Curvature value (higher = more curved).
    pub curvature: f32,
    /// Ridge width at this minutia in pixels.
    pub ridge_width: f32,
}

/// Full ANSI/NIST-ITL Type-9 Extended Feature Set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedFeatureSet {
    /// Level-1 pattern class (field 9.300).
    pub pattern_class: PatternClass,
    /// Singular points: cores and deltas (field 9.301).
    pub singular_points: Vec<SingularPoint>,
    /// Ridge quality map (field 9.302).
    pub quality_map: Option<RidgeQualityMap>,
    /// Ridge flow / orientation map (field 9.303).
    pub flow_map: Option<RidgeFlowMap>,
    /// Ridge wavelength map (field 9.304).
    pub wavelength_map: Option<RidgeWavelengthMap>,
    /// Segmentation mask -- foreground/background.
    pub segmentation: Option<SegmentationMask>,
    /// Extended minutia attributes (field 9.320).
    pub minutia_extended: Vec<MinutiaExtended>,
    /// Isolated ridge dots (field 9.330).
    pub dots: Vec<Dot>,
    /// Incipient ridges (field 9.331).
    pub incipient_ridges: Vec<IncipientRidge>,
    /// Skin conditions, creases, scars (field 9.350).
    pub skin_conditions: Vec<SkinCondition>,
    /// Ridge edge shapes (field 9.360).
    pub ridge_edge_shapes: Vec<RidgeEdgeShape>,
}

impl ExtendedFeatureSet {
    pub fn new(pattern_class: PatternClass) -> Self {
        Self {
            pattern_class,
            singular_points: Vec::new(),
            quality_map: None,
            flow_map: None,
            wavelength_map: None,
            segmentation: None,
            minutia_extended: Vec::new(),
            dots: Vec::new(),
            incipient_ridges: Vec::new(),
            skin_conditions: Vec::new(),
            ridge_edge_shapes: Vec::new(),
        }
    }

    /// Count cores (Type-9 field 9.301 Core entries).
    pub fn core_count(&self) -> usize {
        self.singular_points
            .iter()
            .filter(|p| p.kind == SingularPointKind::Core)
            .count()
    }

    /// Count deltas (Type-9 field 9.301 Delta entries).
    pub fn delta_count(&self) -> usize {
        self.singular_points
            .iter()
            .filter(|p| p.kind == SingularPointKind::Delta)
            .count()
    }
}
