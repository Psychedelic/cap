import Result "mo:base/Result";
import Principal "mo:base/Principal";
import Debug "mo:base/Debug";
import Root "Root";
import Types "Types";
import Router "Router";
import ic "ic:aaaaa-aa";

class Cap(canister_id: Principal, creation_cycles: Nat) {
    let router = "text";

    let ?rootBucket = null;

    public func getTransaction(id: Nat64) : async Result.Result<Root.Event, Types.GetTransactionError> {
        await awaitHandshake();

        let rootBucket = (actor (rootBucket) : Root.Self);

        let transaction_response = await rootBucket.get_transaction({ id=id; witness=false; }); 

        switch(transaction_response) {
            case (#Found(event, witness)) {
                switch(event) {
                    case (null) {
                        #err(#invalidTransaction)
                    };
                    case (?event) {
                        #ok(event)
                    }
                }
            };
            case (#Delegate(_, _)) {
                #err(#unsupportedResponse)
            }
        }
    };

    public func insert(event: Root.IndefiniteEvent) : async Result.Result<Nat64, Types.InsertTransactionError> {
        await awaitHandshake();

        let rootBucket = (actor (rootBucket) : Root.Self);

        let insert_response = await rootBucket.insert(event);

        #ok(insert_response)
    };


    /// Returns the principal of the root canister
    private func performHandshake() {
        let router = (actor (router) : Router.Self);

        let result = await router.get_token_contract_root_bucket({
            witness=false;
            canister= canisterId;
        });

        switch(result.canister) {
            case(null) {
                let settings = ic.canister_settings {
                    controllers = router;
                    compute_allocation = null;
                    memory_allocation = null;
                    freezing_threshold = null;
                };

                // Add cycles and perform the create call
                ExperimentalCycles.add(creation_cycles);
                let create_response = await ic.create_canister(settings);

                // Install the cap code
                let canister = create_response.canister_id;
                let router = (actor (router) : Router.Self);
                await router.install_code(canister);

                let result = await router.get_token_contract_root_bucket({
                    witness=false;
                    canister= canisterId;
                });

                switch(result.canister) {
                    case(null) {
                        Debug.trap("Error while creating root bucket");
                    };
                    case(?canister) {
                        rootBucket := canister;
                    };
                };
            };
            case (?canister) {
                rootBucket := canister;
            };
        };
    };

    func awaitForHandshake(): async () {
        if(rootBucket == null) {
            await performHandshake();
        } else {
            return;
        }
    }
};