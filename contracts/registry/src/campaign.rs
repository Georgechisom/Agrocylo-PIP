use crate::{events, storage};
use crate::types::CampaignInfo;
use soroban_sdk::{Address, Env, String};

pub fn register_campaign(
    env: &Env,
    campaign_id: u64,
    farmer: Address,
    title: String,
    description: String,
) {
    farmer.require_auth();

    if storage::has_campaign(env, campaign_id) {
        panic!("campaign already registered");
    }

    let campaign = CampaignInfo {
        id: campaign_id,
        farmer: farmer.clone(),
        title: title.clone(),
        description,
        created_at: env.ledger().timestamp(),
    };

    storage::set_campaign(env, &campaign);
    storage::extend_instance_ttl(env);

    events::campaign_registered(env, campaign_id, farmer, title);
}

pub fn get_campaign(env: &Env, campaign_id: u64) -> Option<CampaignInfo> {
    storage::get_campaign(env, campaign_id)
}
