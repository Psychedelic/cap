declare -a cid_arr

NETWORK="ic"

wallet=$(dfx identity --network $NETWORK get-wallet)
principal=$(dfx identity --network $NETWORK get-principal)

router_cid="e22n6-waaaa-aaaah-qcd2q-cai"
root="eznu4-oyaaa-aaaah-qcehq-cai"

count=50000

echo "wallet: $wallet"
echo "principal: $principal"

timestamp=$(date +%s%N)

perform_insert () {
  ts=$(date +%s%N)
  #echo "doing insert $1"

  dfx canister --network $NETWORK --wallet=$(dfx identity --network $NETWORK get-wallet) call eznu4-oyaaa-aaaah-qcehq-cai insert '(record {
         status = variant { Completed };
         operation = "Approve";
         details = vec {
            record {
              "approved_user";
              variant {
                Principal = principal "avesb-mgo2l-ds25i-g7kd4-3he5l-z7ary-3biiq-sojiw-xjgbk-ich5l-mae"
              };
            };
         };
         caller = principal "zxt4e-ian3w-g4ll2-3n5mz-lfqkc-eyj7k-yg6jl-rsbud-f6sft-zdfq3-pae";
  })'
  echo "insert $1 took "$((($(date +%s%N) - $ts)/1000000))
}


for ((i = 0; i < $count; i++)); do
  perform_insert $i &
done