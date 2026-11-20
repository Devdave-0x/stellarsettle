#![cfg(test)]

extern crate std;

use super::*;
use crate::test_common::*;
use soroban_sdk::{testutils::Ledger as _, String};

fn new_shipment(t: &TestSetup, client: &StellarSettleContractClient, id: &str) -> String {
    let shipment_id = String::from_str(&t.env, id);
    create_standard_shipment(
        client,
        &t.env,
        &shipment_id,
        &t.buyer,
        &t.supplier,
        &t.logistics,
        &t.arbiter,
        &t.token_id,
        1_000_000_000,
    );
    shipment_id
}

// #320: disputes must be opened within the review window after proof submission.
#[test]
#[should_panic(expected = "dispute window closed")]
fn test_raise_dispute_after_window_expired_is_rejected() {
    let t = setup();
    let client = StellarSettleContractClient::new(&t.env, &t.contract_id);
    let shipment_id = String::from_str(&t.env, "SHIP-DISP-WINDOW");
    let mut opts = default_options(&t.env);
    opts.auto_confirm_ledgers = 100;
    client.create_shipment(
        &shipment_id,
        &single_buyer_vec(&t.env, &t.buyer),
        &t.supplier,
        &t.logistics,
        &t.arbiter,
        &t.token_id,
        &1_000_000_000,
        &build_milestones(&t.env),
        &opts,
    );

    t.env.ledger().set_sequence_number(10);
    client.submit_proof(
        &t.supplier,
        &shipment_id,
        &0,
        &String::from_str(&t.env, "ipfs://proof"),
        &Symbol::new(&t.env, "ipfs"),
    );

    t.env.ledger().set_sequence_number(10 + 100);
    client.raise_dispute(&t.buyer, &shipment_id, &0);
}

// #328: oversized milestone notes are rejected.
#[test]
#[should_panic(expected = "note too long")]
fn test_milestone_note_exceeding_max_length_is_rejected() {
    let t = setup();
    let client = StellarSettleContractClient::new(&t.env, &t.contract_id);
    let shipment_id = new_shipment(&t, &client, "SHIP-NOTE-LEN");

    let ok = String::from_bytes(&t.env, &[b'a'; 256]);
    client.add_milestone_note(&t.buyer, &shipment_id, &0, &ok);
    assert_eq!(client.get_milestone_notes(&shipment_id, &0).len(), 1);

    let too_long = String::from_bytes(&t.env, &[b'a'; 257]);
    client.add_milestone_note(&t.buyer, &shipment_id, &0, &too_long);
}

// #329: two conflicting amendments in the same ledger never overwrite the
// milestone; only matching terms from both parties are applied, in either order.
#[test]
fn test_concurrent_conflicting_amendments_do_not_overwrite() {
    for buyer_first in [true, false] {
        let t = setup();
        let client = StellarSettleContractClient::new(&t.env, &t.contract_id);
        let shipment_id = new_shipment(&t, &client, "SHIP-AMEND-RACE");
        let original = client.get_shipment(&shipment_id).milestones.get(0).unwrap();
        let name_a = String::from_str(&t.env, "Amendment A");
        let name_b = String::from_str(&t.env, "Amendment B");

        if buyer_first {
            client.propose_amendment(&t.buyer, &shipment_id, &0, &25, &name_a);
            client.propose_amendment(&t.supplier, &shipment_id, &0, &25, &name_b);
        } else {
            client.propose_amendment(&t.supplier, &shipment_id, &0, &25, &name_b);
            client.propose_amendment(&t.buyer, &shipment_id, &0, &25, &name_a);
        }

        let m = client.get_shipment(&shipment_id).milestones.get(0).unwrap();
        assert_eq!(m.name, original.name);
        assert_eq!(m.payment_percent, original.payment_percent);
        assert_eq!(client.get_amendment_log(&shipment_id, &0).len(), 0);

        // Once both parties agree on the same terms, that amendment applies.
        if buyer_first {
            client.propose_amendment(&t.buyer, &shipment_id, &0, &25, &name_b);
        } else {
            client.propose_amendment(&t.supplier, &shipment_id, &0, &25, &name_a);
        }
        let m = client.get_shipment(&shipment_id).milestones.get(0).unwrap();
        let expected = if buyer_first { &name_b } else { &name_a };
        assert_eq!(&m.name, expected);
        assert_eq!(client.get_amendment_log(&shipment_id, &0).len(), 1);
    }
}
