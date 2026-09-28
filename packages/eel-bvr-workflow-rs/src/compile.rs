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
    /// True when the workflow acquires at least one biometric modality.
    pub has_biometric_capture: bool,
}

impl CompiledWorkflow {
    /// Returns the compiled data item named `name`, if present.
    pub fn data_item(&self, name: &str) -> Option<&CompiledDataItem> {
        self.data_items.get(name)
    }
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

    // Walk steps, resolving data-entry references and tracking reachability.
    let mut steps: Vec<CompiledStep> = Vec::new();
    let mut seen_step_ids: BTreeSet<&str> = BTreeSet::new();
    let mut has_biometric_capture = false;
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
            StepKind::BiometricCapture { modality, .. } => {
                has_biometric_capture = true;
                step.label
                    .clone()
                    .unwrap_or_else(|| format!("Capture {modality:?}"))
            }
            StepKind::Review => step.label.clone().unwrap_or_else(|| "Review".to_owned()),
            StepKind::Submit => step.label.clone().unwrap_or_else(|| "Submit".to_owned()),
        };
        steps.push(CompiledStep { step: step.clone(), items, display_label });
    }

    // Any data item no step collects is dead weight in the form.
    for name in &data_item_order {
        if !data_items[name].reachable {
            errors.push(CompileError::UnreachableDataItem { name: name.clone() });
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
        has_biometric_capture,
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

impl DataItem {
    /// Returns `true` when this kind is represented as a text field.
    fn takes_text(&self) -> bool {
        matches!(self.kind, DataKind::Text | DataKind::Choice)
    }
}
