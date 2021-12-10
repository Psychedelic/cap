# Router

## Type `GetIndexCanistersResponse`
`type GetIndexCanistersResponse = { witness : ?Witness; canisters : [Principal] }`


## Type `GetTokenContractRootBucketArg`
`type GetTokenContractRootBucketArg = { witness : Bool; canister : Principal }`


## Type `GetTokenContractRootBucketResponse`
`type GetTokenContractRootBucketResponse = { witness : ?Witness; canister : ?Principal }`


## Type `GetUserRootBucketsArg`
`type GetUserRootBucketsArg = { user : Principal; witness : Bool }`


## Type `GetUserRootBucketsResponse`
`type GetUserRootBucketsResponse = { witness : ?Witness; contracts : [Principal] }`


## Type `WithWitnessArg`
`type WithWitnessArg = { witness : Bool }`


## Type `Witness`
`type Witness = { certificate : [Nat8]; tree : [Nat8] }`


## Type `Self`
`type Self = actor { deploy_plug_bucket : shared (Principal, Nat64) -> async (); get_index_canisters : shared query WithWitnessArg -> async GetIndexCanistersResponse; get_token_contract_root_bucket : shared query GetTokenContractRootBucketArg -> async GetTokenContractRootBucketResponse; get_user_root_buckets : shared query GetUserRootBucketsArg -> async GetUserRootBucketsResponse; insert_new_users : shared (Principal, [Principal]) -> async (); install_bucket_code : shared Principal -> async (); trigger_upgrade : shared () -> async () }`

