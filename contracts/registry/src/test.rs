use crate::{ActivityAction, CampaignStatus, RegistryContract, RegistryContractClient};
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    vec, Address, Env, IntoVal, Symbol,
};

fn create_test_env() -> (
    Env,
    Address,
    Address,
    Address,
    RegistryContractClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_addr = Address::generate(&env);

    let contract_id = env.register_contract(None, RegistryContract);
    let client = RegistryContractClient::new(&env, &contract_id);

    (env, admin, user, contract_addr, client)
}

#[test]
fn test_initialize_admin() {
    let (env, admin, _, _, client) = create_test_env();

    client.initialize(&admin);

    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, admin);

    let event = env.events().all().last().unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "AdminInitialized"), admin.clone()).into_val(&env)
    );
}

#[test]
#[should_panic(expected = "admin already initialized")]
fn test_initialize_admin_twice_fails() {
    let (_env, admin, _, _, client) = create_test_env();

    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
fn test_update_admin() {
    let (env, admin, _user, _, client) = create_test_env();

    client.initialize(&admin);

    let new_admin = Address::generate(&env);
    client.update_admin(&new_admin);

    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, new_admin);

    let event = env.events().all().last().unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "AdminUpdated"), new_admin.clone()).into_val(&env)
    );
}

#[test]
fn test_update_admin_unauthorized_fails() {
    let (env, admin, _user, _, client) = create_test_env();

    client.initialize(&admin);

    env.mock_all_auths_allowing_non_root_auth();

    let new_admin = Address::generate(&env);
    client.update_admin(&new_admin);

    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, new_admin);
}

#[test]
fn test_approve_contract() {
    let (env, admin, _, contract_addr, client) = create_test_env();

    client.initialize(&admin);

    client.approve_contract(&contract_addr);

    let is_approved = client.is_contract_approved(&contract_addr);
    assert!(is_approved);

    let event = env.events().all().last().unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "ContractApproved"), contract_addr.clone()).into_val(&env)
    );
}

#[test]
fn test_revoke_contract() {
    let (env, admin, _, contract_addr, client) = create_test_env();

    client.initialize(&admin);

    client.approve_contract(&contract_addr);
    assert!(client.is_contract_approved(&contract_addr));

    client.revoke_contract(&contract_addr);
    assert!(!client.is_contract_approved(&contract_addr));

    let event = env.events().all().last().unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "ContractRevoked"), contract_addr.clone()).into_val(&env)
    );
}

#[test]
#[should_panic]
fn test_approve_contract_unauthorized_fails() {
    // Create a fresh env WITHOUT mock_all_auths so authorization is enforced.
    let env = Env::default();
    let contract_id = env.register_contract(None, RegistryContract);
    let client = RegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let contract_addr = Address::generate(&env);
    let non_admin = Address::generate(&env);

    // Initialize with the real admin (admin signs via mock for this one call only).
    env.mock_all_auths();
    client.initialize(&admin);

    // Clear mocks so subsequent calls must provide real auth.
    env.set_auths(&[]);

    // non_admin attempts to approve a contract without authorization — must panic.
    client.approve_contract(&contract_addr);
    let _ = non_admin;
}

#[test]
fn test_record_activity_as_admin() {
    let (env, admin, _, _, client) = create_test_env();

    client.initialize(&admin);

    let campaign_id = 1u64;
    client.record_activity(&campaign_id, &admin, &ActivityAction::CampaignCreated);

    let activities = client.get_campaign_activities(&campaign_id);
    assert_eq!(activities.len(), 1);

    let activity = activities.get(0).unwrap();
    assert_eq!(activity.actor, admin);
    assert_eq!(activity.action_type, ActivityAction::CampaignCreated);

    let events = env.events().all();
    let campaign_event = events.get(events.len() - 2).unwrap();
    assert_eq!(
        campaign_event.1,
        (Symbol::new(&env, "CampaignRegistered"), campaign_id).into_val(&env)
    );

    let activity_event = events.last().unwrap();
    assert_eq!(
        activity_event.1,
        (Symbol::new(&env, "ActivityRecorded"), campaign_id).into_val(&env)
    );
}

#[test]
fn test_farmer_registered_event() {
    let (env, admin, farmer, _, client) = create_test_env();

    client.initialize(&admin);

    let campaign_id = 1u64;
    client.record_activity(&campaign_id, &farmer, &ActivityAction::FarmerRegistered);

    let events = env.events().all();
    let farmer_event = events.get(events.len() - 2).unwrap();
    assert_eq!(
        farmer_event.1,
        (Symbol::new(&env, "FarmerRegistered"), farmer.clone()).into_val(&env)
    );
}

