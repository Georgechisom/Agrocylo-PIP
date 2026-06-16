use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CampaignStatus {
    Funding,
    Funded,
    InProduction,
    Harvested,
    Settled,
    Failed,
    Disputed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Campaign {
    pub id: u64,
    pub farmer: Address,
    pub goal_amount: i128,
    pub raised_amount: i128,
    pub deadline: u64,
    pub status: CampaignStatus,
    pub created_at: u64,
}

#[contracttype]
pub enum DataKey {
    Campaign(u64),
    CampaignCount,
}
