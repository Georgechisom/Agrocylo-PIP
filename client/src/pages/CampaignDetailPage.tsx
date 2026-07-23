import { useState } from 'react';
import type { FormEvent, ReactNode } from 'react';
import { Link, useParams } from 'react-router-dom';
import Header from '../components/Header';
import { StatusBadge } from '../components/campaign/StatusBadge';
import { LifecycleStepper } from '../components/campaign/LifecycleStepper';
import { useWallet, truncateAddress } from '../context/WalletContext';
import {
  useActivity,
  useCampaign,
  useContribution,
  useDispute,
  useFundCampaign,
  useHarvestRecord,
  useOpenDispute,
  useReportHarvest,
  useTranches,
} from '../hooks/contract';
import { formatContractAmount, formatLedgerTimestamp } from '../lib/format';
import {
  isEscrowConfigured,
  isRegistryConfigured,
  explorerAddressUrl,
} from '../lib/soroban/config';
import { describeContractError } from '../lib/soroban/contractClient';
import {
  CONTRACT_SYMBOL_HINT,
  isValidContractSymbol,
} from '../lib/soroban/symbol';

const CAMPAIGN_ID_PATTERN = /^\d+$/;
const FUNDABLE_STATUSES = ['Active', 'Funding'];
const DISPUTABLE_STATUSES = ['Active', 'Funding', 'Funded'];

const cardClass =
  'rounded-campaign border border-soil-200 bg-white p-5 shadow-campaign sm:p-6';
const primaryButtonClass =
  'inline-flex items-center justify-center rounded-lg bg-leaf-700 px-4 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-leaf-800 focus:outline-none focus-visible:ring-2 focus-visible:ring-leaf-500 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50';
const inputClass =
  'w-full rounded-lg border border-soil-300 px-3 py-2 text-body-sm text-soil-900 focus:border-leaf-500 focus:outline-none focus:ring-1 focus:ring-leaf-500';

function InfoRow({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 py-2 text-body-sm">
      <span className="text-soil-500">{label}</span>
      <span className="font-medium text-soil-900">{value}</span>
    </div>
  );
}

function AddressLink({ address }: { address: string }) {
  return (
    <a
      href={explorerAddressUrl(address)}
      target="_blank"
      rel="noreferrer"
      className="font-mono text-body-sm text-leaf-700 hover:text-leaf-800 hover:underline"
    >
      {truncateAddress(address)}
    </a>
  );
}

function NotConfiguredNotice() {
  return (
    <div className="mb-6 rounded-campaign border border-amber-300 bg-amber-50 p-5 sm:p-6">
      <h2 className="text-h4 text-amber-900">Soroban RPC not configured</h2>
      <p className="mt-2 text-body-sm text-amber-800">
        Set{' '}
        <code className="rounded bg-amber-100 px-1 py-0.5 font-mono text-xs">
          VITE_SOROBAN_RPC_URL
        </code>
        ,{' '}
        <code className="rounded bg-amber-100 px-1 py-0.5 font-mono text-xs">
          VITE_PRODUCTION_ESCROW_CONTRACT_ID
        </code>
        , and{' '}
        <code className="rounded bg-amber-100 px-1 py-0.5 font-mono text-xs">
          VITE_REGISTRY_CONTRACT_ID
        </code>{' '}
        (see{' '}
        <code className="rounded bg-amber-100 px-1 py-0.5 font-mono text-xs">
          client/.env.example
        </code>
        ) to load live campaign data.
      </p>
    </div>
  );
}

function NotFoundNotice({
  campaignId,
  error,
}: {
  campaignId: string | undefined;
  error?: unknown;
}) {
  return (
    <div className={`${cardClass} text-center`}>
      <h1 className="text-h3 text-soil-950">Campaign not found</h1>
      <p className="mt-2 text-body-sm text-soil-500">
        {campaignId
          ? `We couldn't find a campaign with id "${campaignId}" on the configured ProductionEscrowContract.`
          : 'No campaign id was provided.'}
      </p>
      {error !== undefined && (
        <p className="mt-2 text-caption text-soil-400">
          {describeContractError(error)}
        </p>
      )}
      <Link
        to="/campaigns"
        className={`${primaryButtonClass} mt-5 inline-flex`}
      >
        Back to campaigns
      </Link>
    </div>
  );
}

