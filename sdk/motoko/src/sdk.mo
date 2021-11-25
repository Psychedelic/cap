/// A shiny new library
///
/// Make it easy and fun to use your new library by including some module specific documentation here.
/// It's always a good idea to include a minimal working example:
///
/// ```motoko
/// import LibraryTemplate "mo:library-template/Library";
///
/// assert(LibraryTemplate.isPalindrome("anna"));
/// assert(not LibraryTemplate.isPalindrome("christoph"));
/// ```

module {
    /// Gets the transaction with the given `id`.
    func get_transaction(id: Nat) : async Result.Result<Event, GetTransactionError> {
        // Get bucket context

        let transaction_response = await Root.get_transaction(WithIdArg {
            id = id;
            witness = false;
        });

        switch(transaction_response) {
            case (#Found{event;}) {
                switch (event) {
                    case null {
                        #err(#noTransaction)
                    };
                    case (?event) {
                        #ok(event)
                    }
                }
            };
            case (#Delegate{}) {
                #err(#unsupportedResponse)
            };
        }
    };

    /// Inserts a transaction
    func insert(event: IndefiniteEvent) : async Result.Result<Nat, InsertTransactionError> {
        // Get context

        let id = await Root.insert(event);

        #ok(id)
    }
};
