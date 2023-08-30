# Repute contract

This repo contains the source code for a minimum smart contract of a reputation system in [ink!](https://use.ink/).

## How it works

Repute at it's core is a book-keeper.

It keep track of registered user reputation since their registration.

A user's reputation is updated when they interact with this contract.

Repute has an important function ```calculate_reputation_score``` that mocks a time based reputation scoring engine.

Repute uses score from `calculate_reputation_score` to update user reputation and rank.

## Development

1. Install `rustup` via https://rustup.rs/
2. Run the following:

```
rustup install 1.68.0
rustup target add wasm32-unknown-unknown
```

3. Install cargo-contract [here](https://github.com/paritytech/cargo-contract)
4. To install contract node, run: 

```
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git --tag v0.23.0 --force --locked
```

### Compiling

You can build release version by running:

```
cargo contract build --release
```

### Testing

Run:

```
cargo contract test
```
