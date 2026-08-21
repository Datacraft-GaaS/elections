use rts_domain::aggregates::form34a::{Form34A, Form34AStatus};
use rts_domain::validators::{validate_ec_dat_10, validate_ec_dat_11};
use uuid::Uuid;

fn make_form(registered_voters: u32, votes_cast: u32, valid_votes: u32, rejected_votes: u32, spoilt_ballots: u32) -> Form34A {
    let mut form = Form34A::new(Uuid::new_v4());
    form.registered_voters = registered_voters; form.votes_cast = votes_cast;
    form.valid_votes = valid_votes; form.rejected_votes = rejected_votes;
    form.spoilt_ballots = spoilt_ballots; form.station_code = "TEST-001".to_string();
    form
}

#[test] fn ec_dat_10_happy_path() { assert!(validate_ec_dat_10(&make_form(700, 500, 490, 7, 3)).is_ok()); }
#[test] fn ec_dat_10_zero_results_allowed() { assert!(validate_ec_dat_10(&make_form(700, 0, 0, 0, 0)).is_ok()); }
#[test] fn ec_dat_10_arithmetic_mismatch() { assert!(validate_ec_dat_10(&make_form(700, 501, 490, 7, 3)).is_err()); }
#[test] fn ec_dat_10_zero_registered_fails() { assert!(validate_ec_dat_10(&make_form(0, 0, 0, 0, 0)).is_err()); }
#[test] fn ec_dat_11_happy_path() { assert!(validate_ec_dat_11(&make_form(700, 500, 490, 7, 3)).is_ok()); }
#[test] fn ec_dat_11_spoilt_exceeds_valid_allowed() { assert!(validate_ec_dat_11(&make_form(700, 700, 50, 50, 600)).is_ok()); }
#[test] fn ec_dat_11_rejected_plus_valid_exceeds_registered() { assert!(validate_ec_dat_11(&make_form(700, 700, 400, 400, 0)).is_err()); }
#[test] fn ec_dat_11_exact_boundary() { assert!(validate_ec_dat_11(&make_form(700, 700, 350, 350, 0)).is_ok()); }
#[test] fn ec_dat_11_one_over_boundary() { assert!(validate_ec_dat_11(&make_form(700, 701, 351, 350, 0)).is_err()); }