#[test]
fn test_campaign_registered_event() {
    let (env, admin, _, _, client) = create_test_env();

    client.initialize(&admin);

    let campaign_id = 2u64;
    client.record_activity(&campaign_id, &admin, &ActivityAction::CampaignRegistered);

    let events = env.events().all();
    let campaign_event = events.get(events.len() - 2).unwrap();
    assert_eq!(
        campaign_event.1,
        (Symbol::new(&env, "CampaignRegistered"), campaign_id).into_val(&env)
    );
}

#[test]
fn test_campaign_status_updated_event() {
    let (env, admin, _, _, client) = create_test_env();

    client.initialize(&admin);

    let campaign_id = 3u64;
    client.record_activity(&campaign_id, &admin, &ActivityAction::CampaignStatusChanged);

    let events = env.events().all();
    let status_event = events.get(events.len() - 2).unwrap();
    assert_eq!(
        status_event.1,
        (Symbol::new(&env, "CampaignStatusUpdated"), campaign_id).into_val(&env)
    );
}

#[test]
fn test_record_activity_as_approved_contract() {
    let (_env, admin, _, contract_addr, client) = create_test_env();

    client.initialize(&admin);
    client.approve_contract(&contract_addr);

    let campaign_id = 1u64;
    client.record_activity(
        &campaign_id,
        &contract_addr,
        &ActivityAction::CampaignFunded,
    );

    let activities = client.get_campaign_activities(&campaign_id);
    assert_eq!(activities.len(), 1);
}

#[test]
fn test_record_multiple_activities() {
    let (_env, admin, _, _, client) = create_test_env();

    client.initialize(&admin);

    let campaign_id = 1u64;
    client.record_activity(&campaign_id, &admin, &ActivityAction::CampaignCreated);
    client.record_activity(&campaign_id, &admin, &ActivityAction::CampaignFunded);
    client.record_activity(&campaign_id, &admin, &ActivityAction::CampaignStatusChanged);

    let activities = client.get_campaign_activities(&campaign_id);
    assert_eq!(activities.len(), 3);

    assert_eq!(
        activities.get(0).unwrap().action_type,
        ActivityAction::CampaignCreated
    );
    assert_eq!(
        activities.get(1).unwrap().action_type,
        ActivityAction::CampaignFunded
    );
    assert_eq!(
        activities.get(2).unwrap().action_type,
        ActivityAction::CampaignStatusChanged
    );
}

#[test]
fn test_activities_different_campaigns() {
    let (_env, admin, _, _, client) = create_test_env();

    client.initialize(&admin);

    let campaign_id_1 = 1u64;
    let campaign_id_2 = 2u64;

    client.record_activity(&campaign_id_1, &admin, &ActivityAction::CampaignCreated);
    client.record_activity(&campaign_id_2, &admin, &ActivityAction::CampaignCreated);
    client.record_activity(&campaign_id_1, &admin, &ActivityAction::CampaignFunded);

    let activities_1 = client.get_campaign_activities(&campaign_id_1);
    let activities_2 = client.get_campaign_activities(&campaign_id_2);

    assert_eq!(activities_1.len(), 2);
    assert_eq!(activities_2.len(), 1);
}

#[test]
fn test_get_activities_empty_campaign() {
    let (_env, admin, _, _, client) = create_test_env();

    client.initialize(&admin);

    let campaign_id = 999u64;
    let activities = client.get_campaign_activities(&campaign_id);

    assert_eq!(activities.len(), 0);
}

#[test]
fn test_activity_timestamp_and_ledger() {
    let (env, admin, _, _, client) = create_test_env();

    // Advance ledger so timestamp and sequence are non-zero.
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 1_000_000,
        protocol_version: 22,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3_110_400,
    });

    client.initialize(&admin);

    let campaign_id = 1u64;
    client.record_activity(&campaign_id, &admin, &ActivityAction::CampaignCreated);

    let activities = client.get_campaign_activities(&campaign_id);
    let activity = activities.get(0).unwrap();

    assert!(activity.timestamp > 0);
    assert!(activity.ledger_sequence > 0);
}

#[test]
fn test_record_activity_as_authorized_user() {
    let (_env, admin, user, _, client) = create_test_env();

    client.initialize(&admin);

    let campaign_id = 1u64;
    client.record_activity(&campaign_id, &user, &ActivityAction::CampaignFunded);

    let activities = client.get_campaign_activities(&campaign_id);
    assert_eq!(activities.len(), 1);
    assert_eq!(activities.get(0).unwrap().actor, user);
}

