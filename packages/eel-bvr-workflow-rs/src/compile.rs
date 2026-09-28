//! Resolves a [`WorkflowSpec`] into a [`CompiledWorkflow`].
//!
//! This is the equivalent of BiCCL's reference compiler: it turns a declarative
//! document into a form the client can execute without further interpretation,
//! and refuses to produce one when the document is internally inconsistent.

use std::collections::{BTreeSet, HashMap};

use thiserror::Error;

use crate::model::{Constraints, DataItem, DataKind, StepKind, WorkflowSpec};

/// Why a spec could not be compiled.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CompileError {
    /// `version` is not understood by this build.
    #[error("unsupported workflow spec version {got}, expected 1")]
    UnsupportedVersion {
        /// The version found in the document.
        got: u32,
    },
    /// The workflow declares no data items.
    #[error("workflow declares no data items")]
    NoDataItems,
    /// The workflow declares no steps.
    #[error("workflow declares no steps")]
    NoSteps,
    /// Two data items share a name.
    #[error("duplicate data item name {name:?}")]
    DuplicateDataItem {
        /// The repeated name.
        name: String,
    },
    /// Two steps share an id.
    #[error("duplicate step id {id:?}")]
    DuplicateStepId {
        /// The repeated id.
        id: String,
    },
    /// A step referenced a data item that does not exist.
    #[error("step {step:?} references unknown data item {item:?}")]
    UnknownDataItem {
        /// The referring step.
        step: String,
        /// The unresolvable item name.
        item: String,
    },
    /// A data item is never collected by any step.
    #[error("data item {name:?} is declared but never collected by a step")]
    UnreachableDataItem {
        /// The orphaned item.
        name: String,
    },
    /// `min_length` exceeds `max_length`.
    #[error("data item {name:?} has min_length {min} greater than max_length {max}")]
    LengthBoundsInverted {
        /// The offending item.
        name: String,
        /// The lower bound.
        min: usize,
        /// The upper bound.
        max: usize,
    },
    /// `min_age_years` exceeds `max_age_years`.
    #[error("data item {name:?} has min_age_years {min} greater than max_age_years {max}")]
    AgeBoundsInverted {
        /// The offending item.
        name: String,
        /// The lower bound.
        min: u16,
        /// The upper bound.
        max: u16,
    },
    /// A choice item has no options.
    #[error("data item {name:?} is a choice but declares no options")]
    ChoiceWithoutOptions {
        /// The offending item.
        name: String,
    },
    /// An option list contains a duplicate.
    #[error("data item {name:?} lists option {option:?} more than once")]
    DuplicateOption {
        /// The offending item.
        name: String,
        /// The repeated option.
        option: String,
    },
    /// An age bound was set on a non-date item.
    #[error("data item {name:?} sets age bounds but its kind is not a date")]
    AgeBoundOnNonDate {
        /// The offending item.
        name: String,
    },
    /// A string length bound was set on an item kind that has no string form.
    #[error("data item {name:?} sets length bounds but its kind is {kind:?}")]
    LengthBoundOnNonText {
        /// The offending item.
        name: String,
        /// The offending kind.
        kind: DataKind,
    },
    /// A data class was not one of A, B, or C.
    #[error("data item {name:?} has data class {class:?}, expected one of A, B, C")]
    InvalidDataClass {
        /// The offending item.
        name: String,
        /// The offending value.
        class: String,
    },
    /// Two capture requirements share an id.
    #[error("duplicate capture requirement id {id:?}")]
    DuplicateCaptureId {
        /// The repeated id.
        id: String,
    },
    /// A step referenced a capture requirement that does not exist.
    #[error("step {step:?} references unknown capture requirement {capture:?}")]
    UnknownCapture {
        /// The referring step.
        step: String,
        /// The unresolvable requirement id.
        capture: String,
    },
    /// A capture requirement is never used by any step.
    #[error("capture requirement {id:?} is declared but never used by a step")]
    UnreferencedCapture {
        /// The orphaned requirement.
        id: String,
    },
    /// A capture requirement lists no kinds.
    #[error("capture requirement {id:?} declares no capture kinds")]
    CaptureWithoutKinds {
        /// The offending requirement.
        id: String,
    },
    /// A capture requirement repeats a kind.
    #[error("capture requirement {id:?} lists kind {kind:?} more than once")]
    DuplicateCaptureKind {
        /// The offending requirement.
        id: String,
        /// The repeated kind.
        kind: crate::model::CaptureKind,
    },
    /// `at_least` demands more captures than the requirement lists.
    #[error("capture requirement {id:?} demands at least {wanted} captures but lists only {have}")]
    CapturePolicyUnsatisfiable {
        /// The offending requirement.
        id: String,
        /// The demanded count.
        wanted: u16,
        /// The listed count.
        have: usize,
    },
    /// `at_least` demands zero, which makes the step vacuous.
    #[error("capture requirement {id:?} demands at least 0 captures, which can never fail")]
    CapturePolicyVacuous {
        /// The offending requirement.
        id: String,
    },
}

