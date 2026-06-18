#![no_std]

pub mod events;
mod storage;
mod types;

pub use types::*;

use events::*;
use soroban_sdk::{
    contract, contractimpl,
    token::Client as TokenClient,
    Address, Env, Symbol, Vec,
};

#[contract]
pub struct ProductionEscrowContract;

/// Funds still held in escrow (not yet released or marked refundable).
fn escrow_held(campaign: &Campaign) -> i128 {
    campaign.total_funded - campaign.released - campaign.refundable
}

fn require_admin(env: &Env) {
    storage::get_admin(env).require_auth();
}

/// Returns true if the campaign is in a terminal / blocked state where
/// tranche releases must be prevented.
fn is_terminal(status: &CampaignStatus) -> bool {
    matches!(
        status,
        CampaignStatus::Disputed
            | CampaignStatus::Resolved
            | CampaignStatus::Settled
    )
}

#[contractimpl]
impl ProductionEscrowContract {
    pub fn initialize(env: Env, admin: Address) {
        if storage::has_admin(&env) {
            panic!("admin already initialized");
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::extend_instance_ttl(&env);
    }

    pub fn create_campaign(
        env: Env,
        campaign_id: u64,
        farmer: Address,
        target_amount: i128,
        token_address: Address,
        deadline: u64,
        harvest_metadata: Symbol,
    ) {
        if storage::has_campaign(&env, campaign_id) {
            panic!("campaign already exists");
        }
        if target_amount <= 0 {
            panic!("target amount must be greater than zero");
        }

        farmer.require_auth();

        let campaign = Campaign {
            farmer: farmer.clone(),
            target_amount,
            token_address,
            deadline,
            harvest_metadata,
            total_funded: 0,
            released: 0,
            refundable: 0,
            status: CampaignStatus::Active,
        };
        storage::set_campaign(&env, campaign_id, &campaign);
        storage::extend_instance_ttl(&env);

        emit_campaign_created(&env, campaign_id, farmer, target_amount);
    }

    /// Investor funds a campaign. Requires investor authorization, transfers
    /// tokens from investor into the contract, tracks the contribution, and
    /// automatically moves the campaign to Funded when the target is reached.
    pub fn fund_campaign(env: Env, campaign_id: u64, investor: Address, amount: i128) {
        if amount <= 0 {
            panic!("amount must be positive");
        }

        investor.require_auth();

        let mut campaign = storage::get_campaign(&env, campaign_id);
        if campaign.status != CampaignStatus::Active
            && campaign.status != CampaignStatus::Funding
        {
            panic!("campaign not accepting contributions");
        }

        // Transfer tokens from investor into this contract.
        let token = TokenClient::new(&env, &campaign.token_address);
        token.transfer(&investor, &env.current_contract_address(), &amount);

        // Cap at target — reject overfunding.
        let remaining = campaign.target_amount - campaign.total_funded;
        if amount > remaining {
            panic!("contribution exceeds remaining target");
        }

        campaign.total_funded += amount;
        campaign.status = CampaignStatus::Funding;

        // Transition to Funded when target is exactly reached.
        if campaign.total_funded >= campaign.target_amount {
            campaign.status = CampaignStatus::Funded;
            emit_campaign_funded(&env, campaign_id, campaign.total_funded);
        }

        storage::set_campaign(&env, campaign_id, &campaign);

        let contributed =
            storage::get_contribution(&env, campaign_id, &investor) + amount;
        storage::set_contribution(&env, campaign_id, &investor, contributed);
        storage::extend_instance_ttl(&env);

        emit_contribution_received(&env, campaign_id, investor, amount);
    }

    /// Legacy path kept for existing tests — does not do a token transfer.
    pub fn receive_contribution(env: Env, campaign_id: u64, investor: Address, amount: i128) {
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut campaign = storage::get_campaign(&env, campaign_id);
        if campaign.status != CampaignStatus::Active {
            panic!("campaign not accepting contributions");
        }

        campaign.total_funded += amount;
        storage::set_campaign(&env, campaign_id, &campaign);

        let contributed =
            storage::get_contribution(&env, campaign_id, &investor) + amount;
        storage::set_contribution(&env, campaign_id, &investor, contributed);
        storage::extend_instance_ttl(&env);

        emit_contribution_received(&env, campaign_id, investor, amount);
    }

    pub fn complete_funding(env: Env, campaign_id: u64, total_funded: i128) {
        let mut campaign = storage::get_campaign(&env, campaign_id);
        if campaign.status != CampaignStatus::Active {
            panic!("campaign not active");
        }

        campaign.status = CampaignStatus::Funded;
        storage::set_campaign(&env, campaign_id, &campaign);
        storage::extend_instance_ttl(&env);

        emit_campaign_funded(&env, campaign_id, total_funded);
    }

    /// Admin configures ordered tranches for a funded campaign.
    /// The sum of tranche amounts must not exceed total_funded.
    pub fn configure_tranches(env: Env, campaign_id: u64, tranches: Vec<Tranche>) {
        require_admin(&env);

        let campaign = storage::get_campaign(&env, campaign_id);
        if campaign.status != CampaignStatus::Funded {
            panic!("can only configure tranches for a funded campaign");
        }
        if tranches.is_empty() {
            panic!("tranche list must not be empty");
        }

        let mut total: i128 = 0;
        for t in tranches.iter() {
            if t.amount <= 0 {
                panic!("each tranche amount must be positive");
            }
            total += t.amount;
        }
        if total > campaign.total_funded {
            panic!("total tranche amounts exceed funded amount");
        }

        storage::set_tranches(&env, campaign_id, &tranches);
        storage::extend_instance_ttl(&env);

        emit_tranches_configured(&env, campaign_id, tranches.len());
    }

    /// Admin releases the next unreleased tranche to the farmer.
    /// Blocked for terminal/disputed campaigns and when release would exceed
    /// escrow balance.
    pub fn release_tranche(env: Env, campaign_id: u64, recipient: Address, amount: i128) {
        if amount <= 0 {
            panic!("amount must be positive");
        }

        require_admin(&env);

        let mut campaign = storage::get_campaign(&env, campaign_id);

        if is_terminal(&campaign.status) {
            panic!("cannot release tranche: campaign is in a terminal state");
        }
        if campaign.status != CampaignStatus::Funded {
            panic!("campaign not funded");
        }
        if amount > escrow_held(&campaign) {
            panic!("amount exceeds escrow balance");
        }

        // If tranches are configured, mark the next unreleased one.
        let mut tranches = storage::get_tranches(&env, campaign_id);
        if !tranches.is_empty() {
            let mut found = false;
            let mut updated: Vec<Tranche> = Vec::new(&env);
            for t in tranches.iter() {
                if !found && !t.released && t.amount == amount {
                    let mut t2 = t.clone();
                    t2.released = true;
                    updated.push_back(t2);
                    found = true;
                } else {
                    updated.push_back(t);
                }
            }
            if !found {
                panic!("no matching unreleased tranche for this amount");
            }
            tranches = updated;
            storage::set_tranches(&env, campaign_id, &tranches);
        }

        campaign.released += amount;
        storage::set_campaign(&env, campaign_id, &campaign);
        storage::extend_instance_ttl(&env);

        emit_tranche_released(&env, campaign_id, recipient, amount);
    }

    pub fn report_harvest(env: Env, campaign_id: u64, farmer: Address) {
        let campaign = storage::get_campaign(&env, campaign_id);
        if campaign.status != CampaignStatus::Funded {
            panic!("campaign not funded");
        }
        storage::extend_instance_ttl(&env);

        emit_harvest_reported(&env, campaign_id, farmer);
    }

    pub fn open_dispute(env: Env, campaign_id: u64, opener: Address, reason: Symbol) {
        let mut campaign = storage::get_campaign(&env, campaign_id);
        if campaign.status != CampaignStatus::Active
            && campaign.status != CampaignStatus::Funding
            && campaign.status != CampaignStatus::Funded
        {
            panic!("campaign not disputable");
        }

        let is_farmer = campaign.farmer == opener;
        let is_contributor =
            storage::get_contribution(&env, campaign_id, &opener) > 0;
        let is_admin =
            storage::has_admin(&env) && storage::get_admin(&env) == opener;
        if !is_farmer && !is_contributor && !is_admin {
            panic!("not authorized to open dispute");
        }
        opener.require_auth();

        let dispute = Dispute {
            campaign_id,
            opener: opener.clone(),
            reason: reason.clone(),
            timestamp: env.ledger().timestamp(),
            ledger_sequence: env.ledger().sequence(),
            status: DisputeStatus::Open,
            resolution: DisputeResolution::Pending,
        };
        storage::set_dispute(&env, campaign_id, &dispute);

        campaign.status = CampaignStatus::Disputed;
        storage::set_campaign(&env, campaign_id, &campaign);
        storage::extend_instance_ttl(&env);

        emit_dispute_opened(&env, campaign_id, opener, reason);
    }

    pub fn resolve_dispute(
        env: Env,
        campaign_id: u64,
        resolution: DisputeResolution,
        payout_amount: i128,
    ) {
        require_admin(&env);

        let mut campaign = storage::get_campaign(&env, campaign_id);
        if campaign.status != CampaignStatus::Disputed {
            panic!("campaign not disputed");
        }
        let mut dispute = storage::get_dispute(&env, campaign_id);
        if dispute.status != DisputeStatus::Open {
            panic!("dispute already resolved");
        }

        let held = escrow_held(&campaign);
        let payout_to_farmer: i128;
        let refundable_to_investors: i128;
        match resolution {
            DisputeResolution::Pending => panic!("invalid resolution"),
            DisputeResolution::FullRefund => {
                if payout_amount != 0 {
                    panic!("payout must be zero for full refund");
                }
                payout_to_farmer = 0;
                refundable_to_investors = held;
            }
            DisputeResolution::FullPayout => {
                if payout_amount != 0 {
                    panic!("payout must be zero for full payout");
                }
                payout_to_farmer = held;
                refundable_to_investors = 0;
            }
            DisputeResolution::PartialSettlement => {
                if payout_amount <= 0 || payout_amount >= held {
                    panic!("invalid partial settlement amount");
                }
                payout_to_farmer = payout_amount;
                refundable_to_investors = held - payout_amount;
            }
        }

        campaign.released += payout_to_farmer;
        campaign.refundable += refundable_to_investors;
        campaign.status = CampaignStatus::Resolved;
        storage::set_campaign(&env, campaign_id, &campaign);

        dispute.status = DisputeStatus::Resolved;
        dispute.resolution = resolution.clone();
        storage::set_dispute(&env, campaign_id, &dispute);
        storage::extend_instance_ttl(&env);

        let admin = storage::get_admin(&env);
        emit_dispute_resolved(
            &env,
            campaign_id,
            admin,
            resolution,
            payout_to_farmer,
            refundable_to_investors,
        );
    }

    pub fn claim_refund(env: Env, campaign_id: u64, investor: Address) {
        let campaign = storage::get_campaign(&env, campaign_id);
        if campaign.status != CampaignStatus::Resolved {
            panic!("no refund available");
        }

        let contributed =
            storage::get_contribution(&env, campaign_id, &investor);
        if contributed <= 0 {
            panic!("nothing to refund");
        }

        let share = contributed * campaign.refundable / campaign.total_funded;
        if share <= 0 {
            panic!("nothing to refund");
        }
        investor.require_auth();

        storage::set_contribution(&env, campaign_id, &investor, 0);
        storage::extend_instance_ttl(&env);

        emit_refund_claimed(&env, campaign_id, investor, share);
    }

    pub fn settle_campaign(env: Env, campaign_id: u64, farmer: Address, final_amount: i128) {
        if final_amount <= 0 {
            panic!("amount must be positive");
        }

        let mut campaign = storage::get_campaign(&env, campaign_id);
        if campaign.status == CampaignStatus::Disputed {
            panic!("campaign is disputed");
        }
        if campaign.status != CampaignStatus::Funded {
            panic!("campaign not funded");
        }
        if final_amount > escrow_held(&campaign) {
            panic!("amount exceeds escrow balance");
        }

        campaign.released += final_amount;
        campaign.status = CampaignStatus::Settled;
        storage::set_campaign(&env, campaign_id, &campaign);
        storage::extend_instance_ttl(&env);

        emit_campaign_settled(&env, campaign_id, farmer, final_amount);
    }

    pub fn get_campaign(env: Env, campaign_id: u64) -> Campaign {
        storage::get_campaign(&env, campaign_id)
    }

    pub fn get_dispute(env: Env, campaign_id: u64) -> Dispute {
        storage::get_dispute(&env, campaign_id)
    }

    pub fn get_contribution(env: Env, campaign_id: u64, investor: Address) -> i128 {
        storage::get_contribution(&env, campaign_id, &investor)
    }

    pub fn get_tranches(env: Env, campaign_id: u64) -> Vec<Tranche> {
        storage::get_tranches(&env, campaign_id)
    }
}

#[cfg(test)]
mod test;
