use crate::types::DisputeResolution;
use soroban_sdk::{Address, Env, Symbol};

pub fn emit_campaign_created(env: &Env, campaign_id: u64, farmer: Address, target_amount: i128) {
    let topics = (Symbol::new(env, "CampaignCreated"), campaign_id);
    let payload = (farmer, env.ledger().timestamp(), target_amount);
    env.events().publish(topics, payload);
}

pub fn emit_contribution_received(env: &Env, campaign_id: u64, investor: Address, amount: i128) {
    let topics = (Symbol::new(env, "ContributionReceived"), campaign_id);
    let payload = (investor, env.ledger().timestamp(), amount);
    env.events().publish(topics, payload);
}

pub fn emit_campaign_funded(env: &Env, campaign_id: u64, total_funded: i128) {
    // For campaign funded, the "actor" could be the contract itself or omitted.
    // We'll put a placeholder or omit actor, but requirement says "actor address".
    // We can just pass the campaign_id in topics, and (timestamp, total_funded) in payload.
    let topics = (Symbol::new(env, "CampaignFunded"), campaign_id);
    let payload = (env.ledger().timestamp(), total_funded);
    env.events().publish(topics, payload);
}

pub fn emit_tranche_released(env: &Env, campaign_id: u64, recipient: Address, amount: i128) {
    let topics = (Symbol::new(env, "TrancheReleased"), campaign_id);
    let payload = (recipient, env.ledger().timestamp(), amount);
    env.events().publish(topics, payload);
}

pub fn emit_harvest_reported(env: &Env, campaign_id: u64, farmer: Address) {
    let topics = (Symbol::new(env, "HarvestReported"), campaign_id);
    let payload = (farmer, env.ledger().timestamp());
    env.events().publish(topics, payload);
}

pub fn emit_dispute_opened(env: &Env, campaign_id: u64, opener: Address, reason: Symbol) {
    let topics = (Symbol::new(env, "DisputeOpened"), campaign_id);
    let payload = (
        opener,
        reason,
        env.ledger().timestamp(),
        env.ledger().sequence(),
    );
    env.events().publish(topics, payload);
}

pub fn emit_dispute_resolved(
    env: &Env,
    campaign_id: u64,
    admin: Address,
    resolution: DisputeResolution,
    payout_to_farmer: i128,
    refundable_to_investors: i128,
) {
    let topics = (Symbol::new(env, "DisputeResolved"), campaign_id);
    let payload = (
        admin,
        resolution,
        payout_to_farmer,
        refundable_to_investors,
        env.ledger().timestamp(),
        env.ledger().sequence(),
    );
    env.events().publish(topics, payload);
}

pub fn emit_refund_claimed(env: &Env, campaign_id: u64, investor: Address, amount: i128) {
    let topics = (Symbol::new(env, "RefundClaimed"), campaign_id);
    let payload = (investor, env.ledger().timestamp(), amount);
    env.events().publish(topics, payload);
}

pub fn emit_campaign_settled(env: &Env, campaign_id: u64, farmer: Address, final_amount: i128) {
    let topics = (Symbol::new(env, "CampaignSettled"), campaign_id);
    let payload = (farmer, env.ledger().timestamp(), final_amount);
    env.events().publish(topics, payload);
}
