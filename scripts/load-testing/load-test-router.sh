declare -a cid_arr

NETWORK="local"

wallet=$(dfx identity --network $NETWORK get-wallet)
principal=$(dfx identity --network $NETWORK get-principal)

router_cid="rrkah-fqaaa-aaaaa-aaaaq-cai"

count=50

echo "wallet: $wallet"
echo "principal: $principal"

timestamp=$(date +%s%N)

for ((i = 0; i < $count; i++)); do
  `dfx identity new load-test-$timestamp-$i`
  `dfx identity use load-test-$timestamp-$i`
  ts=$(date +%s%N)
  cid=`dfx canister --network $NETWORK --wallet=$(dfx identity --network $NETWORK get-wallet) call --with-cycles 100000000000 aaaaa-aa create_canister "(record { cycles=(100_000_000_000:nat64); controller=(opt principal \"$principal\") })" | tail -n -1`
  cid_cleaned=`echo "$cid" | cut -d'"' -f 2`
  echo $cid_cleaned >> ./scripts/load-testing/cids.out
  echo "created canister $i: $cid_cleaned"
  cid_arr+=($cid_cleaned)
  echo "Step 1 took "$((($(date +%s%N) - $ts)/1000000))
done

for ((i = 0; i < $count; i++)); do
  `dfx identity use load-test-$timestamp-$i`
  ts=$(date +%s%N)
  echo "adding controller for bucket canister $i: ${cid_arr[i]}"
  dfx canister --network $NETWORK --wallet=$(dfx identity --network $NETWORK get-wallet) call aaaaa-aa update_settings "(record { canister_id=(principal \"${cid_arr[i]}\"); settings=(record { controller = opt principal \"$router_cid\"; null; null; null; })})"
  echo "starting install"
  dfx canister --network $NETWORK --wallet=$(dfx identity --network $NETWORK get-wallet) call $router_cid install_bucket_code "(principal \"${cid_arr[i]}\")"
  echo "Step 2 took "$((($(date +%s%N) - $ts)/1000000))
done