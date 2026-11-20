#![cfg(test)]

use super::*;
use crate::test_common::{build_milestones, create_standard_shipment, default_options, setup, single_buyer_vec};
use soroban_sdk::{token, Env, String, Symbol};

fn ipfs(env: &Env) -> Symbol {
    Symbol::new(env, "ipfs")
}

// #317: In Sequential mode, proof for milestone 3 (index 2) cannot be submitted
// (and so cannot be confirmed) before milestone 2 (index 1) is confirmed.
// Error: panics with "MilestoneOutOfOrder".
#[test]
#[should_panic(expected = "MilestoneOutOfOrder")]
fn test_out_of_order_milestone_confirmation_rejected() {
    let t = setup();
    let client = StellarSettleContractClient::new(&t.env, &t.contract_id);
    let id = String::from_str(&t.env, "SEQ-317");
    let mut options = default_options(&t.env);
    options.milestone_mode = MilestoneMode::Sequential;
    client.create_shipment(
        &id,
        &single_buyer_vec(&t.env, &t.buyer),
        &t.supplier,
        &t.logistics,
        &t.arbiter,
        &t.token_id,
        &1_000_000,
        &build_milestones(&t.env),
        &options,
    );

    client.submit_proof(&t.supplier, &id, &0, &String::from_str(&t.env, "ipfs://m1"), &ipfs(&t.env));
    client.confirm_milestone(&t.buyer, &id, &0);

    // Skip milestone 2 and go straight to milestone 3.
    client.submit_proof(&t.supplier, &id, &2, &String::from_str(&t.env, "ipfs://m3"), &ipfs(&t.env));
}

// #316: A second (different) proof for an already-confirmed milestone is rejected
// and leaves the milestone state untouched.
#[test]
fn test_duplicate_proof_for_confirmed_milestone_rejected() {
    let t = setup();
    let client = StellarSettleContractClient::new(&t.env, &t.contract_id);
    let id = String::from_str(&t.env, "DUP-316");
    create_standard_shipment(
        &client, &t.env, &id, &t.buyer, &t.supplier, &t.logistics, &t.arbiter, &t.token_id, 1_000_000,
    );

    let first = String::from_str(&t.env, "ipfs://first");
    client.submit_proof(&t.supplier, &id, &0, &first, &ipfs(&t.env));
    client.confirm_milestone(&t.buyer, &id, &0);
    let before = client.get_shipment(&id);

    let result = client.try_submit_proof(
        &t.supplier,
        &id,
        &0,
        &String::from_str(&t.env, "ipfs://second"),
        &ipfs(&t.env),
    );
    assert!(result.is_err());

    let after = client.get_shipment(&id);
    let m = after.milestones.get(0).unwrap();
    assert_eq!(m.status, MilestoneStatus::Confirmed);
    assert_eq!(m.proof_hash, first);
    assert_eq!(after.released_amount, before.released_amount);
}

// #322: Arbiter rules fully for the buyer: supplier gets nothing for the disputed
// milestone and the buyer gets the disputed amount back.
#[test]
fn test_arbiter_rules_for_buyer_full_refund() {
    let t = setup();
    let client = StellarSettleContractClient::new(&t.env, &t.contract_id);
    let token_client = token::Client::new(&t.env, &t.token_id);
    let id = String::from_str(&t.env, "ARB-322");
    let amount: i128 = 1_000_000;
    let buyer_start = token_client.balance(&t.buyer);
    create_standard_shipment(
        &client, &t.env, &id, &t.buyer, &t.supplier, &t.logistics, &t.arbiter, &t.token_id, amount,
    );

    client.submit_proof(&t.supplier, &id, &0, &String::from_str(&t.env, "ipfs://p"), &ipfs(&t.env));
    client.raise_dispute(&t.buyer, &id, &0);
    client.resolve_dispute(&t.arbiter, &id, &0, &false, &None);

    // Supplier receives zero; disputed funds stay in escrow for the buyer.
    assert_eq!(token_client.balance(&t.supplier), 0);
    assert_eq!(client.get_shipment(&id).released_amount, 0);
    assert_eq!(client.get_escrow_balance(&id), amount);

    // Buyer reclaims the escrow, including the disputed milestone amount.
    client.cancel_shipment(&t.buyer, &id);
    assert_eq!(token_client.balance(&t.buyer), buyer_start);
    assert_eq!(token_client.balance(&t.supplier), 0);
}
