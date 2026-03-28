# Balance Control

Multi-chain cryptocurrency balance monitor with threshold alerting and Prometheus metrics integration.

## Overview

Balance Control is a Rust-based monitoring service that tracks cryptocurrency balances across multiple blockchain networks (EVM chains, Solana, and Tron). It sends Telegram alerts when balances fall below configured thresholds and exposes metrics for Prometheus monitoring.

## Features

- **Multi-Chain Support**: Monitor balances across Ethereum, BSC, Polygon, Tron, and Solana
- **Native & Token Balances**: Track both native tokens (ETH, BNB, SOL, TRX) and ERC20/TRC20/SPL tokens
- **Threshold Alerts**: Receive Telegram notifications when balances drop below configured thresholds
- **Prometheus Metrics**: Export balance metrics for monitoring and visualization
- **Configurable Check Intervals**: Set custom polling intervals for balance checks
- **Docker Support**: Easy deployment with Docker and Docker Compose

## Architecture

```
┌─────────┐      ┌──────────────────────┐      ┌──────────────┐
│ Config  │─────▶│  Main (Tokio)        │─────▶│ Monitored    │
│ (YAML)  │      │                      │      │ Accounts     │
└─────────┘      └──────────────────────┘      └──────────────┘
                           │
                           ▼
        ┌──────────────────────────────────────┐
        │  BalanceMonitorService               │
        │  (Check Cycle Loop)                  │
        └──────────────────────────────────────┘
                 │              │              │
                 ▼              ▼              ▼
        ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
        │ EVM Provider│  │   Solana    │  │    Tron     │
        │   (Alloy)   │  │  Provider   │  │  Provider   │
        └─────────────┘  └─────────────┘  └─────────────┘
                 │
                 ▼
        ┌─────────────────────┐
        │  Token Registry     │
        │ (Hardcoded Tokens)  │
        └─────────────────────┘

        Infrastructure:
        ┌──────────────────┐  ┌──────────────────┐
        │ Telegram Notifier│  │ Prometheus       │
        │   (teloxide)     │  │ Metrics Server   │
        └──────────────────┘  └──────────────────┘
```

## Supported Networks

- **Ethereum** (and EVM-compatible chains)
- **BSC** (Binance Smart Chain)
- **Polygon**
- **Tron**
- **Solana**

## Supported Tokens

### Ethereum
- USDT: `0xdAC17F958D2ee523a2206206994597C13D831ec7`
- USDC: `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48`
- WETH: `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2`

### BSC
- USDT: `0x55d398326f99059fF775485246999027B3197955`
- USDC: `0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d`

### Polygon
- USDT: `0xc2132D05D31c914a87C6611C10748AEb04B58e8F`
- USDC: `0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359`

### Tron
- USDT: `TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t`
- USDC: `TEkxiTehnzSmSe2XqrBj4w32RUN966rdz8`

### Solana
- USDT: `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB`
- USDC: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`

## Installation

### Prerequisites

- Rust 1.70+ (for building from source)
- Docker & Docker Compose (for containerized deployment)
- Telegram Bot Token (create via [@BotFather](https://t.me/botfather))
- RPC endpoints for the networks you want to monitor

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd balance-control

# Build the project
cargo build --release

# The binary will be available at target/release/balance-control
```

### Using Docker

```bash
# Build the Docker image
docker build -t balance-control .

# Or use docker-compose
docker-compose up -d
```

## Configuration

Create a `config.yaml` file based on `config.example.yaml`:

```yaml
settings:
  check_interval_secs: 60        # How often to check balances
  prometheus_port: 9090          # Port for metrics endpoint
  telegram:
    token: "YOUR_BOT_TOKEN"      # Telegram bot token
    chat_id: 123456789           # Your Telegram chat ID

networks:
  ethereum:
    rpc_url: "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY"
  bsc:
    rpc_url: "https://bsc-dataseed.binance.org"
  tron:
    mode: "http"
    rpc_url: "https://api.trongrid.io"
  solana:
    rpc_url: "https://api.mainnet-beta.solana.com"

accounts:
  - address: "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18"
    alias: "Main_EVM_HotWallet"
    network: "ethereum"
    threshold: 0.5               # Alert if balance < 0.5
    tokens:
      - "native"                 # Native ETH
      - "0xdAC17F958D2ee523a2206206994597C13D831ec7"  # USDT
      - "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"  # USDC
```

### Configuration Options

