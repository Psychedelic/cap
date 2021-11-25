import Result "mo:base/Result";
import Principal "mo:base/Principal";
import Root "Root";
import Types "Types";
import Router "Router";

class Cap(canister_id: Principal, creation_cycles: Nat) {
    let router = "text";

    let ?rootBucket = null;

    func getTransaction(id: Nat64) : async Result.Result<Root.Event, Types.GetTransactionError> {
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

    func insert(event: Root.IndefiniteEvent) : async Result.Result<Nat64, Types.InsertTransactionError> {
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
                // TODO
            };
            case (?canister) {
                rootBucket := canister;
            };
        };
    };

    func awaitForHandshake(): async () {
        // TODO D:
    }
};