# Validator Query Contract

A minimal CosmWasm smart contract whose only purpose is to query a validator
through the chain's gRPC staking module (`/cosmos.staking.v1beta1.Query/Validator`)
and return the decoded result. It demonstrates the generic `GrpcQuery` mechanism
(`QuerierWrapper::query_grpc`, added in CosmWasm 2.0) for reaching chain data that
isn't exposed through cosmwasm-std's built-in `StakingQuery`.

The contract has no state and no execute messages — just `instantiate` and `query`.

## Build

```bash
cargo build
cargo wasm
```

## Test

```bash
cargo test
```

The unit test mocks the gRPC response so the encode/decode logic can be verified
without a live chain.

## Generate Schema

```bash
cargo run --bin schema
```

## Query Messages

### Validator
```json
{
  "validator": {
    "validator_addr": "cosmosvaloper1..."
  }
}
```

Returns:
```json
{
  "operator_address": "cosmosvaloper1...",
  "moniker": "...",
  "jailed": false,
  "status": "BOND_STATUS_BONDED",
  "tokens": "...",
  "delegator_shares": "...",
  "commission_rate": "..."
}
```

## Operational note

gRPC queries are only reachable if the target chain's `x/wasm` params allowlist
`/cosmos.staking.v1beta1.Query/Validator` (wasmd's gRPC/Stargate query allowlist)
and the node's wasmvm build supports the CosmWasm 2.0 capability. This contract's
unit tests pass with a mocked querier regardless, but a real query against a chain
that hasn't allowlisted the path will fail at runtime.
