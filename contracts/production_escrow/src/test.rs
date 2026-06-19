#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, IntoVal, Symbol, Vec,
};

// ─── shared helpers ──────────────────────────────────────────────────────────

struct Setup {
    env: Env,
    client: ProductionEscrowContractClient<'static>,
    admin: Address,
    farmer: Address,
    investor1: Address,
    investor2: Address,
    campaign_id: u64,
    token: Address,
}

/// Creates a mock token, mints `amount` to `recipient`.
fn create_token<'a>(env: &'a Env, admin: &Address) -> (Address, StellarAssetClient<'a>) {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token_id.address());
    (token_id.address(), sac)
}

/// Fully funded campaign via the legacy receive_contribution path (no token
/// transfer) so existing dispute/refund tests keep working.
fn funded_campaign() -> Setup {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let investor1 = Address::generate(&env);
    let investor2 = Address::generate(&env);
    let campaign_id = 1u64;
    let token_address = Address::generate(&env);
    let deadline = 1000000u64;
    let harvest_metadata = Symbol::new(&env, "maize");

    client.initialize(&admin);
    client.create_campaign(
        &campaign_id,
        &farmer,
        &1000i128,
        &token_address,
        &deadline,
        &harvest_metadata,
    );
    client.receive_contribution(&campaign_id, &investor1, &600i128);
    client.receive_contribution(&campaign_id, &investor2, &400i128);
    client.complete_funding(&campaign_id, &1000i128);

    Setup {
        env,
        client,
        admin,
        farmer,
        investor1,
        investor2,
        campaign_id,
        token: token_address,
    }
}

/// Campaign funded via fund_campaign with real token transfers.
fn token_funded_campaign() -> Setup {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let investor1 = Address::generate(&env);
    let investor2 = Address::generate(&env);
    let campaign_id = 1u64;
    let deadline = 1000000u64;
    let harvest_metadata = Symbol::new(&env, "maize");

    let (token_address, sac) = create_token(&env, &admin);

    // Mint enough tokens to investors.
    sac.mint(&investor1, &600i128);
    sac.mint(&investor2, &400i128);

    client.initialize(&admin);
    client.create_campaign(
        &campaign_id,
        &farmer,
        &1000i128,
        &token_address,
        &deadline,
        &harvest_metadata,
    );

    // fund_campaign transfers tokens and auto-transitions to Funded at target.
    client.fund_campaign(&campaign_id, &investor1, &600i128);
    client.fund_campaign(&campaign_id, &investor2, &400i128);

    Setup {
        env,
        client,
        admin,
        farmer,
        investor1,
        investor2,
        campaign_id,
        token: token_address,
    }
}

// ─── fund_campaign tests ─────────────────────────────────────────────────────

#[test]
fn test_fund_campaign_single_investor_reaches_target() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let investor = Address::generate(&env);
    let campaign_id = 1u64;

    let (token_address, sac) = create_token(&env, &admin);
    sac.mint(&investor, &1000i128);

    client.initialize(&admin);
    client.create_campaign(
        &campaign_id,
        &farmer,
        &1000i128,
        &token_address,
        &1000000u64,
        &Symbol::new(&env, "wheat"),
    );

    client.fund_campaign(&campaign_id, &investor, &1000i128);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.total_funded, 1000);
    assert_eq!(campaign.status, CampaignStatus::Funded);

    let contribution = client.get_contribution(&campaign_id, &investor);
    assert_eq!(contribution, 1000);

    // Token balance moved to contract.
    let token = TokenClient::new(&env, &token_address);
    assert_eq!(token.balance(&investor), 0);
    assert_eq!(token.balance(&contract_id), 1000);
}

#[test]
fn test_fund_campaign_multi_investor_tracks_contributions() {
    let s = token_funded_campaign();

    assert_eq!(s.client.get_contribution(&s.campaign_id, &s.investor1), 600);
    assert_eq!(s.client.get_contribution(&s.campaign_id, &s.investor2), 400);

    let campaign = s.client.get_campaign(&s.campaign_id);
    assert_eq!(campaign.total_funded, 1000);
    assert_eq!(campaign.status, CampaignStatus::Funded);
}

