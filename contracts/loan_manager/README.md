# loan_manager

Soroban lending pool. Depositors supply the debt asset; borrowers lock collateral and borrow against it; undercollateralised positions can be liquidated.

## Key functions

- Admin: `init`, `set_oracles`, `set_market`, `set_liq_params`, `set_interest_rate` (capped at `MAX_INTEREST_RATE_BPS`), `pause` / `unpause`, `seize_defaulted`.
- Liquidity: `deposit`, `withdraw`.
- Borrowing: `deposit_collateral`, `borrow`, `borrow_with_term`, `repay`, `withdraw_collateral`.
- Liquidation: `liquidate`, `health_factor`.
- Oracles: `record_price`, `preview_price`, `get_twap`, `twap_now` (primary and secondary oracle failover plus TWAP).
- Views: `get_position`, `get_depositor_balance`, `get_pool_info`, `get_depositors`, `get_liquidation_params`, `get_market_config`, `get_oracles`, `get_admin`, `get_interest_rate`, `is_paused_view`.

## Storage layout

Keys are defined in `src/storage.rs` (`DataKey`): admin, paused flag, oracle addresses, market config (collateral/debt assets and liquidation threshold), liquidation params, interest rate (bps), pool totals, per-user `Position` (collateral, debt), per-depositor balances and the depositor list, and TWAP price observations.

## Interaction with the escrow contract

None. `loan_manager` is standalone and does not call, or get called by, the main `stellarsettle` escrow contract; the two only share the workspace.

## Interest

The configured rate is stored and capped, but debt is currently not accrued over time (see `src/test_fz_interest.rs`).

## Tests

Unit tests live in `src/test_*.rs`, wired up from `src/test.rs`.
