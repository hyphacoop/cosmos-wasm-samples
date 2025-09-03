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
RECIPIENT=cosmos*
###############################

# FILENAME=target/wasm32-unknown-unknown/release/bitmap_pay.wasm # cargo wasm result
FILENAME=bitmap-pay/artifacts/bitmap_pay.wasm # cosmwasm/optimizer result

echo "> Storing the contract on chain"
tx_hash=$($CHAIN_BINARY tx wasm store $FILENAME --home $HOME --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.txhash')
sleep $COMMIT_TIMEOUT
echo "> Querying the hash for the code ID"
code_id=$($CHAIN_BINARY query tx $tx_hash -o json --node $NODE | jq -r '.events[] | select(.type=="store_code").attributes[] | select(.key=="code_id").value')
echo "> Code ID: $code_id"

echo "> Instantiating the contract"
instantiate_json=$(jq -n \
  --argjson xsize 4 \
  --argjson ysize 4 \
  --arg recipient "$RECIPIENT" \
  --argjson supply_base_fee 50 \
  --argjson supply_fee_factor 50 \
  --argjson update_base_fee 50 \
  --argjson update_fee_factor 10 \
  --arg fee_denom $CHAIN_DENOM \
  '{"x_size":$xsize,"y_size":$ysize, "recipient": $recipient, "supply_base_fee": $supply_base_fee, "supply_fee_factor": $supply_fee_factor, "update_base_fee": $update_base_fee, "update_fee_factor": $update_fee_factor, "fee_denom": $fee_denom}')
tx_hash=$($CHAIN_BINARY tx wasm instantiate $code_id "$instantiate_json" --home $HOME --label "bitmap" --no-admin --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.txhash')
sleep $COMMIT_TIMEOUT
echo "> Querying the hash for the contract address"
contract=$($CHAIN_BINARY query tx $tx_hash -o json --node $NODE | jq -r '.events[] | select(.type=="instantiate").attributes[] | select(.key=="_contract_address").value')
echo "> Contract address: $contract"

grid=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_grid":{}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.z_values')
echo "> Grid: $grid"

echo "> Instantiating the contract"
instantiate_json=$(jq -n \
  --argjson xsize 4 \
  --argjson ysize 4 \
  --arg zvalues  "0000000000880088008800002222220000AA00AA00AA00000000000000880088008800002222220000AA00AA00AA0000" \
  --arg recipient "$RECIPIENT" \
  --argjson supply_base_fee 50 \
  --argjson supply_fee_factor 50 \
  --argjson update_base_fee 50 \
  --argjson update_fee_factor 10 \
  --arg fee_denom $CHAIN_DENOM \
  '{"x_size":$xsize,"y_size":$ysize, "z_values":$zvalues, "recipient": $recipient, "supply_base_fee": $supply_base_fee, "supply_fee_factor": $supply_fee_factor, "update_base_fee": $update_base_fee, "update_fee_factor": $update_fee_factor, "fee_denom": $fee_denom}')
tx_hash=$($CHAIN_BINARY tx wasm instantiate $code_id "$instantiate_json" --home $HOME --label "bitmap" --no-admin --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.txhash')
sleep $COMMIT_TIMEOUT
echo "> Querying the hash for the contract address"
contract=$($CHAIN_BINARY query tx $tx_hash -o json --node $NODE | jq -r '.events[] | select(.type=="instantiate").attributes[] | select(.key=="_contract_address").value')
echo "> Contract address: $contract"

grid=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_grid":{}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.z_values')
echo "> Grid: $grid"
params=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_params":{}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data')
echo "> Params: $params"
$CHAIN_BINARY q bank balances $RECIPIENT -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.balances'

result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point at (0,0): $result"

cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,0): $cost"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":1}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,1): $cost"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":2}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,2): $cost"

cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Executing the contract (unhappy path: bad index)"
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":200,"y":0,"z":"0011AA"}}'  --home $HOME --from $WALLET --amount ${cost}uatom --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID

echo "> Executing the contract (unhappy path: insufficient funds)"
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":0,"y":0,"z":"0011AA"}}'  --home $HOME --from $WALLET --amount 10uatom --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID

echo "> Setting (0,0) for the first time"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":0,"y":0,"z":"0011AA"}}' --home $HOME --from $WALLET --amount ${cost}uatom --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID
sleep $COMMIT_TIMEOUT
$CHAIN_BINARY q bank balances $RECIPIENT -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.balances'

result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point at (0,0): $result"

cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,0): $cost"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":1}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,1): $cost"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":2}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,2): $cost"

echo "> Setting (0,0) for the second time"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":0,"y":0,"z":"0033CC"}}' --home $HOME --from $WALLET --amount ${cost}uatom --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID
sleep $COMMIT_TIMEOUT
$CHAIN_BINARY q bank balances $RECIPIENT -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.balances'

result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point at (0,0): $result"

cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,0): $cost"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":1}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,1): $cost"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":2}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,2): $cost"

echo "> Setting (0,0) for the third time"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":0,"y":0,"z":"0033CC"}}' --home $HOME --from $WALLET --amount ${cost}uatom --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID
sleep $COMMIT_TIMEOUT
$CHAIN_BINARY q bank balances $RECIPIENT -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.balances'

result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point at (0,0): $result"

cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,0): $cost"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":1}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,1): $cost"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":2}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,2): $cost"

echo "> Setting (0,1) for the first time"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":1}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
$CHAIN_BINARY tx wasm execute $contract '{"set":{"x":0,"y":1,"z":"EE1122"}}' --home $HOME --from $WALLET --amount ${cost}uatom --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID
sleep $COMMIT_TIMEOUT
$CHAIN_BINARY q bank balances $RECIPIENT -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.balances'

result=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_point":{"x":0,"y":1}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.point')
echo "> Point at (0,1): $result"

cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":0}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,0): $cost"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":1}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,1): $cost"
cost=$($CHAIN_BINARY q wasm contract-state smart $contract '{"get_cost":{"x":0,"y":2}}' -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.data.cost')
echo "> Cost at (0,2): $cost"