#[test]
fn test_fund_campaign_status_is_funding_before_target() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let investor = Address::generate(&env);
    let campaign_id = 1u64;

    let (token_address, sac) = create_token(&env, &admin);
    sac.mint(&investor, &500i128);

    client.initialize(&admin);
    client.create_campaign(
        &campaign_id,
        &farmer,
        &1000i128,
        &token_address,
        &1000000u64,
        &Symbol::new(&env, "rice"),
    );

    client.fund_campaign(&campaign_id, &investor, &500i128);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.status, CampaignStatus::Funding);
    assert_eq!(campaign.total_funded, 500);
}

#[test]
fn test_fund_campaign_emits_contribution_and_funded_events() {
    let s = token_funded_campaign();

    let events = s.env.events().all();
    // Last event should be CampaignFunded (emitted when target hit).
    let funded_event = events
        .iter()
        .rev()
        .find(|e| {
            e.1 == (Symbol::new(&s.env, "CampaignFunded"), s.campaign_id)
                .into_val(&s.env)
        });
    assert!(funded_event.is_some(), "CampaignFunded event not emitted");

    let contrib_event = events
        .iter()
        .rev()
        .find(|e| {
            e.1 == (Symbol::new(&s.env, "ContribReceived"), s.campaign_id)
                .into_val(&s.env)
        });
    assert!(contrib_event.is_some(), "ContribReceived event not emitted");
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_fund_campaign_zero_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let investor = Address::generate(&env);
    let (token_address, _) = create_token(&env, &admin);

    client.initialize(&admin);
    client.create_campaign(
        &1u64,
        &farmer,
        &1000i128,
        &token_address,
        &1000000u64,
        &Symbol::new(&env, "corn"),
    );
    client.fund_campaign(&1u64, &investor, &0i128);
}

#[test]
#[should_panic(expected = "contribution exceeds remaining target")]
fn test_fund_campaign_overfunding_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let investor = Address::generate(&env);

    let (token_address, sac) = create_token(&env, &admin);
    sac.mint(&investor, &2000i128);

    client.initialize(&admin);
    client.create_campaign(
        &1u64,
        &farmer,
        &1000i128,
        &token_address,
        &1000000u64,
        &Symbol::new(&env, "corn"),
    );
    client.fund_campaign(&1u64, &investor, &1500i128);
}

#[test]
#[should_panic(expected = "campaign not accepting contributions")]
fn test_fund_campaign_non_active_status_fails() {
    let s = token_funded_campaign(); // already Funded
    let extra_investor = Address::generate(&s.env);
    let (_, sac) = create_token(&s.env, &s.admin);
    sac.mint(&extra_investor, &100i128);
    s.client.fund_campaign(&s.campaign_id, &extra_investor, &100i128);
}

// ─── configure_tranches tests ────────────────────────────────────────────────

fn make_tranche(env: &Env, amount: i128, milestone: &str) -> Tranche {
    Tranche {
        amount,
        milestone: Symbol::new(env, milestone),
        released: false,
    }
}

#[test]
fn test_configure_tranches_stores_correctly() {
    let s = funded_campaign();
    let mut tranches: Vec<Tranche> = Vec::new(&s.env);
    tranches.push_back(make_tranche(&s.env, 400, "planting"));
    tranches.push_back(make_tranche(&s.env, 300, "midseason"));
    tranches.push_back(make_tranche(&s.env, 300, "harvest"));

    s.client.configure_tranches(&s.campaign_id, &tranches);

    let stored = s.client.get_tranches(&s.campaign_id);
    assert_eq!(stored.len(), 3);
    assert_eq!(stored.get(0).unwrap().amount, 400);
    assert_eq!(stored.get(1).unwrap().amount, 300);
    assert_eq!(stored.get(2).unwrap().amount, 300);
}

#[test]
fn test_configure_tranches_emits_event() {
    let s = funded_campaign();
    let mut tranches: Vec<Tranche> = Vec::new(&s.env);
    tranches.push_back(make_tranche(&s.env, 500, "planting"));
    tranches.push_back(make_tranche(&s.env, 500, "harvest"));

    s.client.configure_tranches(&s.campaign_id, &tranches);

    let events = s.env.events().all();
    let event = events
        .iter()
        .rev()
        .find(|e| {
            e.1 == (Symbol::new(&s.env, "TranchesConfigured"), s.campaign_id)
                .into_val(&s.env)
        });
    assert!(event.is_some());
}

