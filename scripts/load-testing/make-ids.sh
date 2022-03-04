#!/bin/bash

count=2

for ((i = 0; i < $count; i++)); do
  `dfx identity new load-test-$i`
  `dfx identity use load-test-$i`
  wallet_cid=`dfx canister --network $NETWORK call --with-cycles 100000000000 aaaaa-aa create_canister "(record { cycles=(100_000_000_000:nat64); controller=(opt principal \"$principal\") })" | tail -n -1`
  dfx identity deploy-wallet $wallet_cid
done