function DetailSkeleton() {
  return (
    <div className="space-y-6">
      <div className="h-24 animate-pulse rounded-campaign border border-soil-200 bg-soil-50" />
      <div className="grid gap-6 lg:grid-cols-3">
        <div className="h-64 animate-pulse rounded-campaign border border-soil-200 bg-soil-50 lg:col-span-2" />
        <div className="h-64 animate-pulse rounded-campaign border border-soil-200 bg-soil-50" />
      </div>
    </div>
  );
}

function FundForm({ campaignId }: { campaignId: string }) {
  const wallet = useWallet();
  const mutation = useFundCampaign();
  const [amount, setAmount] = useState('');

  const amountError =
    amount.length > 0 && (!/^\d+$/.test(amount) || BigInt(amount) <= 0n)
      ? 'Enter a whole number greater than zero.'
      : null;

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    if (!wallet.publicKey || amountError || amount.length === 0) return;
    mutation.mutate({
      campaignId,
      investor: wallet.publicKey,
      amount: BigInt(amount),
    });
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-3">
      <h3 className="text-h4 text-soil-900">Fund this campaign</h3>
      <div>
        <label
          htmlFor="fund-amount"
          className="mb-1 block text-label text-soil-500"
        >
          Amount (contract units)
        </label>
        <input
          id="fund-amount"
          inputMode="numeric"
          className={inputClass}
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="1000"
        />
        {amountError && (
          <p className="mt-1 text-caption text-status-failed-dark">
            {amountError}
          </p>
        )}
      </div>
      <button
        type="submit"
        disabled={
          mutation.isPending || amount.length === 0 || Boolean(amountError)
        }
        className={primaryButtonClass}
      >
        {mutation.isPending ? 'Confirm in wallet…' : 'Fund campaign'}
      </button>
      {mutation.isError && (
        <p className="text-body-sm text-status-failed-dark">
          {describeContractError(mutation.error)}
        </p>
      )}
      {mutation.isSuccess && (
        <p className="text-body-sm text-status-active-dark">
          Contribution confirmed. Funding progress will update shortly.
        </p>
      )}
    </form>
  );
}

function ReportHarvestForm({ campaignId }: { campaignId: string }) {
  const wallet = useWallet();
  const mutation = useReportHarvest();
  const [outcome, setOutcome] = useState('');

  const outcomeError =
    outcome.length > 0 && !isValidContractSymbol(outcome)
      ? CONTRACT_SYMBOL_HINT
      : null;

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    if (!wallet.publicKey || outcomeError || outcome.length === 0) return;
    mutation.mutate({ campaignId, farmer: wallet.publicKey, outcome });
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-3">
      <h3 className="text-h4 text-soil-900">Report harvest</h3>
      <div>
        <label
          htmlFor="harvest-outcome"
          className="mb-1 block text-label text-soil-500"
        >
          Outcome
        </label>
        <input
          id="harvest-outcome"
          className={inputClass}
          value={outcome}
          onChange={(e) => setOutcome(e.target.value)}
          placeholder="successful_yield"
        />
        <p className="mt-1 text-caption text-soil-400">
          {outcomeError ?? CONTRACT_SYMBOL_HINT}
        </p>
      </div>
      <button
        type="submit"
        disabled={
          mutation.isPending || outcome.length === 0 || Boolean(outcomeError)
        }
        className={primaryButtonClass}
      >
        {mutation.isPending ? 'Confirm in wallet…' : 'Report harvest'}
      </button>
      {mutation.isError && (
        <p className="text-body-sm text-status-failed-dark">
          {describeContractError(mutation.error)}
        </p>
      )}
      {mutation.isSuccess && (
        <p className="text-body-sm text-status-active-dark">
          Harvest reported. The campaign status will update shortly.
        </p>
      )}
    </form>
  );
}

