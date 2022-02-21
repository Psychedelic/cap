# Bucket Interface

The bucket interface describes the interface for canisters that can be queried to return a certain portion of
transactions for a token contract. This is a read-only interface.

// TODO(qti3e)

## query get_token_transactions

Returns the transactions for a certain token id. It will return
all the transactions that involve a `TokenIdU64(n)`.