#[test]
#[should_panic(expected = "can only configure tranches for a funded campaign")]
fn test_configure_tranches_on_active_campaign_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let token = Address::generate(&env);

    client.initialize(&admin);
    client.create_campaign(
        &1u64, &farmer, &1000i128, &token, &1000000u64,
        &Symbol::new(&env, "corn"),
    );

    let mut tranches: Vec<Tranche> = Vec::new(&env);
    tranches.push_back(make_tranche(&env, 500, "planting"));
    client.configure_tranches(&1u64, &tranches);
}

#[test]
#[should_panic(expected = "total tranche amounts exceed funded amount")]
fn test_configure_tranches_exceeding_funded_amount_fails() {
    let s = funded_campaign();
    let mut tranches: Vec<Tranche> = Vec::new(&s.env);
    tranches.push_back(make_tranche(&s.env, 600, "planting"));
    tranches.push_back(make_tranche(&s.env, 600, "harvest")); // 1200 > 1000
    s.client.configure_tranches(&s.campaign_id, &tranches);
}

#[test]
#[should_panic]
fn test_configure_tranches_unauthorized_fails() {
    let s = funded_campaign();
    s.env.mock_auths(&[]); // clear admin auth
    let mut tranches: Vec<Tranche> = Vec::new(&s.env);
    tranches.push_back(make_tranche(&s.env, 500, "planting"));
    s.client.configure_tranches(&s.campaign_id, &tranches);
}

// ─── release_tranche tests ───────────────────────────────────────────────────

#[test]
fn test_release_tranche_updates_accounting() {
    let s = funded_campaign();
    s.client.release_tranche(&s.campaign_id, &s.farmer, &300i128);

    let campaign = s.client.get_campaign(&s.campaign_id);
    assert_eq!(campaign.released, 300);
    // escrow_held = 1000 - 300 - 0 = 700
    assert_eq!(campaign.total_funded - campaign.released - campaign.refundable, 700);
}

#[test]
fn test_release_tranche_emits_event() {
    let s = funded_campaign();
    s.client.release_tranche(&s.campaign_id, &s.farmer, &200i128);

    let events = s.env.events().all();
    let event = events
        .iter()
        .rev()
        .find(|e| {
            e.1 == (Symbol::new(&s.env, "TrancheReleased"), s.campaign_id)
                .into_val(&s.env)
        });
    assert!(event.is_some());
}

#[test]
fn test_release_tranche_with_configured_tranches_marks_released() {
    let s = funded_campaign();
    let mut tranches: Vec<Tranche> = Vec::new(&s.env);
    tranches.push_back(make_tranche(&s.env, 400, "planting"));
    tranches.push_back(make_tranche(&s.env, 600, "harvest"));
    s.client.configure_tranches(&s.campaign_id, &tranches);

    s.client.release_tranche(&s.campaign_id, &s.farmer, &400i128);

    let stored = s.client.get_tranches(&s.campaign_id);
    assert!(stored.get(0).unwrap().released);
    assert!(!stored.get(1).unwrap().released);
}

#[test]
#[should_panic(expected = "amount exceeds escrow balance")]
fn test_release_tranche_over_release_prevented() {
    let s = funded_campaign();
    // Try to release more than escrowed.
    s.client.release_tranche(&s.campaign_id, &s.farmer, &1001i128);
}

#[test]
#[should_panic(expected = "cannot release tranche: campaign is in a terminal state")]
fn test_release_tranche_blocked_for_disputed_campaign() {
    let s = funded_campaign();
    s.client.open_dispute(
        &s.campaign_id,
        &s.investor1,
        &Symbol::new(&s.env, "Delay"),
    );
    s.client.release_tranche(&s.campaign_id, &s.farmer, &100i128);
}

#[test]
#[should_panic(expected = "cannot release tranche: campaign is in a terminal state")]
fn test_release_tranche_blocked_for_resolved_campaign() {
    let s = funded_campaign();
    s.client.open_dispute(
        &s.campaign_id,
        &s.investor1,
        &Symbol::new(&s.env, "Delay"),
    );
    s.client.resolve_dispute(
        &s.campaign_id,
        &DisputeResolution::FullRefund,
        &0i128,
    );
    s.client.release_tranche(&s.campaign_id, &s.farmer, &100i128);
}

