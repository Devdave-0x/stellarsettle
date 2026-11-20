extern crate stellarsettle;
use soroban_sdk::{testutils::Address as _, token, vec, Address, Env, String, Symbol};
use stellarsettle::*;

struct Setup {
    env: Env,
    client: StellarSettleContractClient<'static>,
    arbiter: Address,
    shipment_id: String,
}

fn milestone(env: &Env, name: &str, payment_percent: u32) -> Milestone {
    Milestone {
        name: String::from_str(env, name),
        payment_percent,
        proof_hash: String::from_str(env, ""),
        status: MilestoneStatus::Pending,
        release_after_ledger: 0,
        proof_submitted_ledger: None,
        dispute_opened_ledger: None,
        deadline_ledger: 0,
        penalty_bps_per_ledger: 0,
    }
}

fn options(env: &Env) -> ShipmentOptions {
    ShipmentOptions {
        response_deadline: 0,
        penalty_bps: 0,
        milestone_mode: MilestoneMode::Parallel,
        holdback_ledgers: 0,
        dispute_cooldown_ledgers: 0,
        late_penalty_bps_per_ledger: 0,
        auto_confirm_ledgers: 0,
        dispute_bond_amount: 0,
        dispute_bond_bps: 0,
        arbiter_fee_bps: 0,
        logistics_fee_bps: 0,
        supplier_collateral: 0,
        expires_at_ledger: None,
        metadata_hash: None,
        referrer: None,
        buyer_cancel_fee_bps: 0,
        early_bonus_pool: 0,
        review_window_ledgers: None,
        milestone_splits: vec![env],
        deadlines: vec![env],
        dispute_timeout_seconds: 0,
        default_resolution: Resolution::Buyer,
        backup_arbiter: None,
        confirmation_cooldown_ledgers: None,
        arbiter_panel: vec![env],
        grace_period_ledgers: 0,
        jurisdiction: None,
        quality_grades: vec![env],
        milestone_quantities: vec![env],
        retainage_bps: 0,
        warranty_bps: 0,
        warranty_ledgers: 0,
        inspector: None,
        inspected_milestones: vec![env],
        proof_submitters: vec![env],
        require_dual_attestation: false,
        fund_from_vault: false,
        milestone_suppliers: vec![env],
        refund_recipient: None,
    }
}

fn setup() -> Setup {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarSettleContract, ());
    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    let buyer = Address::generate(&env);
    let supplier = Address::generate(&env);
    let logistics = Address::generate(&env);
    let arbiter = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token_id).mint(&buyer, &10_000_000_000);

    let client = StellarSettleContractClient::new(&env, &contract_id);
    client.init(&buyer);

    let shipment_id = String::from_str(&env, "SHIP-ARB-AUTH");
    client.create_shipment(
        &shipment_id,
        &vec![&env, buyer.clone()],
        &supplier,
        &logistics,
        &arbiter,
        &token_id,
        &1_000_000_000,
        &vec![
            &env,
            milestone(&env, "Goods Dispatched", 50),
            milestone(&env, "Delivered", 50),
        ],
        &options(&env),
    );
    client.submit_proof(
        &supplier,
        &shipment_id,
        &0,
        &String::from_str(&env, "ipfs://proof"),
        &Symbol::new(&env, "ipfs"),
    );

    Setup {
        env,
        client,
        arbiter,
        shipment_id,
    }
}

// #319: the arbiter must not be able to act on buyer-only endpoints.

#[test]
fn test_arbiter_cannot_confirm_milestone() {
    let t = setup();
    assert!(t
        .client
        .try_confirm_milestone(&t.arbiter, &t.shipment_id, &0)
        .is_err());
    assert_eq!(
        t.client.get_shipment(&t.shipment_id).milestones.get(0).unwrap().status,
        MilestoneStatus::ProofSubmitted
    );
}

#[test]
fn test_arbiter_cannot_cancel_shipment() {
    let t = setup();
    assert!(t
        .client
        .try_cancel_shipment(&t.arbiter, &t.shipment_id)
        .is_err());
    assert_eq!(
        t.client.get_shipment(&t.shipment_id).status,
        ShipmentStatus::Active
    );
}

#[test]
fn test_arbiter_cannot_raise_dispute() {
    let t = setup();
    assert!(t
        .client
        .try_raise_dispute(&t.arbiter, &t.shipment_id, &0)
        .is_err());
}

#[test]
fn test_arbiter_cannot_propose_amendment() {
    let t = setup();
    let name = String::from_str(&t.env, "Changed");
    assert!(t
        .client
        .try_propose_amendment(&t.arbiter, &t.shipment_id, &1, &50, &name)
        .is_err());
}
