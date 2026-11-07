 ⚓️ AnchorFlow

**Real-time fiat corridor health monitoring and smart routing for Stellar anchors.**

This workspace currently contains Soroban contracts. The monitoring engine,
REST API, database, and dashboard described below are planned platform
components and are not included in this repository yet.

AnchorFlow continuously probes anchors across every registered corridor, aggregates live fee and reliability data, and exposes a routing API that always returns the best available anchor for a given asset pair and destination country. A Soroban smart contract acts as the on-chain source of truth — health attestations are written every epoch and queryable by any contract on Stellar, with no trust in AnchorFlow's server required.

> Organization: **StellarForge** — forging the infrastructure layer of the Stellar ecosystem.  
> License: MIT · Built on Stellar

---

## The Problem

Stellar's anchor network enables fiat settlement across 60+ countries, but it is largely unmonitored. Developers often hardcode one anchor per corridor and have little visibility when a service degrades. SEP-24 flows can silently fail, fee estimates drift over time, and there is no public infrastructure that tells developers or end users:

- Whether a corridor is operational
- Which anchor currently offers the best route
- What the real cost of a transfer is
- How reliable a corridor has been historically
- Which fallback route should be used when an anchor is unavailable

AnchorFlow solves this visibility gap.

---

## What AnchorFlow Does

| Without AnchorFlow | With AnchorFlow |
|-------------------|-----------------|
| HTTP 503 errors with no fallback | Automatic rerouting to the healthiest anchor |
| Fee data becomes outdated | Live normalized fee monitoring |
| One hardcoded anchor per corridor | Dynamic routing based on health scores |
| No reliability history | On-chain audit trail of anchor performance |
| Corridor outages discovered by users | Continuous proactive monitoring |

---

## Key Features

### ⚡ Real-Time Anchor Monitoring

- Continuous SEP-6, SEP-24, and SEP-31 health checks
- Latency measurement
- Uptime tracking
- Error-rate monitoring
- Historical reliability scoring

### 🧭 Smart Routing Engine

- Automatic anchor selection
- Health-based route optimization
- Fee-aware routing decisions
- Automatic fallback support
- Country and asset-specific routing

### ⛓️ On-Chain Health Attestations

- Health scores written to Soroban every epoch
- Publicly verifiable monitoring data
- Immutable reliability history
- Queryable by any Soroban contract

### 📊 Developer Dashboard

- Live corridor coverage map
- Fee comparison tools
- Health timelines
- Routing API playground
- Embeddable status widgets

---

## Architecture

```text
Stellar Horizon + SEP Servers
  (stellar.toml discovery · SEP-6/24/31 endpoints)
          │
          │ polled every 60s
          ▼
Prober + Health Engine (Node.js)
  ├─ Synthetic SEP-24 probes
  ├─ Latency measurement
  ├─ Error tracking
  ├─ Uptime monitoring
  ├─ Fee normalization
  ├─ Health score calculation
  └─ PostgreSQL time-series storage
          │
          │ REST API
          ▼
React Dashboard (Vite + TanStack Query)
  ├─ Live corridor map
  ├─ Anchor health analytics
  ├─ Routing playground
  └─ Embeddable status widgets
          ▲
          │
Soroban Smart Contract (Rust)
  ├─ Anchor Registry
  ├─ Health Attestations
  ├─ Routing Hints
  └─ Public Verification Layer
```

---

## Soroban Contract

The Soroban contract serves as the trust-minimized source of truth for anchor health data.

### Core Components

#### Anchor Registry

Stores registered anchors and supported corridors.

#### Health Attestations

Stores signed health scores submitted every monitoring epoch.

#### Routing Hints

Provides route recommendations directly on-chain.

---

## Soroban Contract API

| Function | Description |
|----------|-------------|
| `initialize(admin)` | Initialize the registry with its administrator |
| `register_anchor(admin, anchor_id, metadata)` | Register an anchor and its authorized health reporter |
| `get_anchor(anchor_id)` | Retrieve registered anchor metadata |
| `attest_health(reporter, anchor_id, score, timestamp)` | Record an authenticated health score from 0 to 100 |
| `get_health(anchor_id)` | Retrieve the latest health attestation |
| `get_attestation_count(anchor_id)` | Retrieve the number of attestations |
| `health_history(anchor_id, from, limit)` | Retrieve a page of history by zero-based sequence; page limit is 50 |

Only the registry administrator can register anchors. The reporter assigned to
an anchor must authorize each attestation; timestamps must not be in the future
or move backward. Historical records are append-only and paginated by sequence.
Route selection and asset-based anchor queries remain roadmap items.

