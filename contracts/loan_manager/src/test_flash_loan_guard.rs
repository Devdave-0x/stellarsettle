#![cfg(test)]

extern crate std;

// Flash-loan guard (#334): borrowing requires collateral already posted, so
// an uncollateralised borrow-and-repay in one transaction is not possible.

use crate::test::test_common::{open_position, seed_liquidity, setup};

#[test]
#[should_panic(expected = "insufficient collateral")]
fn test_borrow_without_collateral_panics() {
    let s = setup();
    seed_liquidity(&s, &[&s.d1], &[10_000]);
    s.client().borrow(&s.alice, &1_000);
}

#[test]
fn test_borrow_and_repay_same_tx_leaves_pool_unchanged() {
    let s = setup();
    seed_liquidity(&s, &[&s.d1], &[10_000]);
    open_position(&s, &s.alice, 5_000, 1_000);
    s.client().repay(&s.alice, &1_000);

    let (td, tb, _) = s.client().get_pool_info();
    assert_eq!(td, 10_000);
    assert_eq!(tb, 0);
    assert_eq!(s.client().get_position(&s.alice).debt, 0);
}
