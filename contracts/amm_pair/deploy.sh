#!/bin/sh

# build and start blockchain
# make start-server
CODE_ID=0
ADDRESS=""
INIT='{"symbol": "ETHU"}'
LABEL="shade_pair_contract"
docker exec -i secretdev bash -c "secretd tx compute store /root/code/contract.wasm.gz --from a --gas=auto --gas-adjustment=1.15 -y --keyring-backend test" > createresult
# #  # /root/code/contract.wasm.gz
#docker exec -i secretdev cat  < createresult
docker exec -i secretdev secretd query compute list-code > result.json
# get 1 id and creator
ADDRESS=$(jq .[0] -r < result.json | jq ."creator")
CODE_ID=$(jq .[0] -r < result.json | jq ."id")
echo $ADDRESS
echo $CODE_ID
LASTID=$(jq '. | length' < result.json)
echo $LASTID
docker exec -i secretdev secretd tx compute instantiate 1 "$INIT" --from a --label $LABEL -y --keyring-backend test > tx.json
TX=$(jq ."txhash" < tx.json )
echo $TX
docker exec -i secretdev secretd query compute list-contract-by-code $LASTID
#docker exec -i secretdev bash -c "cd ./code && secretd tx compute store contract.wasm.gz --from a --gas=auto --gas-adjustment=1.15 -y --keyring-backend test /root/code/contract.wasm.gz"
#docker exec secretdev secretd tx compute store ./code/contract.wasm.gz --from a --gas 1000000 -y --keyring-backend test /root/code/contract.wasm.gz


./secretcli tx compute instantiate 861244 '{"name":"Token0","symbol":"XUSDT","admin":"secret1vhm32pcaz2vsjdyxdl7mtyyw7cug25rfqcc2yn","decimals":18,"initial_balances":[{"address":"secret1vhm32pcaz2vsjdyxdl7mtyyw7cug25rfqcc2yn","amount":"10000000"}],"prng_seed":"VE9LRU4w","config":{"public_total_supply":false,"enable_deposit": true,"enable_redeem": true,"enable_mint": true,"enable_burn":true}}' --label shade_test_token_0 --from token0 --keyring-backend file

./secretcli tx compute instantiate 861244 '{"name":"Token0","symbol":"XUSDT","decimals":18,"initial_balances":[],"prng_seed":"VE9LRU4w"}' --label shade_test_token_0 --from token0 --keyring-backend file