#[test]
#[should_panic(expected = "cannot release tranche: campaign is in a terminal state")]
fn test_release_tranche_blocked_for_settled_campaign() {
    let s = funded_campaign();
    s.client.settle_campaign(&s.campaign_id, &s.farmer, &1000i128);
    s.client.release_tranche(&s.campaign_id, &s.farmer, &100i128);
}

#[test]
#[should_panic]
fn test_release_tranche_unauthorized_non_admin_fails() {
    let s = funded_campaign();
    s.env.mock_auths(&[]); // no admin auth
    s.client.release_tranche(&s.campaign_id, &s.farmer, &100i128);
}

// ─── existing tests (unchanged) ──────────────────────────────────────────────

#[test]
fn test_open_dispute_records_fields_and_status() {
    let s = funded_campaign();
    let reason = Symbol::new(&s.env, "Delay");
    s.client.open_dispute(&s.campaign_id, &s.investor1, &reason);

    let dispute = s.client.get_dispute(&s.campaign_id);
    assert_eq!(dispute.campaign_id, s.campaign_id);
    assert_eq!(dispute.opener, s.investor1);
    assert_eq!(dispute.reason, reason);
    assert_eq!(dispute.status, DisputeStatus::Open);
    assert_eq!(dispute.resolution, DisputeResolution::Pending);

    let campaign = s.client.get_campaign(&s.campaign_id);
    assert_eq!(campaign.status, CampaignStatus::Disputed);

    let events = s.env.events().all();
    let event = events.last().unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&s.env, "DisputeOpened"), s.campaign_id).into_val(&s.env)
    );
}

#[test]
fn test_farmer_can_open_dispute() {
    let s = funded_campaign();
    s.client
        .open_dispute(&s.campaign_id, &s.farmer, &Symbol::new(&s.env, "Quality"));
    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Disputed
    );
}

#[test]
#[should_panic(expected = "not authorized to open dispute")]
fn test_open_dispute_unauthorized_fails() {
    let s = funded_campaign();
    let stranger = Address::generate(&s.env);
    s.client
        .open_dispute(&s.campaign_id, &stranger, &Symbol::new(&s.env, "Nope"));
}

#[test]
fn test_resolve_full_refund() {
    let s = funded_campaign();
    s.client
        .open_dispute(&s.campaign_id, &s.investor1, &Symbol::new(&s.env, "Delay"));
    s.client
        .resolve_dispute(&s.campaign_id, &DisputeResolution::FullRefund, &0i128);

    let campaign = s.client.get_campaign(&s.campaign_id);
    assert_eq!(campaign.released, 0);
    assert_eq!(campaign.refundable, 1000);
    assert_eq!(campaign.status, CampaignStatus::Resolved);
}

#[test]
fn test_resolve_partial_settlement() {
    let s = funded_campaign();
    s.client
        .open_dispute(&s.campaign_id, &s.investor1, &Symbol::new(&s.env, "Delay"));
    s.client
        .resolve_dispute(&s.campaign_id, &DisputeResolution::PartialSettlement, &300i128);

    let campaign = s.client.get_campaign(&s.campaign_id);
    assert_eq!(campaign.released, 300);
    assert_eq!(campaign.refundable, 700);
    assert_eq!(campaign.status, CampaignStatus::Resolved);
}

#[test]
fn test_resolve_full_payout() {
    let s = funded_campaign();
    s.client
        .open_dispute(&s.campaign_id, &s.investor1, &Symbol::new(&s.env, "Delay"));
    s.client
        .resolve_dispute(&s.campaign_id, &DisputeResolution::FullPayout, &0i128);

    let campaign = s.client.get_campaign(&s.campaign_id);
    assert_eq!(campaign.released, 1000);
    assert_eq!(campaign.refundable, 0);
    assert_eq!(campaign.status, CampaignStatus::Resolved);
}

#[test]
#[should_panic]
fn test_resolve_unauthorized_fails() {
    let s = funded_campaign();
    s.client
        .open_dispute(&s.campaign_id, &s.investor1, &Symbol::new(&s.env, "Delay"));

    s.env.mock_auths(&[]);
    s.client
        .resolve_dispute(&s.campaign_id, &DisputeResolution::FullPayout, &0i128);
}