/// A data item paired with its declared name, ready for form generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledDataItem {
    /// The item as declared.
    pub item: DataItem,
    /// Whether any step collects this item.
    pub reachable: bool,
}

/// A step paired with its resolved data-item names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledStep {
    /// The step as declared, with names verified to exist.
    pub step: crate::model::Step,
    /// Resolved data-item names, empty for non-entry steps.
    pub items: Vec<String>,
    /// Label to display: explicit override, else a generated default.
    pub display_label: String,
}

/// A spec that passed compilation and is safe to execute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledWorkflow {
    /// Stable workflow identifier.
    pub id: String,
    /// Human-readable workflow name.
    pub label: String,
    /// Data items indexed by name, for O(1) lookup during submission validation.
    pub data_items: HashMap<String, CompiledDataItem>,
    /// Data item names in declaration order, for stable form layout.
    pub data_item_order: Vec<String>,
    /// Steps in execution order.
    pub steps: Vec<CompiledStep>,
    /// Named capture requirements indexed by id.
    pub captures: HashMap<String, CompiledCapture>,
    /// Capture requirement ids in declaration order.
    pub capture_order: Vec<String>,
}

impl CompiledWorkflow {
    /// Returns the compiled data item named `name`, if present.
    pub fn data_item(&self, name: &str) -> Option<&CompiledDataItem> {
        self.data_items.get(name)
    }

    /// Returns the compiled capture requirement named `id`, if present.
    pub fn capture(&self, id: &str) -> Option<&CompiledCapture> {
        self.captures.get(id)
    }
}

/// A capture requirement paired with whether any step uses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledCapture {
    /// The requirement as declared.
    pub requirement: crate::model::CaptureRequirement,
    /// Whether any step references this requirement.
    pub referenced: bool,
}