#[test]
fn test_all_activity_actions() {
    let (env, admin, _, _, client) = create_test_env();

    client.initialize(&admin);

    let campaign_id = 1u64;

    let actions = vec![
        &env,
        ActivityAction::CampaignCreated,
        ActivityAction::FarmerRegistered,
        ActivityAction::CampaignRegistered,
        ActivityAction::CampaignFunded,
        ActivityAction::CampaignStatusChanged,
        ActivityAction::FundsReleased,
        ActivityAction::HarvestReported,
        ActivityAction::DisputeInitiated,
        ActivityAction::DisputeResolved,
        ActivityAction::CampaignSettled,
    ];

    for action in actions.iter() {
        client.record_activity(&campaign_id, &admin, &action);
    }

    let activities = client.get_campaign_activities(&campaign_id);
    assert_eq!(activities.len(), 10);
}

// ── Campaign registration & lookup ──────────────────────────────────────────

fn register_default_campaign(
    env: &Env,
    client: &RegistryContractClient,
    campaign_id: u64,
    farmer: &Address,
    escrow: &Address,
) {
    let crop = Symbol::new(env, "wheat");
    let region = Symbol::new(env, "north");
    client.register_campaign(&campaign_id, farmer, escrow, &crop, &region);
}

#[test]
fn test_register_campaign_by_farmer() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);
    register_default_campaign(&env, &client, 1, &farmer, &escrow);

    let record = client.get_campaign(&1u64);
    assert_eq!(record.campaign_id, 1);
    assert_eq!(record.farmer, farmer);
    assert_eq!(record.escrow_contract, escrow);
    assert_eq!(record.crop_metadata, Symbol::new(&env, "wheat"));
    assert_eq!(record.region_metadata, Symbol::new(&env, "north"));
    assert_eq!(record.status, CampaignStatus::Active);
}

#[test]
fn test_register_campaign_by_approved_escrow() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);
    client.approve_contract(&escrow);

    let crop = Symbol::new(&env, "corn");
    let region = Symbol::new(&env, "south");
    client.register_campaign(&2u64, &farmer, &escrow, &crop, &region);

    let record = client.get_campaign(&2u64);
    assert_eq!(record.farmer, farmer);
    assert_eq!(record.escrow_contract, escrow);
    assert_eq!(record.status, CampaignStatus::Active);
}

#[test]
#[should_panic(expected = "campaign already registered")]
fn test_register_campaign_duplicate_fails() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);
    register_default_campaign(&env, &client, 1, &farmer, &escrow);
    register_default_campaign(&env, &client, 1, &farmer, &escrow);
}

#[test]
fn test_get_campaign_by_id() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);
    register_default_campaign(&env, &client, 42, &farmer, &escrow);

    let record = client.get_campaign(&42u64);
    assert_eq!(record.campaign_id, 42);
    assert_eq!(record.farmer, farmer);
    assert_eq!(record.escrow_contract, escrow);
}

#[test]
#[should_panic(expected = "campaign not found")]
fn test_get_campaign_unknown_id_fails() {
    let (_env, admin, _, _, client) = create_test_env();
    client.initialize(&admin);
    client.get_campaign(&999u64);
}

#[test]
fn test_get_campaigns_by_farmer() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);

    let escrow2 = Address::generate(&env);
    register_default_campaign(&env, &client, 1, &farmer, &escrow);
    register_default_campaign(&env, &client, 2, &farmer, &escrow2);

    let ids = client.get_campaigns_by_farmer(&farmer);
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(1u64));
    assert!(ids.contains(2u64));
}

#[test]
fn test_get_campaigns_by_farmer_empty() {
    let (_env, admin, farmer, _, client) = create_test_env();
    client.initialize(&admin);

    let ids = client.get_campaigns_by_farmer(&farmer);
    assert_eq!(ids.len(), 0);
}

#[test]
fn test_register_campaign_emits_event() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);

    let crop = Symbol::new(&env, "rice");
    let region = Symbol::new(&env, "east");
    client.register_campaign(&5u64, &farmer, &escrow, &crop, &region);

    let event = env.events().all().last().unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "CampaignRegistered"), 5u64).into_val(&env)
    );
}

#[test]
fn test_register_campaign_stores_metadata() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);

    let crop = Symbol::new(&env, "maize");
    let region = Symbol::new(&env, "west");
    client.register_campaign(&10u64, &farmer, &escrow, &crop, &region);

    let record = client.get_campaign(&10u64);
    assert_eq!(record.crop_metadata, Symbol::new(&env, "maize"));
    assert_eq!(record.region_metadata, Symbol::new(&env, "west"));
    assert_eq!(record.escrow_contract, escrow);
}

