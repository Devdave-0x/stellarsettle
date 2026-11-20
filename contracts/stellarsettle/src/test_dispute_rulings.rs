// #323: arbiter ruling in favour of the supplier (full release).
// #324: split arbiter ruling (60% supplier / 40% buyer).
// #321: property test — total released never exceeds the escrowed amount.

#![cfg(test)]

extern crate std;

use super::test_common::*;
use super::*;
use proptest::prelude::*;
use soroban_sdk::{token, vec, Env, String, Symbol};

const TOTAL: i128 = 1_000_000;

fn submit(t: &TestSetup, client: &StellarSettleContractClient, id: &String, m: u32) {
    let proofs = ["ipfs://ruling-proof-0", "ipfs://ruling-proof-1", "ipfs://ruling-proof-2"];
    client.submit_proof(
        &t.supplier,
        id,
        &m,
        &String::from_str(&t.env, proofs[m as usize]),
        &Symbol::new(&t.env, "ipfs"),
    );
}

#[test]
fn test_arbiter_rules_for_supplier_full_release() {
    let t = setup();
    let client = StellarSettleContractClient::new(&t.env, &t.contract_id);
    let tok = token::Client::new(&t.env, &t.token_id);
    let id = String::from_str(&t.env, "RULE-SUP");
    create_standard_shipment(
        &client, &t.env, &id, &t.buyer, &t.supplier, &t.logistics, &t.arbiter, &t.token_id, TOTAL,
    );

    // Milestone 1 carries 50%.
    let payment = TOTAL * 50 / 100;
    submit(&t, &client, &id, 1);
    client.raise_dispute(&t.buyer, &id, &1);

    let escrow_before = client.get_escrow_balance(&id);
    let supplier_before = tok.balance(&t.supplier);

    client.resolve_dispute(&t.arbiter, &id, &1, &true, &None);

    assert_eq!(tok.balance(&t.supplier) - supplier_before, payment);
    assert_eq!(escrow_before - client.get_escrow_balance(&id), payment);
    assert_eq!(client.get_milestone(&id, &1).status, MilestoneStatus::Resolved);
}

#[test]
fn test_arbiter_split_ruling_60_40() {
    let t = setup();
    let client = StellarSettleContractClient::new(&t.env, &t.contract_id);
    let tok = token::Client::new(&t.env, &t.token_id);
    let id = String::from_str(&t.env, "RULE-SPLIT");
    create_standard_shipment(
        &client, &t.env, &id, &t.buyer, &t.supplier, &t.logistics, &t.arbiter, &t.token_id, TOTAL,
    );

    let milestone_amount = TOTAL * 50 / 100;
    let supplier_before = tok.balance(&t.supplier);
    let buyer_before = tok.balance(&t.buyer);

    // Buyer contests 40%; the uncontested 60% goes to the supplier and the
    // arbiter rules the contested 40% back to the buyer.
    submit(&t, &client, &id, 1);
    client.raise_partial_dispute(&t.buyer, &id, &1, &40);
    client.resolve_dispute(&t.arbiter, &id, &1, &false, &None);

    let supplier_got = tok.balance(&t.supplier) - supplier_before;
    let buyer_got = tok.balance(&t.buyer) - buyer_before;
    assert_eq!(supplier_got, milestone_amount * 60 / 100);
    assert_eq!(buyer_got, milestone_amount * 40 / 100);
    assert_eq!(supplier_got + buyer_got, milestone_amount);
    assert_eq!(client.get_milestone(&id, &1).status, MilestoneStatus::Resolved);
}

fn milestone(env: &Env, percent: u32) -> Milestone {
    Milestone {
        name: String::from_str(env, "M"),
        payment_percent: percent,
        proof_hash: String::from_str(env, ""),
        status: MilestoneStatus::Pending,
        release_after_ledger: 0,
        proof_submitted_ledger: None,
        dispute_opened_ledger: None,
        deadline_ledger: 0,
        penalty_bps_per_ledger: 0,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Random milestone splits and settlement paths: released funds must never
    /// exceed the escrow deposit, and the contract must hold the remainder.
    #[test]
    fn prop_released_never_exceeds_escrow(
        a in 1u32..=98,
        b_raw in 1u32..=98,
        actions in prop::collection::vec(0u8..4, 3),
        contested in 1u32..=99,
    ) {
        let b = 1 + b_raw % (99 - a);
        let c = 100 - a - b;

        let t = setup();
        let client = StellarSettleContractClient::new(&t.env, &t.contract_id);
        let tok = token::Client::new(&t.env, &t.token_id);
        let id = String::from_str(&t.env, "PROP-ESCROW");
        client.create_shipment(
            &id,
            &single_buyer_vec(&t.env, &t.buyer),
            &t.supplier,
            &t.logistics,
            &t.arbiter,
            &t.token_id,
            &TOTAL,
            &vec![&t.env, milestone(&t.env, a), milestone(&t.env, b), milestone(&t.env, c)],
            &default_options(&t.env),
        );

        for (i, action) in actions.iter().enumerate() {
            let m = i as u32;
            submit(&t, &client, &id, m);
            match action {
                0 => client.confirm_milestone(&t.buyer, &id, &m),
                1 => {
                    client.raise_dispute(&t.buyer, &id, &m);
                    client.resolve_dispute(&t.arbiter, &id, &m, &true, &None);
                }
                2 => {
                    client.raise_partial_dispute(&t.buyer, &id, &m, &contested);
                    client.resolve_dispute(&t.arbiter, &id, &m, &false, &None);
                }
                _ => {
                    client.raise_dispute(&t.buyer, &id, &m);
                    client.resolve_dispute(&t.arbiter, &id, &m, &false, &None);
                }
            }

            let released = client.get_shipment(&id).released_amount;
            prop_assert!(released >= 0 && released <= TOTAL);
            prop_assert_eq!(tok.balance(&t.contract_id), TOTAL - released);
        }
    }
}
