//! In-memory store for the public training contract surface.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::spec::{
    AccreditationApplication, CompletionRecord, Course, Enrolment, MedicalIncident,
    MedicalPersonnel,
};

/// Shared spec-state handle.
#[derive(Default)]
pub struct SpecState {
    pub courses: Mutex<HashMap<String, Course>>,
    pub enrolments: Mutex<HashMap<String, Enrolment>>,
    pub completions: Mutex<HashMap<String, CompletionRecord>>,
    pub applications: Mutex<HashMap<String, AccreditationApplication>>,
    pub incidents: Mutex<HashMap<String, MedicalIncident>>,
    pub personnel: Mutex<HashMap<String, MedicalPersonnel>>,
}

/// Shared spec-state handle threaded through the router.
pub type SharedSpecState = Arc<SpecState>;

impl SpecState {
    /// Construct an empty spec store.
    pub fn new() -> SharedSpecState {
        Arc::new(Self::default())
    }
}
