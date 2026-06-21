# Agrocylo Smart Contracts

Soroban smart contracts for the Agrocylo Production Investment Platform.

## Contracts

### ProductionEscrowContract

The ProductionEscrowContract manages the escrow lifecycle of agricultural production campaigns.

**Features:**

- Campaign creation with funding goals and deadlines
- Multi-investor funding tracking
- Campaign lifecycle state machine (Funding → Funded → InProduction → Harvested → Settled)
- Alternative terminal states: Failed (refund) and Disputed
- Event emission for state transitions

**Campaign States:**

- `Funding` — campaign created, accepting funds
- `Funded` — funding goal reached
- `InProduction` — production phase active
- `Harvested` — harvest reported by farmer
- `Settled` — funds distributed
- `Failed` — campaign failed, refunds processed
- `Disputed` — dispute in progress

### Registry Contract

The Registry Contract manages campaign activity records and access control for the platform.

**Features:**

- Activity record logging for campaign events
- Admin configuration and management
- Access control for authorized contracts
- Event emission for indexing services

**Activity Records:**

Activity records track important campaign lifecycle events including:
- Farmer registration
- Campaign registration
- Campaign creation
- Funding events
- Status changes
- Fund releases
- Harvest reports
- Dispute initiation and resolution
- Campaign settlement

**Access Control:**

- Admin-only operations require authorization from the current admin address
- Approved contracts can perform registry operations without additional authorization
- Activity records can be created by admin, approved contracts, or authorized users

**Registry Events:**

Registry events use stable, descriptive names so frontend, backend, and analytics services can index the same stream consistently.

| Event | Topics | Payload |
| --- | --- | --- |
| `AdminInitialized` | `(event_name, admin)` | `(admin, timestamp, ledger_sequence)` |
| `AdminUpdated` | `(event_name, new_admin)` | `(actor, old_admin, new_admin, timestamp, ledger_sequence)` |
| `ContractApproved` | `(event_name, contract)` | `(actor, contract, timestamp, ledger_sequence)` |
| `ContractRevoked` | `(event_name, contract)` | `(actor, contract, timestamp, ledger_sequence)` |
| `FarmerRegistered` | `(event_name, farmer)` | `(actor, timestamp, ledger_sequence)` |
| `CampaignRegistered` | `(event_name, campaign_id)` | `(actor, action_type, timestamp, ledger_sequence)` |
| `CampaignStatusUpdated` | `(event_name, campaign_id)` | `(actor, action_type, timestamp, ledger_sequence)` |
| `ActivityRecorded` | `(event_name, campaign_id)` | `(actor, action_type, timestamp, ledger_sequence)` |

## Building

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Testing

```bash
cargo test
```

## Development

### Prerequisites

- Rust toolchain with wasm32-unknown-unknown target
- Soroban CLI tools

### Project Structure

```
contracts/
├── production_escrow/  # Production escrow contract
│   ├── src/
│   │   ├── lib.rs     # Contract entry point
│   │   ├── types.rs   # CampaignStatus enum, Campaign struct, DataKey
│   │   ├── storage.rs # Storage utilities
│   │   └── test.rs    # Test suite
│   └── Cargo.toml
├── registry/           # Registry contract implementation
│   ├── src/
│   │   ├── lib.rs     # Contract entry point
│   │   ├── types.rs   # Data types and enums
│   │   ├── storage.rs # Storage utilities
│   │   ├── admin.rs   # Admin access control
│   │   ├── activity.rs # Activity logging
│   │   ├── events.rs  # Event definitions
│   │   └── test.rs    # Test suite
│   └── Cargo.toml
└── Cargo.toml         # Workspace configuration
```

## License

This project is open source.
