use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetStatus {
    Draft,
    Approved,
    Active,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcurementMethod {
    OpenTender,
    RestrictedTender,
    DirectProcurement,
    RequestForQuotation,
    LowValueProcurement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcurementStatus {
    Pending,
    Awarded,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    Active,
    Completed,
    Terminated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoucherStatus {
    Pending,
    Approved,
    CounterApproved,
    Paid,
    Rejected,
}

// ---------------------------------------------------------------------------
// Aggregates
// ---------------------------------------------------------------------------

/// Top-level budget for a financial year tied to an election.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub id: Uuid,
    pub election_id: Uuid,
    pub financial_year: String,
    /// Total budget amount in KES (cents).
    pub total_kes: i64,
    pub approved_by: Option<Uuid>,
    pub status: BudgetStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Budget {
    pub fn new(election_id: Uuid, financial_year: impl Into<String>, total_kes: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            election_id,
            financial_year: financial_year.into(),
            total_kes,
            approved_by: None,
            status: BudgetStatus::Draft,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn approve(&mut self, approver_id: Uuid) {
        self.approved_by = Some(approver_id);
        self.status = BudgetStatus::Approved;
        self.updated_at = Utc::now();
    }
}

/// A programme line within a budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLine {
    pub id: Uuid,
    pub budget_id: Uuid,
    pub programme_code: String,
    pub description: String,
    /// Allocated amount in KES (cents).
    pub allocated_kes: i64,
    /// Committed (purchase-order raised) amount.
    pub committed_kes: i64,
    /// Actually spent amount.
    pub spent_kes: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BudgetLine {
    pub fn new(
        budget_id: Uuid,
        programme_code: impl Into<String>,
        description: impl Into<String>,
        allocated_kes: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            budget_id,
            programme_code: programme_code.into(),
            description: description.into(),
            allocated_kes,
            committed_kes: 0,
            spent_kes: 0,
            created_at: now,
            updated_at: now,
        }
    }
}

/// PPRA-compliant procurement record against a budget line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcurementRecord {
    pub id: Uuid,
    pub budget_line_id: Uuid,
    pub vendor_id: Uuid,
    pub ppra_ref: String,
    pub amount_kes: i64,
    pub procurement_method: ProcurementMethod,
    pub status: ProcurementStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProcurementRecord {
    pub fn new(
        budget_line_id: Uuid,
        vendor_id: Uuid,
        ppra_ref: impl Into<String>,
        amount_kes: i64,
        procurement_method: ProcurementMethod,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            budget_line_id,
            vendor_id,
            ppra_ref: ppra_ref.into(),
            amount_kes,
            procurement_method,
            status: ProcurementStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Contract issued after procurement award.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: Uuid,
    pub procurement_id: Uuid,
    pub contract_ref: String,
    pub vendor_id: Uuid,
    pub amount_kes: i64,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub status: ContractStatus,
    pub created_at: DateTime<Utc>,
}

impl Contract {
    pub fn new(
        procurement_id: Uuid,
        contract_ref: impl Into<String>,
        vendor_id: Uuid,
        amount_kes: i64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            procurement_id,
            contract_ref: contract_ref.into(),
            vendor_id,
            amount_kes,
            start_date,
            end_date,
            status: ContractStatus::Active,
            created_at: Utc::now(),
        }
    }
}

/// Payment voucher with four-eyes SoD control.
///
/// Invariant: `approved_by != initiated_by` (enforced by [`PaymentVoucher::approve`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentVoucher {
    pub id: Uuid,
    pub procurement_id: Uuid,
    pub amount_kes: i64,
    pub initiated_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub counter_approved_by: Option<Uuid>,
    pub ifmis_ref: Option<String>,
    pub status: VoucherStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PaymentVoucher {
    pub fn new(procurement_id: Uuid, amount_kes: i64, initiated_by: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            procurement_id,
            amount_kes,
            initiated_by,
            approved_by: None,
            counter_approved_by: None,
            ifmis_ref: None,
            status: VoucherStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    /// First-level approval. Enforces SoD: approver must differ from initiator.
    pub fn approve(&mut self, approver_id: Uuid) -> Result<(), DomainError> {
        if approver_id == self.initiated_by {
            return Err(DomainError::SodViolation {
                detail: "approver must differ from initiator".into(),
            });
        }
        self.approved_by = Some(approver_id);
        self.status = VoucherStatus::Approved;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Second-level (counter) approval.
    pub fn counter_approve(&mut self, counter_approver_id: Uuid) -> Result<(), DomainError> {
        if self.status != VoucherStatus::Approved {
            return Err(DomainError::InvalidTransition {
                detail: "voucher must be approved before counter-approval".into(),
            });
        }
        self.counter_approved_by = Some(counter_approver_id);
        self.status = VoucherStatus::CounterApproved;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark as paid with IFMIS reference.
    pub fn mark_paid(&mut self, ifmis_ref: impl Into<String>) -> Result<(), DomainError> {
        if self.status != VoucherStatus::CounterApproved {
            return Err(DomainError::InvalidTransition {
                detail: "voucher must be counter-approved before payment".into(),
            });
        }
        self.ifmis_ref = Some(ifmis_ref.into());
        self.status = VoucherStatus::Paid;
        self.updated_at = Utc::now();
        Ok(())
    }
}

/// Controller of Budget (CoB) report for a reporting period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoBReport {
    pub id: Uuid,
    pub budget_id: Uuid,
    pub reporting_period: String,
    pub total_committed: i64,
    pub total_spent: i64,
    pub variance: i64,
    pub submitted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl CoBReport {
    pub fn new(
        budget_id: Uuid,
        reporting_period: impl Into<String>,
        total_committed: i64,
        total_spent: i64,
        variance: i64,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            budget_id,
            reporting_period: reporting_period.into(),
            total_committed,
            total_spent,
            variance,
            submitted_at: None,
            created_at: Utc::now(),
        }
    }
}

/// Audit package submitted to the Office of the Auditor General.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPackage {
    pub id: Uuid,
    pub budget_id: Uuid,
    pub oag_ref: String,
    pub period_from: DateTime<Utc>,
    pub period_to: DateTime<Utc>,
    /// UUIDs of supporting documents / vouchers included in the package.
    pub items: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl AuditPackage {
    pub fn new(
        budget_id: Uuid,
        oag_ref: impl Into<String>,
        period_from: DateTime<Utc>,
        period_to: DateTime<Utc>,
        items: Vec<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            budget_id,
            oag_ref: oag_ref.into(),
            period_from,
            period_to,
            items,
            created_at: Utc::now(),
        }
    }
}