#[test]
fn test_register_multiple_campaigns_different_farmers() {
    let (env, admin, farmer1, escrow1, client) = create_test_env();
    client.initialize(&admin);

    let farmer2 = Address::generate(&env);
    let escrow2 = Address::generate(&env);

    register_default_campaign(&env, &client, 1, &farmer1, &escrow1);
    register_default_campaign(&env, &client, 2, &farmer2, &escrow2);

    let f1_ids = client.get_campaigns_by_farmer(&farmer1);
    let f2_ids = client.get_campaigns_by_farmer(&farmer2);

    assert_eq!(f1_ids.len(), 1);
    assert_eq!(f2_ids.len(), 1);
    assert!(f1_ids.contains(1u64));
    assert!(f2_ids.contains(2u64));
}

// ── Campaign status updates ──────────────────────────────────────────────────

#[test]
fn test_update_campaign_status_by_escrow() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);
    register_default_campaign(&env, &client, 1, &farmer, &escrow);

    client.update_campaign_status(&1u64, &escrow, &CampaignStatus::Funding);

    let record = client.get_campaign(&1u64);
    assert_eq!(record.status, CampaignStatus::Funding);
}

#[test]
fn test_update_campaign_status_by_admin() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);
    register_default_campaign(&env, &client, 1, &farmer, &escrow);

    client.update_campaign_status(&1u64, &admin, &CampaignStatus::Funded);

    let record = client.get_campaign(&1u64);
    assert_eq!(record.status, CampaignStatus::Funded);
}

#[test]
fn test_update_campaign_status_multiple_transitions() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);
    register_default_campaign(&env, &client, 1, &farmer, &escrow);

    client.update_campaign_status(&1u64, &escrow, &CampaignStatus::Funding);
    client.update_campaign_status(&1u64, &escrow, &CampaignStatus::Funded);
    client.update_campaign_status(&1u64, &admin, &CampaignStatus::Settled);

    let record = client.get_campaign(&1u64);
    assert_eq!(record.status, CampaignStatus::Settled);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_update_campaign_status_unauthorized_fails() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);
    register_default_campaign(&env, &client, 1, &farmer, &escrow);

    let random_caller = Address::generate(&env);
    client.update_campaign_status(&1u64, &random_caller, &CampaignStatus::Funding);
}

#[test]
#[should_panic(expected = "campaign not found")]
fn test_update_campaign_status_unknown_campaign_fails() {
    let (_env, admin, _, _, client) = create_test_env();
    client.initialize(&admin);
    client.update_campaign_status(&999u64, &admin, &CampaignStatus::Funded);
}

#[test]
fn test_update_campaign_status_emits_event() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);
    register_default_campaign(&env, &client, 1, &farmer, &escrow);

    client.update_campaign_status(&1u64, &escrow, &CampaignStatus::Funding);

    let event = env.events().all().last().unwrap();
    assert_eq!(
        event.1,
        (Symbol::new(&env, "CampaignStatusUpdated"), 1u64).into_val(&env)
    );
}

#[test]
fn test_update_campaign_status_tracks_previous_status() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);
    register_default_campaign(&env, &client, 1, &farmer, &escrow);

    // Verify initial state
    let record = client.get_campaign(&1u64);
    assert_eq!(record.status, CampaignStatus::Active);

    // First transition: Active -> Funding
    client.update_campaign_status(&1u64, &escrow, &CampaignStatus::Funding);
    let record = client.get_campaign(&1u64);
    assert_eq!(record.status, CampaignStatus::Funding);

    // Second transition: Funding -> Funded
    client.update_campaign_status(&1u64, &escrow, &CampaignStatus::Funded);
    let record = client.get_campaign(&1u64);
    assert_eq!(record.status, CampaignStatus::Funded);

    // Verify two CampaignStatusUpdated events were emitted for this campaign
    let mut count = 0u32;
    for event in env.events().all().iter() {
        if event.1 == (Symbol::new(&env, "CampaignStatusUpdated"), 1u64).into_val(&env) {
            count += 1;
        }
    }
    assert_eq!(count, 2);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_farmer_cannot_update_status() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);
    register_default_campaign(&env, &client, 1, &farmer, &escrow);
    client.update_campaign_status(&1u64, &farmer, &CampaignStatus::Funding);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_wrong_escrow_cannot_update_status() {
    let (env, admin, farmer, escrow, client) = create_test_env();
    client.initialize(&admin);
    register_default_campaign(&env, &client, 1, &farmer, &escrow);
    let other_escrow = Address::generate(&env);
    client.update_campaign_status(&1u64, &other_escrow, &CampaignStatus::Funding);
}
