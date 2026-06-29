import { Injectable, Logger } from '@nestjs/common';
import { PrismaClient } from '../../../generated/prisma/client';
import type {
  RawSorobanEvent,
  ParsedEvent,
  CampaignCreatedData,
  CampaignInvestedData,
  CampaignSettledData,
} from '../types/soroban-events.types';

@Injectable()
export class EventParserService {
  private readonly logger = new Logger(EventParserService.name);

  constructor(private readonly prisma: PrismaClient) {}

  async processEvent(raw: RawSorobanEvent): Promise<void> {
    try {
      const parsed = this.parseEvent(raw);
      if (!parsed) return;
      await this.persistEvent(parsed);
    } catch (error) {
      this.logger.error(
        { eventId: raw.id, error: error instanceof Error ? error.message : String(error) },
        'Failed to process event',
      );
    }
  }

  private parseEvent(raw: RawSorobanEvent): ParsedEvent | null {
    const topics = raw.topic as unknown[];
    const eventName = String(topics[0] ?? '');
    const value = raw.value;

    switch (eventName) {
      case 'camp_reg':
      case 'CampaignRegistered':
        return this.parseCampaignCreated(raw, topics, value);
      case 'ContribReceived':
        return this.parseCampaignInvested(raw, topics, value);
      case 'CampaignSettled':
        return this.parseCampaignSettled(raw, topics, value);
      default:
        return null;
    }
  }

  private parseCampaignCreated(
    raw: RawSorobanEvent,
    topics: unknown[],
    value: unknown,
  ): ParsedEvent | null {
    const arr = value as unknown as unknown[];
    if (!arr || arr.length < 3) return null;

    const data: CampaignCreatedData = {
      campaignId: String(arr[0] ?? ''),
      farmer: String(arr[1] ?? ''),
      title: String(arr[2] ?? ''),
      timestamp: Number(arr[3] ?? raw.ledger),
    };

    return {
      id: raw.id,
      type: 'campaign.created',
      contractEvent: String(topics[0] ?? ''),
      contractId: raw.contractId ?? '',
      ledger: raw.ledger,
      txHash: raw.txHash,
      data: data as unknown as Record<string, unknown>,
      raw: { topics, value },
    };
  }

  private parseCampaignInvested(
    raw: RawSorobanEvent,
    topics: unknown[],
    value: unknown,
  ): ParsedEvent | null {
    const arr = value as unknown as unknown[];
    if (!arr || arr.length < 3) return null;

    const data: CampaignInvestedData = {
      campaignId: String(topics[1] ?? ''),
      investor: String(arr[0] ?? ''),
      amount: String(arr[2] ?? '0'),
      timestamp: Number(arr[1] ?? raw.ledger),
    };

    return {
      id: raw.id,
      type: 'campaign.invested',
      contractEvent: String(topics[0] ?? ''),
      contractId: raw.contractId ?? '',
      ledger: raw.ledger,
      txHash: raw.txHash,
      data: data as unknown as Record<string, unknown>,
      raw: { topics, value },
    };
  }

  private parseCampaignSettled(
    raw: RawSorobanEvent,
    topics: unknown[],
    value: unknown,
  ): ParsedEvent | null {
    const arr = value as unknown as unknown[];
    if (!arr || arr.length < 4) return null;

    const data: CampaignSettledData = {
      campaignId: String(topics[1] ?? ''),
      farmer: String(arr[0] ?? ''),
      timestamp: Number(arr[1] ?? raw.ledger),
      farmerPayout: String(arr[2] ?? '0'),
      investorReturns: String(arr[3] ?? '0'),
    };

    return {
      id: raw.id,
      type: 'campaign.settled',
      contractEvent: String(topics[0] ?? ''),
      contractId: raw.contractId ?? '',
      ledger: raw.ledger,
      txHash: raw.txHash,
      data: data as unknown as Record<string, unknown>,
      raw: { topics, value },
    };
  }

  private async persistEvent(parsed: ParsedEvent): Promise<void> {
    const existing = await this.prisma.transaction.findUnique({ where: { id: parsed.id } });
    if (existing) return;

    switch (parsed.type) {
      case 'campaign.created':
        await this.handleCampaignCreated(parsed);
        break;
      case 'campaign.invested':
        await this.handleCampaignInvested(parsed);
        break;
      case 'campaign.settled':
        await this.handleCampaignSettled(parsed);
        break;
    }

    await this.prisma.transaction.create({
      data: {
        id: parsed.id,
        type: parsed.type,
        campaignId: (parsed.data as Record<string, unknown>).campaignId as string | undefined,
        userId: (parsed.data as Record<string, unknown>).farmer as string | undefined,
        txHash: parsed.txHash,
        status: 'Confirmed',
        timestamp: BigInt(parsed.ledger),
        data: JSON.stringify(parsed),
      },
    });
  }

  private async handleCampaignCreated(parsed: ParsedEvent): Promise<void> {
    const data = parsed.data as unknown as CampaignCreatedData;
    const campaignId = data.campaignId;

    await this.prisma.user.upsert({
      where: { address: data.farmer },
      update: {},
      create: { address: data.farmer, firstSeenAt: BigInt(data.timestamp) },
    });

    await this.prisma.campaign.upsert({
      where: { id: campaignId },
      update: { status: 'Funding', farmer: data.farmer, title: data.title },
      create: {
        id: campaignId,
        farmer: data.farmer,
        title: data.title,
        createdAt: BigInt(data.timestamp),
      },
    });

    this.logger.log({ campaignId, farmer: data.farmer }, 'Campaign created and persisted');
  }

  private async handleCampaignInvested(parsed: ParsedEvent): Promise<void> {
    const data = parsed.data as unknown as CampaignInvestedData;
    const campaignId = data.campaignId;

    await this.prisma.user.upsert({
      where: { address: data.investor },
      update: {},
      create: { address: data.investor, firstSeenAt: BigInt(data.timestamp) },
    });

    const amount = BigInt(data.amount);

    await this.prisma.investment.create({
      data: {
        id: parsed.id,
        campaignId,
        investor: data.investor,
        amount,
        txHash: parsed.txHash,
        timestamp: BigInt(data.timestamp),
      },
    });

    await this.prisma.campaign.update({
      where: { id: campaignId },
      data: { totalFunded: { increment: amount } },
    });

    this.logger.log({ campaignId, investor: data.investor, amount: data.amount }, 'Investment recorded');
  }

  private async handleCampaignSettled(parsed: ParsedEvent): Promise<void> {
    const data = parsed.data as unknown as CampaignSettledData;
    const campaignId = data.campaignId;

    await this.prisma.campaign.update({
      where: { id: campaignId },
      data: { status: 'Settled' },
    });

    this.logger.log({ campaignId, farmerPayout: data.farmerPayout }, 'Campaign settled');
  }
}
