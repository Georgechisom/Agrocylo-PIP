#![no_std]

mod storage;
mod types;

pub use types::*;

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct ProductionEscrowContract;

#[contractimpl]
impl ProductionEscrowContract {
    pub fn initialize(env: Env) {
        storage::extend_instance_ttl(&env);
        if storage::has_admin(&env) {
            panic!("already initialized");
        }
        storage::set_campaign_count(&env, 0);
    }

    pub fn create_campaign(
        env: Env,
        farmer: Address,
        goal_amount: i128,
        deadline: u64,
    ) -> u64 {
        farmer.require_auth();

        let count = storage::get_campaign_count(&env);
        let campaign_id = count + 1;

        let campaign = Campaign {
            id: campaign_id,
            farmer: farmer.clone(),
            goal_amount,
            raised_amount: 0,
            deadline,
            status: CampaignStatus::Funding,
            created_at: env.ledger().timestamp(),
        };

        storage::set_campaign(&env, &campaign);
        storage::set_campaign_count(&env, campaign_id);

        env.events().publish(
            ("campaign_created", campaign_id),
            farmer,
        );

        campaign_id
    }

    pub fn fund_campaign(env: Env, campaign_id: u64, investor: Address, amount: i128) {
        investor.require_auth();
        let mut campaign = storage::get_campaign(&env, campaign_id);
        campaign.raised_amount += amount;
        campaign.status = CampaignStatus::Funded;
        storage::set_campaign(&env, &campaign);

        env.events().publish(
            ("campaign_funded", campaign_id),
            (investor, amount),
        );
    }

    pub fn start_production(env: Env, campaign_id: u64) {
        let mut campaign = storage::get_campaign(&env, campaign_id);
        campaign.status = CampaignStatus::InProduction;
        storage::set_campaign(&env, &campaign);

        env.events().publish(
            ("production_started", campaign_id),
            (),
        );
    }

    pub fn report_harvest(env: Env, campaign_id: u64) {
        let mut campaign = storage::get_campaign(&env, campaign_id);
        campaign.status = CampaignStatus::Harvested;
        storage::set_campaign(&env, &campaign);

        env.events().publish(
            ("harvest_reported", campaign_id),
            (),
        );
    }

    pub fn settle_campaign(env: Env, campaign_id: u64) {
        let mut campaign = storage::get_campaign(&env, campaign_id);
        campaign.status = CampaignStatus::Settled;
        storage::set_campaign(&env, &campaign);

        env.events().publish(
            ("campaign_settled", campaign_id),
            (),
        );
    }

    pub fn refund_campaign(env: Env, campaign_id: u64) {
        let mut campaign = storage::get_campaign(&env, campaign_id);
        campaign.status = CampaignStatus::Failed;
        storage::set_campaign(&env, &campaign);

        env.events().publish(
            ("campaign_refunded", campaign_id),
            (),
        );
    }

    pub fn enter_dispute(env: Env, campaign_id: u64) {
        let mut campaign = storage::get_campaign(&env, campaign_id);
        campaign.status = CampaignStatus::Disputed;
        storage::set_campaign(&env, &campaign);

        env.events().publish(
            ("campaign_disputed", campaign_id),
            (),
        );
    }

    pub fn get_campaign(env: Env, campaign_id: u64) -> Campaign {
        storage::get_campaign(&env, campaign_id)
    }

    pub fn get_campaign_status(env: Env, campaign_id: u64) -> CampaignStatus {
        let campaign = storage::get_campaign(&env, campaign_id);
        campaign.status
    }
}

#[cfg(test)]
mod test;
