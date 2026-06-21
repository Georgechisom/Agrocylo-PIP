use soroban_sdk::{contracttype, Address, String};

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
pub struct FarmerProfile {
    pub address: Address,
    pub name: String,
    pub location: String,
    pub registration_time: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignInfo {
    pub id: u64,
    pub farmer: Address,
    pub title: String,
    pub description: String,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    ApprovedContract(Address),
    CampaignActivities(u64),
    Farmer(Address),
    Campaign(u64),
    FarmerCount,
    CampaignCount,
}
