#![cfg(test)]
// #325 — create_shipment rejects a fresh token that was never added to the allowlist.

extern crate std;

use super::*;
use crate::test_common::{build_milestones, default_options, setup, single_buyer_vec};
use soroban_sdk::{testutils::Address as _, token, String};

#[test]
#[should_panic(expected = "token is not in the approved whitelist")]
fn test_create_shipment_rejects_fresh_non_allowlisted_token() {
    let t = setup();
    let client = StellarSettleContractClient::new(&t.env, &t.contract_id);

    // Only the harness token is allowlisted, in this test's own env.
    client.add_allowed_token(&t.token_id);

    // Fresh mock token, deliberately NOT added to the allowlist.
    let fresh_token = t
        .env
        .register_stellar_asset_contract_v2(Address::generate(&t.env))
        .address();
    token::StellarAssetClient::new(&t.env, &fresh_token).mint(&t.buyer, &10_000_000_000);

    assert_eq!(
        client.get_allowed_tokens(),
        soroban_sdk::vec![&t.env, t.token_id.clone()]
    );

    client.create_shipment(
        &String::from_str(&t.env, "non-allowlisted-token"),
        &single_buyer_vec(&t.env, &t.buyer),
        &t.supplier,
        &t.logistics,
        &t.arbiter,
        &fresh_token,
        &1_000_000_000i128,
        &build_milestones(&t.env),
        &default_options(&t.env),
    );
}
