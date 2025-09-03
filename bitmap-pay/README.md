# Paid Bitmap

This contract builds on the [bitmap-free](/bitmap-free/README.md) example, and allows users to set points in a 2D grid...for a price. The state holds several variables:

* `x_size` and `y_size`: 8-bit unsigned integers that determine the grid size upon instantiation.
* `z_values`: A string made up of 6-character chunks that represent a 16-bit colour at each grid coordinate.
* `recipient`: The address that will receive the funds for all point-setting transactions.
* `fee_denom`: The denom required for the point-setting fee.

The cost associated with setting a point is calculated with two curves. The first one is associated with the number of points that have not been set since the contract was instantiated, and the second one uses the number of times that the specific point has been updated. This requires the following variables to be set during instantiation:
* `supply_base_fee`
* `supply_fee_factor`
* `update_base_fee`
* `update_fee_factor`

Before set a point, you must run a `get_cost()` query for the coordinates you want. Then you can execute `set_point()` using `--amount` flag with the relevant cost in it.

## Build Contract

Run the following from the root folder in this repo:
```bash
RUSTFLAGS='-C link-arg=-s' cargo wasm
docker run --rm -v "$(pwd)/bitmap-pay":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0
```
This will build an optimized version of the contract under `bitmap-pay/artifacts/bitmap_pay.wasm`

## Run Contract

The commands shown below assume you have a `wasmd` network running with permissionless CosmWasm, as in:
```bash
wasmd q wasm params
code_upload_access:
  addresses: []
  permission: Everybody
instantiate_default_permission: Everybody
```

> Use the [`test-bitmap-pay.sh`](test-bitmap-pay.sh) script in this folder to store, instantiate, and execute the contract in a Gaia chain.

### Deploy Contract

Store the code:
```bash
wasmd tx wasm store path-to/bitmap_pay.wasm -o json | jq -r '.txhash'
# Wait for transaction to go on chain
code_id=$(wasmd query tx $tx_hash -o json | jq -r '.events[] | select(.type=="store_code").attributes[] | select(.key=="code_id").value')
```
Once the code has been stored, you can instantiate, execute, and query it. To instantiate it:
```bash
instantiate_json=$(jq -n \
  --argjson xsize 4 \
  --argjson ysize 4 \
  --arg recipient "<recipient address>" \
  --argjson supply_base_fee <u128> \
  --argjson supply_fee_factor <u8> \
  --argjson update_base_fee <u128> \
  --argjson update_fee_factor <u8> \
  --arg fee_denom "<denom>" \
  '{"x_size":$xsize,"y_size":$ysize, "recipient": $recipient, "supply_base_fee": $supply_base_fee, "supply_fee_factor": $supply_fee_factor, "update_base_fee": $update_base_fee, "update_fee_factor": $update_fee_factor, "fee_denom": $fee_denom}')
tx_hash=$($CHAIN_BINARY tx wasm instantiate $code_id "$instantiate_json" --home $HOME --label "bitmap" --no-admin --from $WALLET --gas auto --gas-adjustment 3 --gas-prices $GAS_PRICE -y -o json --node $NODE --chain-id $CHAIN_ID | jq -r '.txhash')
# Wait for transaction to go on chain
contract_address=$(wasmd query tx $tx_hash -o json | jq -r '.events[] | select(.type=="instantiate").attributes[] | select(.key=="_contract_address").value')
```

The cost for setting a point is calculated as follows:
```
supply_curve_cost = supply_base_fee * e^( (supply_fee_factor / 100) * number_of_points_set_so_far_in_the_grid)
update_curve_cost = update_base_fee * e^( (update_fee_factor / 100) * number_of_times_this_point_has_been_set)
set_point_cost = supply_curve_cost + update_curve_cost;
```

### Query params via CLI

Obtain the cost of setting a point with the `get_cost` function
```bash
params=$(wasmd q wasm contract-state smart $contract_address '{"get_params":{}}' -o json | jq -r '.data')
echo "> Params: $params"
fee_denom=$(echo $params | jq -r '.fee_denom')
```

### Query cost via CLI

Obtain the cost of setting a point with the `get_cost` function
```bash
cost=$(wasmd q wasm contract-state smart $contract_address '{"get_cost":{"x":0,"y":0}}' -o json | jq -r '.data.point')
echo "> Cost: $cost"
```

### Execute: Set values via CLI

Set values with the `set` function
```bash
wasmd tx wasm execute $contract_address '{"set":{"x":0,"y":0,"z":"0011AA"}}' --amount $cost$fee_denom # The x and y values must be within the limits set in the instantiate step
```

### Query values via CLI

Obtain the value of an individual point with the `get_point` function
```bash
result=$(wasmd q wasm contract-state smart $contract_address '{"get_point":{"x":0,"y":0}}' -o json | jq -r '.data.point')
echo "> Point: $result"
```

Obtain the full grid with the `get_grid` function
```bash
result=$(wasmd q wasm contract-state smart $contract_address '{"get_grid":{}}' -o json | jq -r '.data.z_values')
echo "> Full grid: $result"
```

### Render the grid

The `bitmap-free` folder includes a [webpage](/bitmap-free/index.html) that renders the `z_values` as a bitmap. You must set the following in the `API_URL` variable:
* Node API address (your target node must have API enabled)
* Contract address

The query is the `get_grid` JSON in base64:
```
echo '{"get_grid":{}}' | base64
eyJnZXRfZ3JpZCI6e319Cg==
```
