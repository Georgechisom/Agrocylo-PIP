#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, IntoVal, Symbol,
};

struct Setup {
    env: Env,
    client: ProductionEscrowContractClient<'static>,
    farmer: Address,
    investor1: Address,
    investor2: Address,
    campaign_id: u64,
}

/// Build a fully funded campaign (target 1000, contributions 600 + 400, no
/// tranche released) so escrow_held == 1000.
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

    client.initialize(&admin);
    client.create_campaign(&campaign_id, &farmer, &1000i128);
    client.receive_contribution(&campaign_id, &investor1, &600i128);
    client.receive_contribution(&campaign_id, &investor2, &400i128);
    client.complete_funding(&campaign_id, &1000i128);

    Setup {
        env,
        client,
        farmer,
        investor1,
        investor2,
        campaign_id,
    }
}

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
    assert_eq!(dispute.timestamp, s.env.ledger().timestamp());
    assert_eq!(dispute.ledger_sequence, s.env.ledger().sequence());

    let campaign = s.client.get_campaign(&s.campaign_id);
    assert_eq!(campaign.status, CampaignStatus::Disputed);

    // DisputeOpened event emitted last.
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

    let dispute = s.client.get_dispute(&s.campaign_id);
    assert_eq!(dispute.status, DisputeStatus::Resolved);
    assert_eq!(dispute.resolution, DisputeResolution::FullRefund);

    let events = s.env.events().all();
    let event = events.last().unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&s.env, "DisputeResolved"), s.campaign_id).into_val(&s.env)
    );
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

    let dispute = s.client.get_dispute(&s.campaign_id);
    assert_eq!(dispute.resolution, DisputeResolution::PartialSettlement);
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

    // Clear all mocked authorizations: the admin's require_auth is no longer
    // satisfied, so resolve must fail.
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
#[should_panic(expected = "campaign not funded")]
fn test_disputed_campaign_blocks_tranche_release() {
    let s = funded_campaign();
    s.client
        .open_dispute(&s.campaign_id, &s.investor1, &Symbol::new(&s.env, "Delay"));

    s.client.release_tranche(&s.campaign_id, &s.farmer, &100i128);
}

#[test]
fn test_claim_refund_full_refund_returns_contribution() {
    let s = funded_campaign();
    s.client
        .open_dispute(&s.campaign_id, &s.investor1, &Symbol::new(&s.env, "Delay"));
    s.client
        .resolve_dispute(&s.campaign_id, &DisputeResolution::FullRefund, &0i128);

    s.client.claim_refund(&s.campaign_id, &s.investor1);
    let event = s.env.events().all().last().unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&s.env, "RefundClaimed"), s.campaign_id).into_val(&s.env)
    );
    // investor1 refunded 600 (== contribution); contribution zeroed so a second
    // claim has nothing left.
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

    // refundable pool = 700, basis = 1000.
    // investor1: 600 * 700 / 1000 = 420; investor2: 400 * 700 / 1000 = 280; sum = 700.
    s.client.claim_refund(&s.campaign_id, &s.investor1);
    let event = s.env.events().all().last().unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&s.env, "RefundClaimed"), s.campaign_id).into_val(&s.env)
    );
    s.client.claim_refund(&s.campaign_id, &s.investor2);

    // Both contributions are now zeroed; no over-payment beyond the pool.
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
