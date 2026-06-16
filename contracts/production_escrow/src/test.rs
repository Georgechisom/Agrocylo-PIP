use crate::{CampaignStatus, ProductionEscrowContract, ProductionEscrowContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

fn create_test_env() -> (Env, Address, ProductionEscrowContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let farmer = Address::generate(&env);
    let contract_id = env.register_contract(None, ProductionEscrowContract);
    let client = ProductionEscrowContractClient::new(&env, &contract_id);

    (env, farmer, client)
}

#[test]
fn test_initialize() {
    let (_env, _farmer, client) = create_test_env();
    client.initialize();
}

#[test]
fn test_create_campaign() {
    let (_env, farmer, client) = create_test_env();
    client.initialize();

    let goal: i128 = 1000;
    let deadline: u64 = 1000000;
    let campaign_id = client.create_campaign(&farmer, &goal, &deadline);

    assert_eq!(campaign_id, 1);
}

#[test]
fn test_get_campaign() {
    let (_env, farmer, client) = create_test_env();
    client.initialize();

    let goal: i128 = 1000;
    let deadline: u64 = 1000000;
    let campaign_id = client.create_campaign(&farmer, &goal, &deadline);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.id, campaign_id);
    assert_eq!(campaign.farmer, farmer);
    assert_eq!(campaign.goal_amount, goal);
    assert_eq!(campaign.raised_amount, 0);
    assert_eq!(campaign.deadline, deadline);
    assert_eq!(campaign.status, CampaignStatus::Funding);
}

#[test]
fn test_get_campaign_status() {
    let (_env, farmer, client) = create_test_env();
    client.initialize();

    let campaign_id = client.create_campaign(&farmer, &1000, &1000000);
    let status = client.get_campaign_status(&campaign_id);

    assert_eq!(status, CampaignStatus::Funding);
}

#[test]
fn test_create_multiple_campaigns() {
    let (_env, farmer, client) = create_test_env();
    client.initialize();

    let id1 = client.create_campaign(&farmer, &1000, &1000000);
    let id2 = client.create_campaign(&farmer, &2000, &2000000);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);

    let c2 = client.get_campaign(&id2);
    assert_eq!(c2.goal_amount, 2000);
}

#[test]
fn test_campaign_status_funding_after_creation() {
    let (_env, farmer, client) = create_test_env();
    client.initialize();

    let campaign_id = client.create_campaign(&farmer, &1000, &1000000);
    let status = client.get_campaign_status(&campaign_id);

    assert_eq!(status, CampaignStatus::Funding);
}

#[test]
fn test_campaign_full_lifecycle() {
    let (_env, farmer, client) = create_test_env();
    client.initialize();

    let investor = Address::generate(&_env);
    let campaign_id = client.create_campaign(&farmer, &1000, &1000000);

    assert_eq!(client.get_campaign_status(&campaign_id), CampaignStatus::Funding);

    client.fund_campaign(&campaign_id, &investor, &500);
    assert_eq!(client.get_campaign_status(&campaign_id), CampaignStatus::Funded);

    client.start_production(&campaign_id);
    assert_eq!(client.get_campaign_status(&campaign_id), CampaignStatus::InProduction);

    client.report_harvest(&campaign_id);
    assert_eq!(client.get_campaign_status(&campaign_id), CampaignStatus::Harvested);

    client.settle_campaign(&campaign_id);
    assert_eq!(client.get_campaign_status(&campaign_id), CampaignStatus::Settled);
}

#[test]
fn test_campaign_refund() {
    let (_env, farmer, client) = create_test_env();
    client.initialize();

    let campaign_id = client.create_campaign(&farmer, &1000, &1000000);
    assert_eq!(client.get_campaign_status(&campaign_id), CampaignStatus::Funding);

    client.refund_campaign(&campaign_id);
    assert_eq!(client.get_campaign_status(&campaign_id), CampaignStatus::Failed);
}

#[test]
fn test_campaign_dispute() {
    let (_env, farmer, client) = create_test_env();
    client.initialize();

    let campaign_id = client.create_campaign(&farmer, &1000, &1000000);
    assert_eq!(client.get_campaign_status(&campaign_id), CampaignStatus::Funding);

    client.enter_dispute(&campaign_id);
    assert_eq!(client.get_campaign_status(&campaign_id), CampaignStatus::Disputed);
}

#[test]
fn test_campaign_raised_amount_tracking() {
    let (_env, farmer, client) = create_test_env();
    client.initialize();

    let investor = Address::generate(&_env);
    let campaign_id = client.create_campaign(&farmer, &1000, &1000000);

    client.fund_campaign(&campaign_id, &investor, &300);
    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.raised_amount, 300);
}
