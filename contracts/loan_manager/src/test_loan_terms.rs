#![cfg(test)]

extern crate std;

// Interest-rate cap, maximum loan duration, and default/collateral seizure.

use crate::constants::{MAX_INTEREST_RATE_BPS, MAX_LOAN_DURATION_SECS};
use crate::test::test_common::{advance_time, seed_liquidity, setup};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address};

#[test]
fn test_interest_rate_at_cap_succeeds() {
    let s = setup();
    s.client()
        .set_interest_rate(&s.admin, &MAX_INTEREST_RATE_BPS);
    assert_eq!(s.client().get_interest_rate(), MAX_INTEREST_RATE_BPS);
}

#[test]
#[should_panic(expected = "interest rate above cap")]
fn test_interest_rate_above_cap_rejected() {
    let s = setup();
    s.client()
        .set_interest_rate(&s.admin, &(MAX_INTEREST_RATE_BPS + 1));
}

#[test]
fn test_borrow_at_max_duration_succeeds() {
    let s = setup();
    seed_liquidity(&s, &[&s.d1], &[10_000]);
    s.client().deposit_collateral(&s.alice, &4_000);
    s.client()
        .borrow_with_term(&s.alice, &1_000, &MAX_LOAN_DURATION_SECS);
    assert_eq!(s.client().get_position(&s.alice).debt, 1_000);
}

#[test]
#[should_panic(expected = "invalid loan duration")]
fn test_borrow_beyond_max_duration_rejected() {
    let s = setup();
    seed_liquidity(&s, &[&s.d1], &[10_000]);
    s.client().deposit_collateral(&s.alice, &4_000);
    s.client()
        .borrow_with_term(&s.alice, &1_000, &(MAX_LOAN_DURATION_SECS + 1));
}

#[test]
fn test_default_seizes_collateral_to_treasury() {
    let s = setup();
    seed_liquidity(&s, &[&s.d1], &[10_000]);
    s.client().deposit_collateral(&s.alice, &4_000);
    s.client().borrow_with_term(&s.alice, &1_000, &86_400);

    advance_time(&s.env, 86_401);
    let treasury = Address::generate(&s.env);
    let seized = s.client().seize_defaulted(&s.admin, &s.alice, &treasury);

    assert_eq!(seized, 4_000);
    assert_eq!(
        token::Client::new(&s.env, &s.coll_id).balance(&treasury),
        4_000
    );
    let pos = s.client().get_position(&s.alice);
    assert_eq!((pos.collateral, pos.debt), (0, 0));
    let (_, total_borrowed, _) = s.client().get_pool_info();
    assert_eq!(total_borrowed, 0);
}

#[test]
#[should_panic(expected = "loan not yet due")]
fn test_seize_before_due_date_rejected() {
    let s = setup();
    seed_liquidity(&s, &[&s.d1], &[10_000]);
    s.client().deposit_collateral(&s.alice, &4_000);
    s.client().borrow_with_term(&s.alice, &1_000, &86_400);

    let treasury = Address::generate(&s.env);
    s.client().seize_defaulted(&s.admin, &s.alice, &treasury);
}
