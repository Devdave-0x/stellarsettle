// #315 — a shipment with zero total value must be rejected.

extern crate stellarsettle;
use soroban_sdk::{testutils::Address as _, token, vec, Address, Env, String};
use stellarsettle::*;

#[test]
#[should_panic(expected = "amount must be greater than zero")]
fn test_create_shipment_with_zero_total_value_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarSettleContract, ());
    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    let buyer = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token_id).mint(&buyer, &1_000_000);

    let client = StellarSettleContractClient::new(&env, &contract_id);
    client.init(&buyer);

    assert_eq!(stellarsettle::constants::MIN_SHIPMENT_AMOUNT, 1);
    let total_amount: i128 = 0;
    assert!(total_amount < stellarsettle::constants::MIN_SHIPMENT_AMOUNT);

    let milestones = vec![
        &env,
        Milestone {
            name: String::from_str(&env, "Delivered"),
            payment_percent: 100,
            proof_hash: String::from_str(&env, ""),
            status: MilestoneStatus::Pending,
            release_after_ledger: 0,
            proof_submitted_ledger: None,
            dispute_opened_ledger: None,
            deadline_ledger: 0,
            penalty_bps_per_ledger: 0,
        },
    ];
    let options = ShipmentOptions {
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
        milestone_splits: vec![&env],
        deadlines: vec![&env],
        dispute_timeout_seconds: 0,
        default_resolution: Resolution::Buyer,
        backup_arbiter: None,
        confirmation_cooldown_ledgers: None,
        arbiter_panel: vec![&env],
        grace_period_ledgers: 0,
        jurisdiction: None,
        quality_grades: vec![&env],
        milestone_quantities: vec![&env],
        retainage_bps: 0,
        warranty_bps: 0,
        warranty_ledgers: 0,
        inspector: None,
        inspected_milestones: vec![&env],
        proof_submitters: vec![&env],
        require_dual_attestation: false,
        fund_from_vault: false,
        milestone_suppliers: vec![&env],
        refund_recipient: None,
    };

    client.create_shipment(
        &String::from_str(&env, "ZERO-VALUE"),
        &vec![&env, buyer],
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &token_id,
        &total_amount,
        &milestones,
        &options,
    );
}
