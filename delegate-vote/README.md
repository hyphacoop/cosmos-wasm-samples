# Delegate Contract

A CosmWasm smart contract that supports delegating tokens to validators and voting on governance proposals.

## Features

- **Delegate**: Delegate uatom tokens to a validator
- **Vote**: Vote on governance proposals with Yes, No, Abstain, or NoWithVeto options
- **WeightedVote**: Vote on governance proposals with weighted options (e.g., 60% Yes, 40% Abstain)

## Build

```bash
cargo build
cargo wasm
```

## Test

```bash
cargo test
```

## Generate Schema

```bash
cargo run --bin schema
```

## Execute Messages

### Delegate
```json
{
  "delegate": {
    "validator": "cosmosvaloper1...",
    "amount": {
      "denom": "uatom",
      "amount": "1000000"
    }
  }
}
```

### Vote
```json
{
  "vote": {
    "proposal_id": 1,
    "option": "yes"
  }
}
```

Vote options: `yes`, `no`, `abstain`, `no_with_veto`

### WeightedVote
```json
{
  "weighted_vote": {
    "proposal_id": 1,
    "options": [
      {
        "option": "yes",
        "weight": "0.6"
      },
      {
        "option": "abstain",
        "weight": "0.4"
      }
    ]
  }
}
```

Weights should be decimal values that sum to 1.0. Vote options: `yes`, `no`, `abstain`, `no_with_veto`
