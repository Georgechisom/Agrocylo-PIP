use soroban_sdk::{contracttype, Address, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityAction {
    CampaignCreated,
    CampaignFunded,
    CampaignStatusChanged,
    FundsReleased,
    HarvestReported,
    DisputeInitiated,
    DisputeResolved,
    CampaignSettled,
    FarmerRegistered,
    CampaignRegistered,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityRecord {
    pub actor: Address,
    pub action_type: ActivityAction,
    pub timestamp: u64,
    pub ledger_sequence: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CampaignStatus {
    Active,
    Funding,
    Funded,
    Disputed,
    Resolved,
    Settled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignRecord {
    pub campaign_id: u64,
    pub farmer: Address,
    pub escrow_contract: Address,
    pub crop_metadata: Symbol,
    pub region_metadata: Symbol,
    pub status: CampaignStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    ApprovedContract(Address),
    CampaignActivities(u64),
    Campaign(u64),
    FarmerCampaigns(Address),
}
