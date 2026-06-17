#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Events, Address, Env, IntoVal, Symbol};

#[test]
fn test_events_emitted() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);

    let campaign_id = 1u64;
    let farmer = Address::generate(&env);
    let investor = Address::generate(&env);
    let target_amount = 1000i128;
    let amount = 500i128;

    // Call all functions
    client.create_campaign(&campaign_id, &farmer, &target_amount);
    client.receive_contribution(&campaign_id, &investor, &amount);
    client.complete_funding(&campaign_id, &amount);
    client.release_tranche(&campaign_id, &farmer, &amount);
    client.report_harvest(&campaign_id, &farmer);
    client.open_dispute(&campaign_id, &investor, &Symbol::new(&env, "Delay"));
    client.resolve_dispute(&campaign_id, &farmer, &Symbol::new(&env, "Resolved"));
    client.claim_refund(&campaign_id, &investor, &amount);
    client.settle_campaign(&campaign_id, &farmer, &amount);

    let events = env.events().all();
    assert_eq!(events.len(), 9);

    // Verify CampaignCreated
    let event = events.get(0).unwrap();
    assert_eq!(event.0, contract_id.clone());
    assert_eq!(
        event.1,
        (Symbol::new(&env, "CampaignCreated"), campaign_id).into_val(&env)
    );

    // Verify ContributionReceived
    let event = events.get(1).unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "ContributionReceived"), campaign_id).into_val(&env)
    );

    // Verify CampaignFunded
    let event = events.get(2).unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "CampaignFunded"), campaign_id).into_val(&env)
    );

    // Verify TrancheReleased
    let event = events.get(3).unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "TrancheReleased"), campaign_id).into_val(&env)
    );

    // Verify HarvestReported
    let event = events.get(4).unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "HarvestReported"), campaign_id).into_val(&env)
    );

    // Verify DisputeOpened
    let event = events.get(5).unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "DisputeOpened"), campaign_id).into_val(&env)
    );

    // Verify DisputeResolved
    let event = events.get(6).unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "DisputeResolved"), campaign_id).into_val(&env)
    );

    // Verify RefundClaimed
    let event = events.get(7).unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "RefundClaimed"), campaign_id).into_val(&env)
    );

    // Verify CampaignSettled
    let event = events.get(8).unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "CampaignSettled"), campaign_id).into_val(&env)
    );
}
