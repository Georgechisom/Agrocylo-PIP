use crate::types::{ActivityAction, ActivityRecord};
use soroban_sdk::{Address, Env, Symbol};

pub const ADMIN_INITIALIZED: &str = "AdminInitialized";
pub const ADMIN_UPDATED: &str = "AdminUpdated";
pub const CONTRACT_APPROVED: &str = "ContractApproved";
pub const CONTRACT_REVOKED: &str = "ContractRevoked";
pub const FARMER_REGISTERED: &str = "FarmerRegistered";
pub const CAMPAIGN_REGISTERED: &str = "CampaignRegistered";
pub const CAMPAIGN_STATUS_UPDATED: &str = "CampaignStatusUpdated";
pub const ACTIVITY_RECORDED: &str = "ActivityRecorded";

// Registry event contract for indexers:
// - AdminInitialized(admin) -> (admin, timestamp, ledger_sequence)
// - AdminUpdated(new_admin) -> (actor, old_admin, new_admin, timestamp, ledger_sequence)
// - ContractApproved(contract) -> (actor, contract, timestamp, ledger_sequence)
// - ContractRevoked(contract) -> (actor, contract, timestamp, ledger_sequence)
// - FarmerRegistered(farmer) -> (actor, timestamp, ledger_sequence)
// - CampaignRegistered(campaign_id) -> (actor, action_type, timestamp, ledger_sequence)
// - CampaignStatusUpdated(campaign_id) -> (actor, action_type, timestamp, ledger_sequence)
// - ActivityRecorded(campaign_id) -> (actor, action_type, timestamp, ledger_sequence)

pub fn admin_initialized(env: &Env, admin: Address) {
    env.events().publish(
        (Symbol::new(env, ADMIN_INITIALIZED), admin.clone()),
        (admin, env.ledger().timestamp(), env.ledger().sequence()),
    );
}

pub fn admin_updated(env: &Env, old_admin: Address, new_admin: Address) {
    env.events().publish(
        (Symbol::new(env, ADMIN_UPDATED), new_admin.clone()),
        (
            old_admin.clone(),
            old_admin,
            new_admin,
            env.ledger().timestamp(),
            env.ledger().sequence(),
        ),
    );
}

pub fn contract_approved(env: &Env, actor: Address, contract: Address) {
    env.events().publish(
        (Symbol::new(env, CONTRACT_APPROVED), contract.clone()),
        (
            actor,
            contract,
            env.ledger().timestamp(),
            env.ledger().sequence(),
        ),
    );
}

pub fn contract_revoked(env: &Env, actor: Address, contract: Address) {
    env.events().publish(
        (Symbol::new(env, CONTRACT_REVOKED), contract.clone()),
        (
            actor,
            contract,
            env.ledger().timestamp(),
            env.ledger().sequence(),
        ),
    );
}

pub fn activity_recorded(env: &Env, campaign_id: u64, record: ActivityRecord) {
    emit_activity_index_event(env, campaign_id, &record);

    env.events().publish(
        (Symbol::new(env, ACTIVITY_RECORDED), campaign_id),
        (
            record.actor,
            record.action_type,
            record.timestamp,
            record.ledger_sequence,
        ),
    );
}

fn emit_activity_index_event(env: &Env, campaign_id: u64, record: &ActivityRecord) {
    let payload = (
        record.actor.clone(),
        record.action_type.clone(),
        record.timestamp,
        record.ledger_sequence,
    );

    match &record.action_type {
        ActivityAction::FarmerRegistered => {
            env.events().publish(
                (Symbol::new(env, FARMER_REGISTERED), record.actor.clone()),
                (record.actor.clone(), record.timestamp, record.ledger_sequence),
            );
        }
        ActivityAction::CampaignCreated | ActivityAction::CampaignRegistered => {
            env.events()
                .publish((Symbol::new(env, CAMPAIGN_REGISTERED), campaign_id), payload);
        }
        ActivityAction::CampaignStatusChanged => {
            env.events().publish(
                (Symbol::new(env, CAMPAIGN_STATUS_UPDATED), campaign_id),
                payload,
            );
        }
        _ => {}
    }
}
