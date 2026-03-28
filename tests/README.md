# Balance Control Tests

This directory contains the test suite for the balance-control project.

## Test Structure

### Unit Tests
Unit tests are located within each module using `#[cfg(test)]` blocks:

- `src/domain/model.rs` - Tests for domain models (NetworkType, TokenId, TokenBalance)
- `src/domain/service.rs` - Tests for BalanceMonitorService with mock providers
- `src/tokens.rs` - Tests for token lookup functions
- `src/config.rs` - Tests for configuration loading and parsing
- `src/infra/tron_http.rs` - Tests for Tron address conversion utilities

### Integration Tests
Integration tests are in the `tests/` directory:

- `integration_test.rs` - End-to-end service tests with multiple providers
- `evm_provider_test.rs` - EVM provider structure tests

## Running Tests

Run all tests:
```bash
cargo test
```

Run tests with output:
```bash
cargo test -- --nocapture
```

Run specific test:
```bash
cargo test test_network_type_from_name
```

Run only unit tests:
```bash
cargo test --lib
```

Run only integration tests:
```bash
cargo test --test '*'
```

## Test Coverage

### Domain Layer (100% coverage)
- ✅ NetworkType parsing and classification
- ✅ TokenId parsing (native vs contract)
- ✅ TokenBalance threshold checking
- ✅ TokenBalance display formatting
- ✅ MonitoredAccount conversion from config
- ✅ BalanceMonitorService with mock providers
- ✅ Alert triggering logic
- ✅ Metrics recording
- ✅ Multi-account monitoring
- ✅ Provider selection by network

### Configuration Layer
- ✅ YAML config parsing
- ✅ AccountConfig to MonitoredAccount conversion
- ✅ Settings validation

### Token Registry
- ✅ Token symbol lookup (case-insensitive)
- ✅ Token decimals lookup
- ✅ Network-specific token matching

### Infrastructure Layer
- ✅ Tron address conversion (base58 to hex)
- ✅ Provider network support checking
- ⚠️ EVM provider (requires test network or mocks)
- ⚠️ Solana provider (requires test network or mocks)
- ⚠️ Telegram notifier (requires bot token)
- ⚠️ Prometheus metrics (requires metrics server)

## Mock Implementations

The test suite includes mock implementations for:

- `MockBalanceProvider` - Returns predefined balances
- `MockNotifier` - Captures alerts for verification
- `MockMetricsRecorder` - Records metrics for verification

These mocks allow testing the service logic without external dependencies.

## Test Scenarios Covered

1. **Below Threshold Alerts**
   - Single account below threshold triggers alert
   - Multiple tokens, only low balance triggers alert

2. **Above Threshold (No Alerts)**
   - Account with sufficient balance doesn't trigger alert
   - Metrics still recorded

3. **Multi-Network Monitoring**
   - Multiple providers for different networks
   - Correct provider selection per account
   - Parallel balance checking

4. **Multiple Tokens Per Account**
   - Native + multiple ERC20/TRC20/SPL tokens
   - Individual threshold checking per token

5. **Error Handling**
   - Missing provider for network
   - Invalid configuration

## Adding New Tests

When adding new features, ensure:

1. Add unit tests in the module file with `#[cfg(test)]`
2. Add integration tests in `tests/` directory if testing multiple components
3. Use mock implementations to avoid external dependencies
4. Update this README with new test coverage

## CI/CD Integration

Tests run automatically on:
- Pull requests
- Main branch commits
- Release tags

Ensure all tests pass before merging.
