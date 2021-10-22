# Indexer Interface

The indexer interface describes the common interface of all the primary canisters
on the Cap network that can be queried to return the whereabouts of transactions.

The main canister (a.k.a. router) implements the indexer interface.

## query get_index_canister

This method can be invoked on any of the index canisters to return the list
of all the index canister.

## query get_token_contract_root_bucket

Return the index canister of the 

## query get_user_root_buckets