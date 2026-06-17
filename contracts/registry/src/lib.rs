#![no_std]

mod activity;
mod admin;
mod campaign;
mod events;
mod farmer;
mod storage;
mod types;

pub use types::*;

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    pub fn initialize(env: Env, admin: Address) {
        admin::initialize(&env, &admin);
    }

    pub fn update_admin(env: Env, new_admin: Address) {
        admin::update_admin(&env, &new_admin);
    }

    pub fn get_admin(env: Env) -> Address {
        admin::get_admin(&env)
    }

    pub fn approve_contract(env: Env, contract: Address) {
        admin::approve_contract(&env, &contract);
    }

    pub fn revoke_contract(env: Env, contract: Address) {
        admin::revoke_contract(&env, &contract);
    }

    pub fn is_contract_approved(env: Env, contract: Address) -> bool {
        admin::is_contract_approved(&env, &contract)
    }

    pub fn register_farmer(env: Env, farmer: Address, name: String, location: String) {
        farmer::register_farmer(&env, farmer, name, location);
    }

    pub fn get_farmer(env: Env, farmer: Address) -> Option<FarmerProfile> {
        farmer::get_farmer(&env, &farmer)
    }

    pub fn register_campaign(
        env: Env,
        campaign_id: u64,
        farmer: Address,
        title: String,
        description: String,
    ) {
        campaign::register_campaign(&env, campaign_id, farmer, title, description);
    }

    pub fn get_campaign(env: Env, campaign_id: u64) -> Option<CampaignInfo> {
        campaign::get_campaign(&env, campaign_id)
    }

    pub fn record_activity(
        env: Env,
        campaign_id: u64,
        actor: Address,
        action_type: ActivityAction,
    ) {
        activity::record_activity(&env, campaign_id, &actor, action_type);
    }

    pub fn get_campaign_activities(env: Env, campaign_id: u64) -> Vec<ActivityRecord> {
        activity::get_campaign_activities(&env, campaign_id)
    }
}

#[cfg(test)]
mod test;