function OpenDisputeForm({ campaignId }: { campaignId: string }) {
  const wallet = useWallet();
  const mutation = useOpenDispute();
  const [open, setOpen] = useState(false);
  const [reason, setReason] = useState('');

  const reasonError =
    reason.length > 0 && !isValidContractSymbol(reason)
      ? CONTRACT_SYMBOL_HINT
      : null;

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    if (!wallet.publicKey || reasonError || reason.length === 0) return;
    mutation.mutate({ campaignId, opener: wallet.publicKey, reason });
  }

  if (!open) {
    return (
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="text-body-sm font-semibold text-status-disputed-dark hover:underline"
      >
        Open a dispute
      </button>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-3">
      <h3 className="text-h4 text-soil-900">Open a dispute</h3>
      <div>
        <label
          htmlFor="dispute-reason"
          className="mb-1 block text-label text-soil-500"
        >
          Reason
        </label>
        <input
          id="dispute-reason"
          className={inputClass}
          value={reason}
          onChange={(e) => setReason(e.target.value)}
          placeholder="missed_deadline"
        />
        <p className="mt-1 text-caption text-soil-400">
          {reasonError ?? CONTRACT_SYMBOL_HINT}
        </p>
      </div>
      <div className="flex gap-2">
        <button
          type="submit"
          disabled={
            mutation.isPending || reason.length === 0 || Boolean(reasonError)
          }
          className="inline-flex items-center justify-center rounded-lg bg-status-disputed px-4 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-status-disputed-dark disabled:cursor-not-allowed disabled:opacity-50"
        >
          {mutation.isPending ? 'Confirm in wallet…' : 'Submit dispute'}
        </button>
        <button
          type="button"
          onClick={() => setOpen(false)}
          className="rounded-lg border border-soil-300 px-4 py-2.5 text-sm font-semibold text-soil-700 hover:bg-soil-50"
        >
          Cancel
        </button>
      </div>
      {mutation.isError && (
        <p className="text-body-sm text-status-failed-dark">
          {describeContractError(mutation.error)}
        </p>
      )}
    </form>
  );
}

function ActionPanel({
  campaignId,
  farmer,
  status,
}: {
  campaignId: string;
  farmer: string;
  status: string;
}) {
  const wallet = useWallet();
  const contribution = useContribution(
    campaignId,
    wallet.publicKey ?? undefined,
  );

  if (!wallet.isConnected) {
    return (
      <div className={cardClass}>
        <h2 className="text-h4 text-soil-900">Actions</h2>
        <p className="mt-2 text-body-sm text-soil-500">
          Connect your wallet to fund this campaign or take farmer/investor
          actions.
        </p>
      </div>
    );
  }

  const isFarmer = wallet.publicKey === farmer;
  const hasContributed = (contribution.data ?? 0n) > 0n;
  const canFund = FUNDABLE_STATUSES.includes(status);
  const canReportHarvest = isFarmer && status === 'Funded';
  const canDispute =
    (isFarmer || hasContributed) && DISPUTABLE_STATUSES.includes(status);

  if (!canFund && !canReportHarvest && !canDispute) {
    return (
      <div className={cardClass}>
        <h2 className="text-h4 text-soil-900">Actions</h2>
        <p className="mt-2 text-body-sm text-soil-500">
          No actions are available for your wallet at this campaign stage.
        </p>
      </div>
    );
  }

  return (
    <div className={`${cardClass} space-y-6`}>
      <h2 className="text-h4 text-soil-900">Actions</h2>
      {canFund && <FundForm campaignId={campaignId} />}
      {canReportHarvest && <ReportHarvestForm campaignId={campaignId} />}
      {canDispute && <OpenDisputeForm campaignId={campaignId} />}
    </div>
  );
}

