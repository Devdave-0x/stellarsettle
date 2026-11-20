#![cfg(test)]

extern crate std;

// #330: Partial repayments in instalments. The loan manager has no interest
// model, so accrued interest is expected to be zero: after each payment the
// outstanding debt is exactly `previous_debt - payment`, even across time.

use crate::test::test_common::{advance_time, open_position, seed_liquidity, setup};

/// Expected debt after a payment: principal + accrued interest (0) - payment.
fn expected_debt(previous: i128, payment: i128) -> i128 {
    let accrued_interest = 0;
    previous + accrued_interest - payment
}

fn run_instalments(principal: i128, payments: &[i128], secs_between: u64) {
    let s = setup();
    seed_liquidity(&s, &[&s.d1], &[100_000]);
    open_position(&s, &s.alice, principal * 2, principal);

    let mut debt = principal;
    for p in payments {
        advance_time(&s.env, secs_between);
        s.client().repay(&s.alice, p);
        debt = expected_debt(debt, *p);
        assert_eq!(s.client().get_position(&s.alice).debt, debt);
        assert_eq!(s.client().get_pool_info().1, debt);
    }
}

#[test]
fn test_partial_repay_two_instalments() {
    run_instalments(1_000, &[400, 600], 0);
}

#[test]
fn test_partial_repay_three_instalments_over_time() {
    run_instalments(3_000, &[1_000, 500, 1_500], 86_400);
}

#[test]
fn test_partial_repay_leaves_remaining_balance() {
    run_instalments(5_000, &[1, 999, 2_000], 3_600);
}
