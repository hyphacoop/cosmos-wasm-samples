#!/bin/bash

########  CHAIN CONFIG ########
WALLET=cosmos*
CHAIN_BINARY=gaiad
HOME=~/.gaia
CHAIN_DENOM=uatom
NODE=http://localhost:26657
CHAIN_ID=testnet
COMMIT_TIMEOUT=3
GAS_PRICE=0.005$CHAIN_DENOM
###############################

# FILENAME=target/wasm32-unknown-unknown/release/bitmap_free.wasm # cargo wasm result
FILENAME=bitmap-free/artifacts/bitmap_free.wasm # cosmwasm/optimizer result


echo "> Storing the contract on chain"
tx_hash=$($CHAIN_BINARY tx wasm store $FILENAME --home $HOME --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.txhash')
sleep $COMMIT_TIMEOUT
code_id=$($CHAIN_BINARY query tx $tx_hash -o json --node $NODE | jq -r '.events[] | select(.type=="store_code").attributes[] | select(.key=="code_id").value')
echo "> Code ID: $code_id"

echo "> Instantiating a contract without z_values"
tx_hash=$($CHAIN_BINARY tx wasm instantiate $code_id '{"x_size":16,"y_size":16}' --home $HOME --label "bitmap" --no-admin --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.txhash')
sleep $COMMIT_TIMEOUT
contract=$($CHAIN_BINARY query tx $tx_hash -o json --node $NODE | jq -r '.events[] | select(.type=="instantiate").attributes[] | select(.key=="_contract_address").value')
echo "> Contract address: $contract"
result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_grid":{}}' -o json --node $NODE | jq -r '.data.z_values')
echo "> Full grid: $result"


echo "> Instantiating a contract with z_values"
tx_hash=$($CHAIN_BINARY tx wasm instantiate $code_id '{"x_size":16,"y_size":16, "z_values":"9900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff019900fc9900fc9900fc9900fc9900fc9900fc9900fc9900fc0bff010bff010bff010bff010bff010bff010bff010bff01000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff"}' --home $HOME --label "bitmap" --no-admin --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.txhash')
sleep $COMMIT_TIMEOUT
contract=$($CHAIN_BINARY query tx $tx_hash -o json --node $NODE | jq -r '.events[] | select(.type=="instantiate").attributes[] | select(.key=="_contract_address").value')
echo "> Contract address: $contract"

result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_grid":{}}' -o json --node $NODE | jq -r '.data.z_values')
echo "> Full grid: $result"
result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point at (0,0): $result"
result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":1,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point at (1,0): $result"
result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":1,"y":1}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point at (1,1): $result"

echo "> Executing the contract (unhappy path - out of bounds)"
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":200,"y":1,"z":"0011AA"}}'  --home $HOME --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID
echo "> Executing the contract (set 0,1)"
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":0,"y":0,"z":"0011AA"}}' --home $HOME --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID
sleep $COMMIT_TIMEOUT
echo "> Executing the contract (set 1,0)"
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":1,"y":0,"z":"0022BB"}}' --home $HOME --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID
sleep $COMMIT_TIMEOUT
echo "> Executing the contract (set 1,1)"
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":1,"y":1,"z":"0033CC"}}' --home $HOME --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID
sleep $COMMIT_TIMEOUT

result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_grid":{}}' -o json --node $NODE | jq -r '.data.z_values')
echo "> Full grid: $result"
result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point at (0,0): $result"
result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":1,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point at (1,0): $result"
result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":1,"y":1}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point at (1,1): $result"