#[test]
#[should_panic(expected = "campaign is disputed")]
fn test_disputed_campaign_blocks_settlement() {
    let s = funded_campaign();
    s.client
        .open_dispute(&s.campaign_id, &s.investor1, &Symbol::new(&s.env, "Delay"));
    s.client.settle_campaign(&s.campaign_id, &s.farmer, &500i128);
}

#[test]
fn test_claim_refund_full_refund_returns_contribution() {
    let s = funded_campaign();
    s.client
        .open_dispute(&s.campaign_id, &s.investor1, &Symbol::new(&s.env, "Delay"));
    s.client
        .resolve_dispute(&s.campaign_id, &DisputeResolution::FullRefund, &0i128);
    s.client.claim_refund(&s.campaign_id, &s.investor1);
    assert!(s.client.try_claim_refund(&s.campaign_id, &s.investor1).is_err());
    let event = s.env.events().all().last().unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&s.env, "RefundClaimed"), s.campaign_id).into_val(&s.env)
    );
    let err = s.client.try_claim_refund(&s.campaign_id, &s.investor1);
    assert!(err.is_err());
}

#[test]
fn test_claim_refund_partial_settlement_is_pro_rata() {
    let s = funded_campaign();
    s.client
        .open_dispute(&s.campaign_id, &s.investor1, &Symbol::new(&s.env, "Delay"));
    s.client
        .resolve_dispute(&s.campaign_id, &DisputeResolution::PartialSettlement, &300i128);

    s.client.claim_refund(&s.campaign_id, &s.investor1);
    s.client.claim_refund(&s.campaign_id, &s.investor2);

    assert!(s.client.try_claim_refund(&s.campaign_id, &s.investor1).is_err());
    assert!(s.client.try_claim_refund(&s.campaign_id, &s.investor2).is_err());
}

#[test]
fn test_happy_path_settlement() {
    let s = funded_campaign();
    s.client.report_harvest(&s.campaign_id, &s.farmer);
    s.client.settle_campaign(&s.campaign_id, &s.farmer, &1000i128);

    let campaign = s.client.get_campaign(&s.campaign_id);
    assert_eq!(campaign.released, 1000);
    assert_eq!(campaign.status, CampaignStatus::Settled);
}

#[test]
fn test_create_campaign_successful() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let token_address = Address::generate(&env);
    let campaign_id = 42u64;

    client.initialize(&admin);
    client.create_campaign(
        &campaign_id, &farmer, &5000i128, &token_address,
        &2000000u64, &Symbol::new(&env, "wheat"),
    );

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.farmer, farmer);
    assert_eq!(campaign.total_funded, 0);
    assert_eq!(campaign.status, CampaignStatus::Active);
}

#[test]
#[should_panic(expected = "campaign already exists")]
fn test_create_campaign_duplicate_id_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let token = Address::generate(&env);

    client.initialize(&admin);
    client.create_campaign(&1u64, &farmer, &1000i128, &token, &1000000u64, &Symbol::new(&env, "corn"));
    client.create_campaign(&1u64, &farmer, &1000i128, &token, &1000000u64, &Symbol::new(&env, "corn"));
}

#[test]
#[should_panic(expected = "target amount must be greater than zero")]
fn test_create_campaign_zero_target_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let token = Address::generate(&env);

    client.initialize(&admin);
    client.create_campaign(&1u64, &farmer, &0i128, &token, &1000000u64, &Symbol::new(&env, "soy"));
}

#[test]
#[should_panic(expected = "target amount must be greater than zero")]
fn test_create_campaign_negative_target_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let token = Address::generate(&env);

    client.initialize(&admin);
    client.create_campaign(&1u64, &farmer, &-500i128, &token, &1000000u64, &Symbol::new(&env, "barley"));
}

#[test]
fn test_create_campaign_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let token = Address::generate(&env);

    client.initialize(&admin);
    client.create_campaign(&123u64, &farmer, &2500i128, &token, &1500000u64, &Symbol::new(&env, "oats"));

    let events = env.events().all();
    let event = events.last().unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "CampaignCreated"), 123u64).into_val(&env)
    );
}
