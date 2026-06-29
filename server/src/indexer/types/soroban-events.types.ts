export interface RawSorobanEvent {
  id: string;
  type: string;
  contractId?: string;
  topic: unknown[];
  value: unknown;
  ledger: number;
  ledgerClosedAt: string;
  txHash: string;
}

export interface ParsedEvent {
  /** Unique event ID from Soroban */
  id: string;
  /** High-level event type name (campaign.created, campaign.invested, etc.) */
  type: string;
  /** Contract event name (CampaignCreated, ContribReceived, etc.) */
  contractEvent: string;
  contractId: string;
  ledger: number;
  txHash: string;
  /** Parsed, human-friendly payload */
  data: Record<string, unknown>;
  raw: Record<string, unknown>;
}

export interface CampaignCreatedData {
  campaignId: string;
  farmer: string;
  title: string;
  timestamp: number;
}

export interface CampaignInvestedData {
  campaignId: string;
  investor: string;
  amount: string;
  timestamp: number;
}

export interface CampaignSettledData {
  campaignId: string;
  farmer: string;
  timestamp: number;
  farmerPayout: string;
  investorReturns: string;
}

export interface OrderCreatedData {
  orderId: string;
  campaignId: string;
  buyer: string;
  amount: string;
  timestamp: number;
}

export interface OrderConfirmedData {
  orderId: string;
  campaignId: string;
  buyer: string;
  timestamp: number;
}

export type ParsedEventData =
  | CampaignCreatedData
  | CampaignInvestedData
  | CampaignSettledData
  | OrderCreatedData
  | OrderConfirmedData;
