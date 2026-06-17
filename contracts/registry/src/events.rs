use crate::types::ActivityRecord;
use soroban_sdk::{symbol_short, Address, Env, String};

pub fn admin_updated(env: &Env, old_admin: Address, new_admin: Address) {
    env.events()
        .publish((symbol_short!("admin_upd"),), (old_admin, new_admin));
}

pub fn contract_approved(env: &Env, contract: Address) {
    env.events()
        .publish((symbol_short!("cont_appr"),), contract);
}

pub fn contract_revoked(env: &Env, contract: Address) {
    env.events()
        .publish((symbol_short!("cont_revo"),), contract);
}

pub fn activity_recorded(env: &Env, campaign_id: u64, record: ActivityRecord) {
    env.events().publish(
        (symbol_short!("act_rec"), campaign_id),
        (
            record.actor,
            record.action_type,
            record.timestamp,
            record.ledger_sequence,
        ),
    );
}

pub fn farmer_registered(env: &Env, farmer: Address, name: String) {
    env.events()
        .publish((symbol_short!("farm_reg"),), (farmer, name));
}

pub fn campaign_registered(env: &Env, campaign_id: u64, farmer: Address, title: String) {
    env.events()
        .publish((symbol_short!("camp_reg"),), (campaign_id, farmer, title));
}
