# Security notes

## Flash loans (`loan_manager`)

Flash loans are not an intended feature. `borrow` requires the caller's
collateral to already be deposited and checks the resulting health factor
before any tokens leave the pool, so a borrow cannot be funded by the same
borrowed amount and repaid within one transaction without collateral.
Borrowing and repaying in the same transaction leaves pool accounting
unchanged. Covered by `contracts/loan_manager/src/test_flash_loan_guard.rs`.
