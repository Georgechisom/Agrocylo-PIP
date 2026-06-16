use soroban_sdk::Env;
use crate::types::{Campaign, DataKey};

const DAY_IN_LEDGERS: u32 = 17280;
const INSTANCE_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS * 30;
const INSTANCE_BUMP_AMOUNT: u32 = DAY_IN_LEDGERS * 90;

pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::CampaignCount)
}

pub fn get_campaign_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::CampaignCount)
        .unwrap_or(0)
}

pub fn set_campaign_count(env: &Env, count: u64) {
    env.storage()
        .instance()
        .set(&DataKey::CampaignCount, &count);
}

pub fn get_campaign(env: &Env, id: u64) -> Campaign {
    env.storage()
        .persistent()
        .get(&DataKey::Campaign(id))
        .unwrap()
}

#[allow(dead_code)]
pub fn has_campaign(env: &Env, id: u64) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Campaign(id))
}

pub fn set_campaign(env: &Env, campaign: &Campaign) {
    let key = DataKey::Campaign(campaign.id);
    env.storage().persistent().set(&key, campaign);
    env.storage()
        .persistent()
        .extend_ttl(&key, INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}
