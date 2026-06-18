use soroban_sdk::{contracttype, Address, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CampaignStatus {
    Active,
    Funded,
    Disputed,
    Resolved,
    Settled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Open,
    Resolved,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    Pending,
    FullRefund,
    PartialSettlement,
    FullPayout,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Campaign {
    pub farmer: Address,
    pub target_amount: i128,
    pub token_address: Address,
    pub deadline: u64,
    pub harvest_metadata: Symbol,
    pub total_funded: i128,
    pub released: i128,
    pub refundable: i128,
    pub status: CampaignStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispute {
    pub campaign_id: u64,
    pub opener: Address,
    pub reason: Symbol,
    pub timestamp: u64,
    pub ledger_sequence: u32,
    pub status: DisputeStatus,
    pub resolution: DisputeResolution,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Campaign(u64),
    Dispute(u64),
    Contribution(u64, Address),
}
