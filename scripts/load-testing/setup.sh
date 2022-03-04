#!/bin/bash

# create router canister
dfx canister --network ic create ic-history-router-stg

router_cid=$(dfx canister --network ic id ic-history-router-stg)

# build router
dfx build --network ic ic-history-router-stg

# install router wasm
dfx canister --network ic install ic-history-router-stg