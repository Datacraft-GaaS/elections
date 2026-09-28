//! Validates a collected submission against a [`CompiledWorkflow`].
//!
//! Kept separate from the UI so the same rules apply to the desktop client, the
//! Android bvr-kit, and any replay of a captured record. Validation is total: it
//! reports every violation rather than stopping at the first.

use chrono::Datelike;
use thiserror::Error;

use crate::compile::CompiledWorkflow;
use crate::model::{DataKind, StepKind};

/// A single violation found in a submission.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    /// A required item was absent or blank.
    #[error("data item {item:?} is required but was not provided")]
    MissingRequired {
        /// The offending item.
        item: String,
    },
    /// A required consent item was not affirmatively ticked.
    #[error("consent for data item {item:?} was not given")]
    ConsentNotGiven {
        /// The offending item.
        item: String,
    },
    /// The value did not satisfy the contract pattern for its kind.
    #[error("data item {item:?} failed the {kind:?} pattern check")]
    PatternMismatch {
        /// The offending item.
        item: String,
        /// The item's kind.
        kind: DataKind,
    },
    /// The value was shorter than `min_length`.
    #[error("data item {item:?} is shorter than the {min}-character minimum")]
    TooShort {
        /// The offending item.
        item: String,
        /// The required minimum.
        min: usize,
    },
    /// The value was longer than `max_length`.
    #[error("data item {item:?} is longer than the {max}-character maximum")]
    TooLong {
        /// The offending item.
        item: String,
        /// The allowed maximum.
        max: usize,
    },
    /// The value was not one of the permitted options.
    #[error("data item {item:?} has a value outside its permitted options")]
    NotAnOption {
        /// The offending item.
        item: String,
    },
    /// The date was not a valid ISO 8601 calendar date.
    #[error("data item {item:?} is not a valid ISO 8601 date")]
    NotADate {
        /// The offending item.
        item: String,
    },
    /// The date of birth implies an age outside the configured bounds.
    #[error("data item {item:?} implies age {age}, outside the {min}..={max} bounds")]
    AgeOutOfRange {
        /// The offending item.
        item: String,
        /// The implied whole-year age.
        age: u16,
        /// The lower bound.
        min: u16,
        /// The upper bound.
        max: u16,
    },
    /// The submission carried a field the workflow does not declare.
    #[error("submission contains undeclared data item {item:?}")]
    UndeclaredItem {
        /// The offending field.
        item: String,
    },
    /// A required capture step was not satisfied.
    #[error("capture step {step:?} needs {needed} of {{{available}}}, only {got} captured")]
    CaptureUnsatisfied {
        /// The step whose requirement failed.
        step: String,
        /// The requirement id.
        capture: String,
        /// How many captures the policy demands.
        needed: String,
        /// The kinds the requirement accepts, formatted.
        available: String,
        /// The kinds actually captured.
        got: String,
    },
}

/// Outcome of validating a submission.
#[derive(Debug, PartialEq, Eq)]
pub struct SubmissionError {
    /// Every violation found, in item-declaration order.
    pub violations: Vec<ValidationError>,
}

impl SubmissionError {
    /// Returns `true` when nothing was wrong.
    pub fn is_ok(&self) -> bool {
        self.violations.is_empty()
    }
}

/// A submission is a map of data-item name to raw string value, plus the
/// capture kinds reported as acquired.
///
/// Values are strings deliberately: the client collects text from widgets and
/// the server owns typed parsing, so validation must not depend on a local
/// type that could drift from the contract.
pub type Submission = serde_json::Map<String, serde_json::Value>;

/// Validates `submission` against `workflow`.
///
/// The `captured` slice lists the capture kinds the operator reported
/// acquiring. Each step's named capture requirement is then checked against its
/// policy, so a workflow demanding every listed capture and one demanding only
/// a subset are both expressible without extra steps.
pub fn validate_submission(
    workflow: &CompiledWorkflow,
    submission: &Submission,
    captured: &[crate::model::CaptureKind],
) -> SubmissionError {
    let mut violations = Vec::new();

    // Reject anything the workflow did not declare, so a stale or tampered
    // payload cannot smuggle extra fields into the submit call.
    for key in submission.keys() {
        if !workflow.data_items.contains_key(key) {
            violations.push(ValidationError::UndeclaredItem { item: key.clone() });
        }
    }

    for name in &workflow.data_item_order {
        let Some(entry) = workflow.data_item(name) else {
            continue;
        };
        let item = &entry.item;
        let raw = submission.get(name);

        let Some(value) = raw else {
            if item.required {
                violations.push(ValidationError::MissingRequired { item: name.clone() });
            }
            continue;
        };

        // Consent is boolean-valued and must be affirmatively true.
        if item.kind == DataKind::Consent {
            if value.as_bool() != Some(true) {
                if item.required {
                    violations.push(ValidationError::ConsentNotGiven { item: name.clone() });
                }
            }
            continue;
        }

        // A present-but-null or empty value is absent as far as the form is
        // concerned, so required-ness applies uniformly.
        let Some(text) = value.as_str() else {
            if item.required {
                violations.push(ValidationError::MissingRequired { item: name.clone() });
            }
            continue;
        };
        if text.trim().is_empty() {
            if item.required {
                violations.push(ValidationError::MissingRequired { item: name.clone() });
            }
            continue;
        }

        check_length(item.name.as_str(), text, item, &mut violations);
        check_pattern(item.name.as_str(), item.kind, text, &mut violations);
        check_options(item.name.as_str(), text, item, &mut violations);
        check_age(item.name.as_str(), text, item, &mut violations);
    }

    for step in &workflow.steps {
        if let StepKind::Capture { capture: capture_id } = &step.step.kind {
            let Some(entry) = workflow.capture(capture_id) else {
                continue;
            };
            let requirement = &entry.requirement;
            if requirement.optional || requirement.is_satisfied_by(captured) {
                continue;
            }
            violations.push(ValidationError::CaptureUnsatisfied {
                step: step.step.id.clone(),
                capture: capture_id.clone(),
                needed: describe_policy(requirement.policy),
                available: format_kinds(&requirement.kinds),
                got: format_kinds(captured),
            });
        }
    }

    SubmissionError { violations }
}