---

## REST API

### Anchors

| Endpoint | Description |
|----------|-------------|
| `GET /api/anchors` | List monitored anchors |
| `GET /api/anchors/:id/health` | Health history and metrics |
| `POST /api/anchors/register` | Register a new anchor |

### Routing

| Endpoint | Description |
|----------|-------------|
| `POST /api/route` | Get optimal route |
| `GET /api/corridors` | Corridor coverage information |

### Fees

| Endpoint | Description |
|----------|-------------|
| `GET /api/fees?asset=&country=` | Compare live fees across anchors |

---

## Frontend Pages

### `/`
Global dashboard showing:

- Active corridors
- Network health
- Coverage statistics
- Anchor rankings

### `/anchors/:id`

Displays:

- Health score history
- Latency charts
- Fee trends
- Supported corridors

### `/route`

Interactive routing sandbox:

- Select asset
- Choose destination country
- View recommended anchor
- Compare fees

### `/corridors`

Coverage matrix displaying:

- Active routes
- Available anchors
- Reliability metrics

### `/embed`

Generate embeddable status badges for:

- Wallets
- Exchanges
- Payment applications
- Stellar-based platforms

---

## SEP Standards Supported

### SEP-6

Hosted deposit and withdrawal monitoring.

### SEP-24

Interactive deposit and withdrawal monitoring using synthetic transaction probes.

### SEP-31

Cross-border payment corridor monitoring.

### SEP-41

Asset event decoding and token metadata normalization.

---

## Technology Stack

| Layer | Technology |
|---------|------------|
| Blockchain | Stellar |
| Smart Contracts | Soroban (Rust) |
| Monitoring Engine | Node.js |
| Database | PostgreSQL |
| Frontend | React |
| Build Tool | Vite |
| Data Fetching | TanStack Query |
| APIs | REST |
| Standards | SEP-6, SEP-24, SEP-31, SEP-41 |

---

## Quick Start

### Prerequisites

- Rust
- Stellar CLI
- Node.js 20+
- PostgreSQL

### Clone Repository

```bash
git clone https://github.com/stellarforge/anchorflow
cd anchorflow
```

### Configure Environment

```bash
cp .env.example .env
```

Configure:

```env
HORIZON_URL=
DATABASE_URL=
SOROBAN_RPC_URL=
ANCHORFLOW_CONTRACT_ID=
```

---

## Build and Deploy

### Build Contract

```bash
make build
```

### Deploy Contract

```bash
make deploy
```

After deployment, copy the generated contract ID into:

```env
ANCHORFLOW_CONTRACT_ID=
```

---

## Start Services

### Monitoring Engine

```bash
make engine-install
make engine
```

### Frontend Dashboard

```bash
make frontend-install
make frontend
```

Open:

```text
http://localhost:5174
```

### Run Everything

```bash
make install
make dev
```

---

## Validated Need

As of June 2026, there is no public infrastructure dedicated to monitoring Stellar anchor health, corridor availability, and real-time routing quality.

Developers frequently encounter:

- Silent corridor failures
- Unexpected fee changes
- Degraded SEP-24 experiences
- Missing fallback routes
- Limited visibility into anchor reliability

AnchorFlow provides the missing observability layer for the Stellar ecosystem by making anchor performance transparent, measurable, and verifiable.

Just as blockchain ecosystems rely on price oracles for trustworthy market data, Stellar applications need trustworthy infrastructure data to route payments effectively.

---

## Future Roadmap

### Phase 1

- Anchor monitoring engine
- Health scoring system
- REST API
- Dashboard

### Phase 2

- Soroban health attestations
- Public corridor explorer
- Historical analytics

### Phase 3

- Decentralized attestation network
- Reputation system
- Multi-region monitoring nodes

### Phase 4

- Predictive corridor health
- AI-assisted routing recommendations
- Enterprise SLA monitoring

---

## Contributing

We welcome contributions from developers, infrastructure operators, and members of the Stellar ecosystem.

```bash
git clone https://github.com/stellarforge/anchorflow
cd anchorflow
make install
make dev
```

### Contribution Areas

- Monitoring infrastructure
- Soroban smart contracts
- Frontend development
- API development
- Documentation
- Testing
- DevOps

Please open an issue before starting major work and include tests for all new functionality.

---

## License

MIT License

---

### Built by StellarForge ⚒️

Forging the infrastructure layer of the Stellar ecosystem.
