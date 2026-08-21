//! Iris biometric score fusion: bilateral (left+right) and biometric (iris+periocular).

pub mod bilateral;
pub mod biometric;

pub use bilateral::BilateralFusion;
pub use biometric::BiometricFusion;
