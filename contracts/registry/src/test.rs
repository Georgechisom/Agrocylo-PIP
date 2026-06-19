use crate::{ActivityAction, RegistryContract, RegistryContractClient};
use soroban_sdk::{
    testutils::{Address as _, Events},
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
    assert_eq!(
        event.2,
        (admin, env.ledger().timestamp(), env.ledger().sequence()).into_val(&env)
    );
}

#[test]
#[should_panic(expected = "admin already initialized")]
fn test_initialize_admin_twice_fails() {
    let (env, admin, _, _, client) = create_test_env();

    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
fn test_update_admin() {
    let (env, admin, user, _, client) = create_test_env();

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
    assert_eq!(
        event.2,
        (
            admin.clone(),
            admin,
            new_admin,
            env.ledger().timestamp(),
            env.ledger().sequence()
        )
            .into_val(&env)
    );
}

#[test]
fn test_update_admin_unauthorized_fails() {
    let (env, admin, user, _, client) = create_test_env();

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
    assert_eq!(
        event.2,
        (
            admin,
            contract_addr,
            env.ledger().timestamp(),
            env.ledger().sequence()
        )
            .into_val(&env)
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
    assert_eq!(
        event.2,
        (
            admin,
            contract_addr,
            env.ledger().timestamp(),
            env.ledger().sequence()
        )
            .into_val(&env)
    );
}

#[test]
#[should_panic]
fn test_approve_contract_unauthorized_fails() {
    let (env, admin, user, contract_addr, client) = create_test_env();

    client.initialize(&admin);

    env.mock_all_auths_allowing_non_root_auth();

    client.approve_contract(&contract_addr);
    
    let is_approved = client.is_contract_approved(&contract_addr);
    assert!(is_approved);
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
    assert_eq!(
        activity_event.2,
        (
            admin,
            ActivityAction::CampaignCreated,
            env.ledger().timestamp(),
            env.ledger().sequence()
        )
            .into_val(&env)
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
    assert_eq!(
        farmer_event.2,
        (farmer, env.ledger().timestamp(), env.ledger().sequence()).into_val(&env)
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
    assert_eq!(
        campaign_event.2,
        (
            admin,
            ActivityAction::CampaignRegistered,
            env.ledger().timestamp(),
            env.ledger().sequence()
        )
            .into_val(&env)
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
    assert_eq!(
        status_event.2,
        (
            admin,
            ActivityAction::CampaignStatusChanged,
            env.ledger().timestamp(),
            env.ledger().sequence()
        )
            .into_val(&env)
    );
}

#[test]
fn test_record_activity_as_approved_contract() {
    let (env, admin, _, contract_addr, client) = create_test_env();

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
    let (env, admin, _, _, client) = create_test_env();

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
    let (env, admin, _, _, client) = create_test_env();

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
    let (env, admin, _, _, client) = create_test_env();

    client.initialize(&admin);

    let campaign_id = 999u64;
    let activities = client.get_campaign_activities(&campaign_id);

    assert_eq!(activities.len(), 0);
}

#[test]
fn test_activity_timestamp_and_ledger() {
    let (env, admin, _, _, client) = create_test_env();

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
    let (env, admin, user, _, client) = create_test_env();

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
