#![cfg(test)]
// #326 / #327 — admin-gated allowed-token list management.

extern crate std;

use super::*;
use crate::test_common::{create_standard_shipment, setup};
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    xdr::{ScErrorCode, ScErrorType},
    IntoVal, String,
};

#[test]
fn test_add_use_remove_allowed_token_cycle() {
    let t = setup();
    let client = StellarSettleContractClient::new(&t.env, &t.contract_id);

    client.add_allowed_token(&t.token_id);
    assert!(client.get_allowed_tokens().contains(&t.token_id));

    create_standard_shipment(
        &client,
        &t.env,
        &String::from_str(&t.env, "ALLOW-1"),
        &t.buyer,
        &t.supplier,
        &t.logistics,
        &t.arbiter,
        &t.token_id,
        1_000_000,
    );

    // Keep a different token on the list so the whitelist stays enforced.
    client.add_allowed_token(&Address::generate(&t.env));
    client.remove_allowed_token(&t.token_id);
    assert!(!client.get_allowed_tokens().contains(&t.token_id));

    let result = client.try_create_shipment(
        &String::from_str(&t.env, "ALLOW-2"),
        &crate::test_common::single_buyer_vec(&t.env, &t.buyer),
        &t.supplier,
        &t.logistics,
        &t.arbiter,
        &t.token_id,
        &1_000_000,
        &crate::test_common::build_milestones(&t.env),
        &crate::test_common::default_options(&t.env),
    );
    assert!(result.is_err());
}

#[test]
fn test_non_admin_cannot_add_allowed_token() {
    let t = setup();
    let client = StellarSettleContractClient::new(&t.env, &t.contract_id);
    let token = Address::generate(&t.env);

    // Only the non-admin supplier signs the call.
    t.env.mock_auths(&[MockAuth {
        address: &t.supplier,
        invoke: &MockAuthInvoke {
            contract: &t.contract_id,
            fn_name: "add_allowed_token",
            args: (token.clone(),).into_val(&t.env),
            sub_invokes: &[],
        },
    }]);

    let result = client.try_add_allowed_token(&token);
    assert_eq!(
        result.err(),
        Some(Ok(soroban_sdk::Error::from_type_and_code(
            ScErrorType::Auth,
            ScErrorCode::InvalidAction
        )))
    );
    assert!(!client.get_allowed_tokens().contains(&token));
}