#### Settings
- `check_interval_secs`: Interval between balance checks (in seconds)
- `prometheus_port`: Port for Prometheus metrics endpoint
- `telegram.token`: Your Telegram bot token
- `telegram.chat_id`: Telegram chat ID for alerts

#### Networks
Define RPC endpoints for each network you want to monitor.

#### Accounts
- `address`: Wallet address to monitor
- `alias`: Human-readable name for the wallet
- `network`: Network name (must match a network defined in `networks`)
- `threshold`: Minimum balance threshold for alerts
- `tokens`: List of tokens to monitor
  - Use `"native"` for native tokens (ETH, BNB, SOL, TRX)
  - Use token contract addresses for ERC20/TRC20/SPL tokens

## Usage

### Running Locally

```bash
# Run with default config.yaml
cargo run --release

# Or specify a custom config file
RUST_LOG=info cargo run --release
```

### Running with Docker

```bash
# Using docker-compose (recommended)
docker-compose up -d

# View logs
docker-compose logs -f balance-control

# Stop the service
docker-compose down
```

### Environment Variables

- `RUST_LOG`: Set logging level (e.g., `info`, `debug`, `warn`)
  ```bash
  RUST_LOG=balance_control=debug cargo run
  ```

## Monitoring

### Prometheus Metrics

The service exposes metrics at `http://localhost:9090/metrics`:

- `balance_control_balance`: Current balance for each account/token combination
  - Labels: `account_alias`, `network`, `token`, `address`

### Grafana Dashboard

You can create a Grafana dashboard to visualize the metrics:

1. Add Prometheus as a data source (default: `http://prometheus:9090`)
2. Create panels with queries like:
   ```promql
   balance_control_balance{account_alias="Main_EVM_HotWallet"}
   ```

### Telegram Alerts

When a balance falls below the configured threshold, you'll receive a Telegram message:

```
⚠️ Balance Alert

Account: Main_EVM_HotWallet
Network: ethereum
Token: USDT
Balance: 45.50
Threshold: 100.00
```

## Development

### Project Structure

```
balance-control/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root
│   ├── config.rs            # Configuration loading
│   ├── tokens.rs            # Token registry
│   ├── domain/              # Domain layer (business logic)
│   │   ├── mod.rs
│   │   ├── model.rs         # Domain models
│   │   ├── ports.rs         # Trait definitions
│   │   └── service.rs       # Balance monitoring service
│   └── infra/               # Infrastructure layer
│       ├── mod.rs
│       ├── evm.rs           # EVM provider (Alloy)
│       ├── solana.rs        # Solana provider
│       ├── tron_http.rs     # Tron HTTP provider
│       ├── telegram.rs      # Telegram notifier
│       └── metrics.rs       # Prometheus metrics
├── tests/                   # Integration tests
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
└── config.example.yaml
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_check_account_below_threshold
```

### Adding New Networks

1. Implement the `BalanceProvider` trait in `src/infra/`
2. Add network type to `NetworkType` enum in `src/domain/model.rs`
3. Initialize the provider in `src/main.rs`
4. Add network configuration to `config.yaml`

### Adding New Tokens

Add token information to `src/tokens.rs`:

```rust
StandardToken {
    symbol: "TOKEN",
    address: "0x...",
    decimals: 18,
    network: "ethereum"
}
```

## Troubleshooting

### Common Issues

1. **RPC Connection Errors**
   - Verify RPC URLs are correct and accessible
   - Check if you need API keys for the RPC providers
   - Ensure network connectivity

2. **Telegram Alerts Not Working**
   - Verify bot token is correct
   - Ensure chat_id is correct (use [@userinfobot](https://t.me/userinfobot) to get your chat ID)
   - Check that the bot has been started (send `/start` to your bot)

3. **Balance Not Updating**
   - Check logs for errors: `docker-compose logs -f balance-control`
   - Verify account addresses are correct
   - Ensure token addresses match the network

4. **High Memory Usage**
   - Reduce `check_interval_secs` to check less frequently
   - Reduce number of monitored accounts/tokens

## Performance

- **Memory**: ~50-100 MB per instance
- **CPU**: Minimal (mostly I/O bound)
- **Network**: Depends on check interval and number of accounts

## Security Considerations

- Store `config.yaml` securely (contains sensitive tokens)
- Use read-only RPC endpoints when possible
- Rotate Telegram bot tokens periodically
- Consider using environment variables for sensitive data
- Run with minimal privileges in production

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]

## Support

For issues and questions:
- Open an issue on GitHub
- [Add contact information]
