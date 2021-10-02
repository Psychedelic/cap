# Cap Open Internet Service

Cap is an open internet service designed with the goal of providing a service for storing
the transactions of Fungible and Non-Fungible Tokens on the Internet Computer.

This document goes thorough the interface of this service, this description should be
applicable regardless of the scalability level of the implementation. It is possible to
have an implementation that uses a heavily multi-canister approach or even have a single
canister implementation. In any cases this description tries to be non-opinionated.

## General overview

This specification details several interfaces, a canister can implement any of these
interfaces, the main/entry interface is called the **Router Interface**, a canister
that implements this interface can be used as an entry point to the Cap's Network.

> Implementation Note: The main Cap canister implements the *Router Interface*.

Token contracts integrating the Cap Protocol, can make an initial call to the
*Router Canister* to initialize a canister they have already created on their
side, and from that point forward use that canister to write transactions to Cap.

> Note: In this document the term *reader* describes non-writer users of the protocol,
> writer users are referred to as *Token Contract*. The term *user* itself refers to
> user's of the token contracts that are part of the transactions.

We should make it possible for readers of Cap to read the entire transaction history
of every token, and also read transactions of a certain users. Hence, we have another
interface called the **Indexer Interface**, which provides queries for determining
"Where things are.", an indexer does not contain any actual transaction within itself,
but just the whereabouts of the transactions.

The *Router Interface* extends the *Indexer Interface*, in other terms any *Router Canister*
is also an *Indexer Canister*.

Each token contract can have only one *Root Bucket*, this canister is the canister that
was initialized via the *Router Canister*, and is responsible for providing the methods
which can be used by a token-contract to insert transactions into the Cap network.

As the name suggests, *Root Bucket* itself is also a *Bucket*, a bucket contains the actual
data of transactions. A *Root Bucket* can scale itself to be using multiple *Bucket*s.