export function CampaignDetailPage() {
  const { campaignId } = useParams();
  const isValidId = Boolean(campaignId && CAMPAIGN_ID_PATTERN.test(campaignId));
  const configured = isEscrowConfigured() && isRegistryConfigured();

  const campaign = useCampaign(isValidId ? campaignId : undefined);
  const status = campaign.data?.status.tag;
  const tranches = useTranches(isValidId ? campaignId : undefined);
  const harvestRecord = useHarvestRecord(isValidId ? campaignId : undefined, {
    enabled: status === 'Harvested' || status === 'Settled',
  });
  const dispute = useDispute(isValidId ? campaignId : undefined, {
    enabled: status === 'Disputed' || status === 'Resolved',
  });
  const activity = useActivity(isValidId ? campaignId : undefined);

  return (
    <section className="mx-auto max-w-7xl px-4 py-12 sm:px-6 sm:py-16 lg:px-8">
      <Header />

      <div className="mb-8 mt-6">
        <Link
          to="/campaigns"
          className="text-body-sm font-semibold text-leaf-700 hover:text-leaf-800"
        >
          ← Back to campaigns
        </Link>
      </div>

      {!configured && <NotConfiguredNotice />}

      {(!isValidId || (configured && campaign.isError)) && (
        <NotFoundNotice campaignId={campaignId} error={campaign.error} />
      )}

      {isValidId && configured && campaign.isLoading && <DetailSkeleton />}

      {isValidId && configured && campaign.isSuccess && campaign.data && (
        <div className="space-y-6">
          <div className={cardClass}>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <p className="text-label text-leaf-700">
                  Campaign #{campaignId}
                </p>
                <h1 className="mt-1 text-soil-950">
                  {campaign.data.harvest_metadata}
                </h1>
              </div>
              <StatusBadge status={campaign.data.status.tag} />
            </div>
            <div className="mt-6">
              <LifecycleStepper status={campaign.data.status.tag} />
            </div>
          </div>

          <div className="grid gap-6 lg:grid-cols-3">
            <div className="space-y-6 lg:col-span-2">
              <div className={cardClass}>
                <h2 className="text-h4 text-soil-900">Funding progress</h2>
                <div className="mt-3">
                  <div className="h-2 overflow-hidden rounded-full bg-soil-100">
                    <div
                      className="h-full rounded-full bg-leaf-600"
                      style={{
                        width: `${Math.min(
                          100,
                          campaign.data.target_amount > 0n
                            ? Number(
                                (campaign.data.total_funded * 100n) /
                                  campaign.data.target_amount,
                              )
                            : 0,
                        )}%`,
                      }}
                    />
                  </div>
                  <p className="mt-2 text-body-sm text-soil-600">
                    {formatContractAmount(campaign.data.total_funded)} /{' '}
                    {formatContractAmount(campaign.data.target_amount)} raised
                  </p>
                </div>
                <div className="mt-4 divide-y divide-soil-100 border-t border-soil-100">
                  <InfoRow
                    label="Farmer"
                    value={<AddressLink address={campaign.data.farmer} />}
                  />
                  <InfoRow
                    label="Token"
                    value={
                      <AddressLink address={campaign.data.token_address} />
                    }
                  />
                  <InfoRow
                    label="Deadline"
                    value={formatLedgerTimestamp(campaign.data.deadline)}
                  />
                  <InfoRow
                    label="Released"
                    value={formatContractAmount(campaign.data.released)}
                  />
                  <InfoRow
                    label="Refundable"
                    value={formatContractAmount(campaign.data.refundable)}
                  />
                  <InfoRow
                    label="Returnable"
                    value={formatContractAmount(campaign.data.returnable)}
                  />
                </div>
              </div>

              <div className={cardClass}>
                <h2 className="text-h4 text-soil-900">Tranches</h2>
                {tranches.isLoading && (
                  <p className="mt-2 text-body-sm text-soil-500">Loading…</p>
                )}
                {tranches.isSuccess && tranches.data.length === 0 && (
                  <p className="mt-2 text-body-sm text-soil-500">
                    No tranches configured yet.
                  </p>
                )}
                {tranches.isSuccess && tranches.data.length > 0 && (
                  <ul className="mt-3 divide-y divide-soil-100 border-t border-soil-100">
                    {tranches.data.map((tranche, index) => (
                      <li
                        key={`${tranche.milestone}-${index}`}
                        className="flex items-center justify-between gap-4 py-2 text-body-sm"
                      >
                        <span className="text-soil-700">
                          {tranche.milestone}
                        </span>
                        <span className="flex items-center gap-2">
                          <span className="font-medium text-soil-900">
                            {formatContractAmount(tranche.amount)}
                          </span>
                          <span
                            className={`status-badge ${tranche.released ? 'bg-status-active-light text-status-active-dark' : 'bg-soil-100 text-soil-500'}`}
                          >
                            {tranche.released ? 'Released' : 'Pending'}
                          </span>
                        </span>
                      </li>
                    ))}
                  </ul>
                )}
              </div>

              <div className={cardClass}>
                <h2 className="text-h4 text-soil-900">Harvest</h2>
                {harvestRecord.isLoading && (
                  <p className="mt-2 text-body-sm text-soil-500">Loading…</p>
                )}
                {harvestRecord.isSuccess && harvestRecord.data && (
                  <div className="mt-3 divide-y divide-soil-100 border-t border-soil-100">
                    <InfoRow
                      label="Outcome"
                      value={harvestRecord.data.outcome}
                    />
                    <InfoRow
                      label="Reported"
                      value={formatLedgerTimestamp(
                        harvestRecord.data.timestamp,
                      )}
                    />
                  </div>
                )}
                {(harvestRecord.isError ||
                  (status !== 'Harvested' && status !== 'Settled')) && (
                  <p className="mt-2 text-body-sm text-soil-500">
                    No harvest reported yet.
                  </p>
                )}
              </div>

              {(status === 'Disputed' || status === 'Resolved') && (
                <div className={cardClass}>
                  <h2 className="text-h4 text-soil-900">Dispute</h2>
                  {dispute.isLoading && (
                    <p className="mt-2 text-body-sm text-soil-500">Loading…</p>
                  )}
                  {dispute.isSuccess && dispute.data && (
                    <div className="mt-3 divide-y divide-soil-100 border-t border-soil-100">
                      <InfoRow
                        label="Opened by"
                        value={<AddressLink address={dispute.data.opener} />}
                      />
                      <InfoRow label="Reason" value={dispute.data.reason} />
                      <InfoRow label="Status" value={dispute.data.status.tag} />
                      <InfoRow
                        label="Resolution"
                        value={dispute.data.resolution.tag}
                      />
                    </div>
                  )}
                </div>
              )}

              <div className={cardClass}>
                <h2 className="text-h4 text-soil-900">Activity</h2>
                {activity.isLoading && (
                  <p className="mt-2 text-body-sm text-soil-500">Loading…</p>
                )}
                {activity.isSuccess && activity.data.length === 0 && (
                  <p className="mt-2 text-body-sm text-soil-500">
                    No recorded activity yet.
                  </p>
                )}
                {activity.isSuccess && activity.data.length > 0 && (
                  <ul className="mt-3 divide-y divide-soil-100 border-t border-soil-100">
                    {activity.data.map((record, index) => (
                      <li
                        key={index}
                        className="flex items-center justify-between gap-4 py-2 text-body-sm"
                      >
                        <span className="text-soil-700">
                          {record.action_type.tag}
                        </span>
                        <span className="text-soil-400">
                          {formatLedgerTimestamp(record.timestamp)}
                        </span>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            </div>

            <div>
              <ActionPanel
                campaignId={campaignId!}
                farmer={campaign.data.farmer}
                status={campaign.data.status.tag}
              />
            </div>
          </div>
        </div>
      )}
    </section>
  );
}
