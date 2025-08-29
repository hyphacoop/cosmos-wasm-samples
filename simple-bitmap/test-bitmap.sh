#!/bin/bash

# FILENAME=target/wasm32-unknown-unknown/release/simple_bitmap.wasm # cargo wasm result
FILENAME=simple-bitmap/artifacts/simple_bitmap.wasm # cosmwasm/optimizer result

# Chain config
WALLET=cosmos*
CHAIN_BINARY=gaiad
HOME=~/.gaia
CHAIN_DENOM=uatom
NODE=http://localhost:26657
CHAIN_ID=testnet
COMMIT_TIMEOUT=3
GAS_PRICE=0.005$CHAIN_DENOM

echo "> Storing the contract on chain"
tx_hash=$($CHAIN_BINARY tx wasm store $FILENAME --home $HOME --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.txhash')
sleep $COMMIT_TIMEOUT
echo "> Querying the hash for the code ID"
code_id=$($CHAIN_BINARY query tx $tx_hash -o json --node $NODE | jq -r '.events[] | select(.type=="store_code").attributes[] | select(.key=="code_id").value')
echo "> Code ID: $code_id"

echo "> Instantiating the contract"
tx_hash=$($CHAIN_BINARY tx wasm instantiate $code_id '{"with_string":{"x_size":16,"y_size":16, "z_values":"9900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff01000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff"}}' --home $HOME --label "bitmap" --no-admin --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.txhash')
sleep $COMMIT_TIMEOUT
echo "> Querying the hash for the contract address"
contract=$($CHAIN_BINARY query tx $tx_hash -o json --node $NODE | jq -r '.events[] | select(.type=="instantiate").attributes[] | select(.key=="_contract_address").value')
echo "> Contract address: $contract"

echo "> Executing the contract (unhappy path)"
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":200,"y":1,"z":"0011AA"}}'  --home $HOME --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID
sleep $COMMIT_TIMEOUT

echo "> Executing the contract (happy path)"
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":0,"y":0,"z":"0011AA"}}' --home $HOME --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID
sleep $COMMIT_TIMEOUT

echo "> Executing the contract (happy path)"
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":1,"y":0,"z":"0022BB"}}' --home $HOME --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID
sleep $COMMIT_TIMEOUT

echo "> Executing the contract (happy path)"
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":1,"y":1,"z":"0033CC"}}' --home $HOME --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID
sleep $COMMIT_TIMEOUT

echo "> Querying the contract for point at (0, 0)"
result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point: $result"

echo "> Querying the contract for point at (1, 0)"
result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":1,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point: $result"

echo "> Querying the contract for point at (1, 1)"
result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":1,"y":1}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point: $result"

echo "> Querying the contract for the entire grid"
result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_grid":{}}' -o json --node $NODE | jq -r '.data.z_values')
echo "> Full grid: $result"