/// Checks `spec` and resolves it into an executable form.
///
/// Every problem found is returned in the `Vec`; compilation is not
/// fail-fast, so a malformed workflow reports all of its faults in one pass.
pub fn compile(spec: &WorkflowSpec) -> Result<CompiledWorkflow, Vec<CompileError>> {
    let mut errors = Vec::new();

    if spec.version != 1 {
        errors.push(CompileError::UnsupportedVersion { got: spec.version });
    }
    if spec.data_items.is_empty() {
        errors.push(CompileError::NoDataItems);
    }
    if spec.steps.is_empty() {
        errors.push(CompileError::NoSteps);
    }

    // Index data items, rejecting duplicates.
    let mut data_items: HashMap<String, CompiledDataItem> = HashMap::new();
    let mut data_item_order: Vec<String> = Vec::new();
    for item in &spec.data_items {
        check_constraints(item, &mut errors);
        if !item.has_valid_data_class() {
            errors.push(CompileError::InvalidDataClass {
                name: item.name.clone(),
                class: item.data_class.clone(),
            });
        }
        if data_items.contains_key(&item.name) {
            errors.push(CompileError::DuplicateDataItem {
                name: item.name.clone(),
            });
            continue;
        }
        data_item_order.push(item.name.clone());
        data_items.insert(
            item.name.clone(),
            CompiledDataItem { item: item.clone(), reachable: false },
        );
    }

    // Index named capture requirements, rejecting duplicates.
        let mut captures: HashMap<String, CompiledCapture> = HashMap::new();
        let mut capture_order: Vec<String> = Vec::new();
        for capture in &spec.capture_requirements {
            check_capture(capture, &mut errors);
            if captures.contains_key(&capture.id) {
                errors.push(CompileError::DuplicateCaptureId { id: capture.id.clone() });
                continue;
            }
            capture_order.push(capture.id.clone());
            captures.insert(
                capture.id.clone(),
                CompiledCapture { requirement: capture.clone(), referenced: false },
            );
        }

        let mut steps: Vec<CompiledStep> = Vec::new();
        let mut seen_step_ids: BTreeSet<&str> = BTreeSet::new();
        for step in &spec.steps {
            if !seen_step_ids.insert(step.id.as_str()) {
                errors.push(CompileError::DuplicateStepId { id: step.id.clone() });
                continue;
            }
            let mut items: Vec<String> = Vec::new();
            let display_label = match &step.kind {
                StepKind::DataEntry { items: names } => {
                    for name in names {
                        match data_items.get_mut(name) {
                            Some(entry) => {
                                entry.reachable = true;
                                items.push(name.clone());
                            }
                            None => errors.push(CompileError::UnknownDataItem {
                                step: step.id.clone(),
                                item: name.clone(),
                            }),
                        }
                    }
                    step.label.clone().unwrap_or_else(|| "Details".to_owned())
                }
                StepKind::Capture { capture: capture_id } => {
                    match captures.get_mut(capture_id) {
                        Some(entry) => {
                            entry.referenced = true;
                            let requirement = &entry.requirement;
                            step.label.clone().unwrap_or_else(|| {
                                if requirement.kinds.len() == 1 {
                                    format!("Capture {:?}", requirement.kinds[0])
                                } else {
                                    format!("{} ({} options)", requirement.label, requirement.kinds.len())
                                }
                            })
                        }
                        None => {
                            errors.push(CompileError::UnknownCapture {
                                step: step.id.clone(),
                                capture: capture_id.clone(),
                            });
                            String::new()
                        }
                    }
                }
                StepKind::Review => step.label.clone().unwrap_or_else(|| "Review".to_owned()),
                StepKind::Submit => step.label.clone().unwrap_or_else(|| "Submit".to_owned()),
            };
            steps.push(CompiledStep { step: step.clone(), items, display_label });
        }

        // Any data item no step collects, and any capture requirement no step
        // uses, are dead configuration.
        for name in &data_item_order {
            if !data_items[name].reachable {
                errors.push(CompileError::UnreachableDataItem { name: name.clone() });
            }
        }
        for id in &capture_order {
            if !captures[id].referenced {
                errors.push(CompileError::UnreferencedCapture { id: id.clone() });
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(CompiledWorkflow {
            id: spec.id.clone(),
            label: spec.label.clone(),
            data_items,
            data_item_order,
            steps,
            captures,
            capture_order,
        })
    }

/// Validates one data item's constraints, appending any violations.
fn check_constraints(item: &DataItem, errors: &mut Vec<CompileError>) {
    let Constraints {
        min_length,
        max_length,
        options,
        min_age_years,
        max_age_years,
    } = &item.constraints;

    if let (Some(min), Some(max)) = (min_length, max_length) {
        if min > max {
            errors.push(CompileError::LengthBoundsInverted {
                name: item.name.clone(),
                min: *min,
                max: *max,
            });
        }
    }
    // Length bounds only mean something for free-text and choice fields, and
    // they are wrong on a pattern-typed item even when the bound happens to
    // agree with the pattern's own length.
    if (min_length.is_some() || max_length.is_some()) && !item.takes_text() {
        errors.push(CompileError::LengthBoundOnNonText {
            name: item.name.clone(),
            kind: item.kind,
        });
    }

    match (min_age_years, max_age_years) {
        (Some(min), Some(max)) if min > max => errors.push(CompileError::AgeBoundsInverted {
            name: item.name.clone(),
            min: *min,
            max: *max,
        }),
        (Some(_), _) | (_, Some(_)) if item.kind != DataKind::Date => {
            errors.push(CompileError::AgeBoundOnNonDate { name: item.name.clone() });
        }
        _ => {}
    }

    if item.kind == DataKind::Choice {
        match options {
            None => errors.push(CompileError::ChoiceWithoutOptions {
                name: item.name.clone(),
            }),
            Some(values) => {
                let mut seen = BTreeSet::new();
                for value in values {
                    if !seen.insert(value.as_str()) {
                        errors.push(CompileError::DuplicateOption {
                            name: item.name.clone(),
                            option: value.clone(),
                        });
                    }
                }
            }
        }
    }
}

/// Validates one capture requirement, appending any violations.
fn check_capture(capture: &crate::model::CaptureRequirement, errors: &mut Vec<CompileError>) {
    use crate::model::CapturePolicy;

    if capture.kinds.is_empty() {
        errors.push(CompileError::CaptureWithoutKinds { id: capture.id.clone() });
        return;
    }

    let mut seen = BTreeSet::new();
    for kind in &capture.kinds {
        if !seen.insert(kind) {
            errors.push(CompileError::DuplicateCaptureKind {
                id: capture.id.clone(),
                kind: *kind,
            });
        }
    }

    if let CapturePolicy::AtLeast(n) = capture.policy {
        if n == 0 {
            errors.push(CompileError::CapturePolicyVacuous { id: capture.id.clone() });
        }
        if usize::from(n) > capture.kinds.len() {
            errors.push(CompileError::CapturePolicyUnsatisfiable {
                id: capture.id.clone(),
                wanted: n,
                have: capture.kinds.len(),
            });
        }
    }
}

impl DataItem {
    /// Returns `true` when this kind is represented as a text field.
    fn takes_text(&self) -> bool {
        matches!(self.kind, DataKind::Text | DataKind::Choice)
    }
}
