// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use budget::domain::{Budget, BudgetLine, PaymentVoucher};
use uuid::Uuid;

#[test]
fn budget_starts_as_draft() {
    let b = Budget::new(Uuid::now_v7(), "2024/2025", 10_000_000);
    assert!(matches!(b.status, budget::domain::BudgetStatus::Draft));
}

#[test]
fn budget_approve_transitions_to_approved() {
    let mut b = Budget::new(Uuid::now_v7(), "2024/2025", 10_000_000);
    let approver = Uuid::now_v7();
    b.approve(approver);
    assert!(matches!(b.status, budget::domain::BudgetStatus::Approved));
    assert_eq!(b.approved_by, Some(approver));
}

#[test]
fn voucher_sod_violation_rejected() {
    let user = Uuid::now_v7();
    let mut v = PaymentVoucher::new(Uuid::now_v7(), 500_000, user);
    let result = v.approve(user); // same user: SoD violation
    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("segregation-of-duties"), "unexpected: {msg}");
}

#[test]
fn voucher_approve_with_different_user_succeeds() {
    let initiator = Uuid::now_v7();
    let approver = Uuid::now_v7();
    let mut v = PaymentVoucher::new(Uuid::now_v7(), 500_000, initiator);
    v.approve(approver).expect("should approve");
    assert!(matches!(v.status, budget::domain::VoucherStatus::Approved));
}