/// Human-readable form of a capture policy, for error messages.
fn describe_policy(policy: crate::model::CapturePolicy) -> String {
    use crate::model::CapturePolicy;
    match policy {
        CapturePolicy::AllOf => "all".to_owned(),
        CapturePolicy::AnyOf => "any".to_owned(),
        CapturePolicy::AtLeast(n) => format!("at least {n}"),
    }
}

/// Renders capture kinds as a stable, sorted, comma-separated list.
///
/// Uses each kind's serialized form rather than its `Debug` form, so a message
/// names the kind the way it appears in the workflow document (`palm_vein`)
/// instead of the Rust spelling (`PalmVein`).
fn format_kinds(kinds: &[crate::model::CaptureKind]) -> String {
    let mut names: Vec<String> = kinds
        .iter()
        .filter_map(|k| serde_json::to_value(k).ok())
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    names.sort();
    if names.is_empty() { "none".to_owned() } else { names.join(", ") }
}

fn check_length(
    name: &str,
    text: &str,
    item: &crate::model::DataItem,
    out: &mut Vec<ValidationError>,
) {
    let len = text.chars().count();
    if let Some(min) = item.constraints.min_length {
        if len < min {
            out.push(ValidationError::TooShort { item: name.to_owned(), min });
        }
    }
    if let Some(max) = item.constraints.max_length {
        if len > max {
            out.push(ValidationError::TooLong { item: name.to_owned(), max });
        }
    }
}

fn check_pattern(
    name: &str,
    kind: DataKind,
    text: &str,
    out: &mut Vec<ValidationError>,
) {
    use crate::model::DataKind as K;
    let field = match kind {
        K::NationalId => "national_id",
        K::Phone => "phone",
        K::StationCode => "polling_station_code",
        // Text, date, and choice are not pattern-validated here; the date kind
        // is checked by check_age, and choice by check_options.
        K::Text | K::Date | K::Choice | K::Consent => return,
    };
    if !eel_spec_helpers_rs::matches_bvr_pattern(field, text) {
        out.push(ValidationError::PatternMismatch { item: name.to_owned(), kind });
    }
}

fn check_options(
    name: &str,
    text: &str,
    item: &crate::model::DataItem,
    out: &mut Vec<ValidationError>,
) {
    if item.kind != DataKind::Choice {
        return;
    }
    if let Some(options) = &item.constraints.options {
        if !options.iter().any(|o| o == text) {
            out.push(ValidationError::NotAnOption { item: name.to_owned() });
        }
    }
}

/// Validates a date item.
///
/// The value must always parse as a real ISO 8601 calendar date, whatever the
/// age bounds say -- an earlier version returned early when no bounds were
/// declared and so accepted values like `1990-13-45`. Age checks then apply
/// individually, so a workflow declaring only `min_age_years` is enforced on
/// that bound alone.
fn check_age(
    name: &str,
    text: &str,
    item: &crate::model::DataItem,
    out: &mut Vec<ValidationError>,
) {
    use crate::model::DataKind as K;
    if item.kind != K::Date {
        return;
    }

    let Ok(date) = text.parse::<chrono::NaiveDate>() else {
        out.push(ValidationError::NotADate { item: name.to_owned() });
        return;
    };

    if item.constraints.min_age_years.is_none() && item.constraints.max_age_years.is_none() {
        return;
    }

    // Whole years elapsed, truncating rather than rounding, so an age band
    // admits someone on the day before their birthday and excludes them the
    // day after -- the conservative reading for a minimum-age rule.
    let today = chrono::Utc::now().date_naive();
    let Some(years) = age_in_whole_years(date, today) else {
        out.push(ValidationError::NotADate { item: name.to_owned() });
        return;
    };

    let min_age = item.constraints.min_age_years.unwrap_or(0);
    let max_age = item.constraints.max_age_years.unwrap_or(u16::MAX);
    if years < min_age || years > max_age {
        out.push(ValidationError::AgeOutOfRange {
            item: name.to_owned(),
            age: years,
            min: min_age,
            max: max_age,
        });
    }
}

/// Whole years from `birth` to `today`, or `None` if `birth` is in the future.
fn age_in_whole_years(birth: chrono::NaiveDate, today: chrono::NaiveDate) -> Option<u16> {
    let mut years = today.year() - birth.year();
    if (today.month(), today.day()) < (birth.month(), birth.day()) {
        years -= 1;
    }
    u16::try_from(years).ok().filter(|y| *y > 0 || birth <= today)
}
