//! Cover tag lifecycle FSM.
//!
//! Encodes the allowed-transitions table from section 5 of the spec.
//! Any transition not listed here is rejected with `TransitionNotAllowed`.

use crate::domain::{CoverState, ReaderType};
use crate::error::RfidError;

// ---------------------------------------------------------------------------
// Transition descriptor
// ---------------------------------------------------------------------------

/// One allowed transition in the FSM table.
#[derive(Debug, Clone)]
pub struct AllowedTransition {
    pub from: CoverState,
    pub to: CoverState,
    /// Reader types that are permitted to trigger this transition.
    pub required_reader_types: Vec<ReaderType>,
    /// Whether a dual-officer signature (four-eyes) is required.
    pub four_eyes: bool,
}

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

pub struct CoverStateMachine;

impl CoverStateMachine {
    /// Full transition table per spec section 5.
    pub fn allowed_transitions() -> Vec<AllowedTransition> {
        vec![
            AllowedTransition {
                from: CoverState::Manufactured,
                to: CoverState::Printer,
                required_reader_types: vec![ReaderType::WarehouseGate],
                four_eyes: false,
            },
            AllowedTransition {
                from: CoverState::Printer,
                to: CoverState::Warehouse,
                required_reader_types: vec![ReaderType::WarehouseGate],
                four_eyes: false,
            },
            AllowedTransition {
                from: CoverState::Warehouse,
                to: CoverState::Dispatched,
                required_reader_types: vec![ReaderType::WarehouseGate],
                four_eyes: true,
            },
            AllowedTransition {
                from: CoverState::Dispatched,
                to: CoverState::InTransit,
                required_reader_types: vec![ReaderType::TransitCheckpoint],
                four_eyes: false,
            },
            AllowedTransition {
                from: CoverState::InTransit,
                to: CoverState::StationReceived,
                required_reader_types: vec![ReaderType::StationArrival],
                four_eyes: false,
            },
            AllowedTransition {
                from: CoverState::StationReceived,
                to: CoverState::StationOpened,
                required_reader_types: vec![ReaderType::KitHandheld],
                four_eyes: true,
            },
            AllowedTransition {
                from: CoverState::StationOpened,
                to: CoverState::VotingActive,
                required_reader_types: vec![ReaderType::KitHandheld],
                four_eyes: false,
            },
            AllowedTransition {
                from: CoverState::VotingActive,
                to: CoverState::StationClosed,
                required_reader_types: vec![ReaderType::KitHandheld],
                four_eyes: true,
            },
            AllowedTransition {
                from: CoverState::StationClosed,
                to: CoverState::SealedReturn,
                required_reader_types: vec![ReaderType::KitHandheld],
                four_eyes: false,
            },
            AllowedTransition {
                from: CoverState::SealedReturn,
                to: CoverState::TallyArrived,
                required_reader_types: vec![ReaderType::TransitCheckpoint],
                four_eyes: false,
            },
            AllowedTransition {
                from: CoverState::TallyArrived,
                to: CoverState::CountComplete,
                required_reader_types: vec![ReaderType::RoDesktop],
                four_eyes: false,
            },
            AllowedTransition {
                from: CoverState::CountComplete,
                to: CoverState::StorageArchive,
                required_reader_types: vec![ReaderType::ArchiveGate],
                four_eyes: false,
            },
            AllowedTransition {
                from: CoverState::StorageArchive,
                to: CoverState::Destroyed,
                required_reader_types: vec![ReaderType::ArchiveGate],
                four_eyes: true,
            },
            // QUARANTINE is reachable from any state by any reader on tamper detection.
            // It is handled as a special case in validate() rather than enumerated for all 14 sources.
        ]
    }

    /// Validate whether transitioning `from -> to` is permitted for the given reader type.
    ///
    /// QUARANTINE is always allowed from any non-terminal state.
    /// Four-eyes requirement is checked when `officer_svid` is None.
    pub fn validate(
        from: &CoverState,
        to: &CoverState,
        reader_type: &ReaderType,
        officer_svid: Option<&str>,
    ) -> Result<(), RfidError> {
        // Terminal states may only transition to QUARANTINE (or not at all if already QUARANTINE).
        if from.is_terminal() && to != &CoverState::Quarantine {
            return Err(RfidError::TransitionFromTerminal(from.to_string()));
        }

        // QUARANTINE is reachable from any state on tamper.
        if to == &CoverState::Quarantine {
            return Ok(());
        }

        // Look up in the explicit table.
        let table = Self::allowed_transitions();
        let rule = table
            .iter()
            .find(|t| &t.from == from && &t.to == to)
            .ok_or_else(|| RfidError::TransitionNotAllowed {
                from: from.to_string(),
                to: to.to_string(),
                reader_type: reader_type.to_string(),
            })?;

        // Check reader type.
        if !rule.required_reader_types.contains(reader_type) {
            return Err(RfidError::TransitionNotAllowed {
                from: from.to_string(),
                to: to.to_string(),
                reader_type: reader_type.to_string(),
            });
        }

        // Check four-eyes requirement.
        if rule.four_eyes && officer_svid.is_none() {
            return Err(RfidError::FourEyesRequired {
                from: from.to_string(),
                to: to.to_string(),
            });
        }

        Ok(())
    }

    /// Returns true if the transition `from -> to` is in the allowed table.
    pub fn is_allowed(from: &CoverState, to: &CoverState, reader_type: &ReaderType) -> bool {
        // QUARANTINE always allowed on tamper.
        if to == &CoverState::Quarantine {
            return true;
        }
        // Terminal states block all other transitions.
        if from.is_terminal() {
            return false;
        }
        Self::allowed_transitions().iter().any(|t| {
            &t.from == from && &t.to == to && t.required_reader_types.contains(reader_type)
        })
    }
}
