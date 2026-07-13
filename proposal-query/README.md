# Proposal Query Contract

A minimal CosmWasm smart contract whose only purpose is to query a governance
proposal through the chain's gRPC gov module (`/cosmos.gov.v1.Query/Proposal`)
and return the decoded result. It demonstrates the generic `GrpcQuery` mechanism
(`QuerierWrapper::query_grpc`, added in CosmWasm 2.0) for reaching chain data that
isn't exposed through cosmwasm-std's built-in queries.

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

### Proposal
```json
{
  "proposal": {
    "proposal_id": 1
  }
}
```

Returns:
```json
{
  "id": 1,
  "title": "...",
  "summary": "...",
  "status": "PROPOSAL_STATUS_VOTING_PERIOD",
  "proposer": "cosmos1...",
  "submit_time": 1700000000,
  "voting_end_time": 1700100000,
  "yes_count": "...",
  "abstain_count": "...",
  "no_count": "...",
  "no_with_veto_count": "..."
}
```

`yes_count`, `abstain_count`, `no_count`, and `no_with_veto_count` come from the
proposal's `final_tally_result`, which the chain only populates once the voting
period has ended; until then they're returned as empty strings.

## Operational note

gRPC queries are only reachable if the target chain's `x/wasm` params allowlist
`/cosmos.gov.v1.Query/Proposal` (wasmd's gRPC/Stargate query allowlist) and the
node's wasmvm build supports the CosmWasm 2.0 capability. This contract's unit
tests pass with a mocked querier regardless, but a real query against a chain
that hasn't allowlisted the path will fail at runtime.
