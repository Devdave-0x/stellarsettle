#![cfg(test)]

extern crate std;

#[path = "fuzz_tests.rs"]
mod fuzz_tests;
#[path = "test_common.rs"]
pub mod test_common;
#[path = "test_flash_loan_guard.rs"]
mod test_flash_loan_guard;
#[path = "test_fz_interest.rs"]
mod test_fz_interest;
#[path = "test_lending.rs"]
mod test_lending;
#[path = "test_liquidation.rs"]
mod test_liquidation;
#[path = "test_loan_terms.rs"]
mod test_loan_terms;
#[path = "test_oracle.rs"]
mod test_oracle;
#[path = "test_partial_repay.rs"]
mod test_partial_repay;
