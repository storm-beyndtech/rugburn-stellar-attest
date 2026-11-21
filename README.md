# RugBurn Asset Passport Registry

A Soroban contract for publishing and verifying SHA-256 commitments to Stellar Asset Passport reports. An integration can look up an asset by code and issuer, then compare the report it received with the publisher’s commitment.

The report carries issuer observations and interpretation. The registry carries the publisher’s score, digest, scan time, model version, and revocation state. A hash match verifies the report bytes against that commitment. Issuer identity, ledger observations, and the decision to accept an asset depend on separate checks.

## Build and test

Install [Rust with rustup](https://rustup.rs/) and [Stellar CLI 30.0.1](https://github.com/stellar/stellar-cli/releases/tag/v30.0.1). The checked-in toolchain file selects Rust and the Soroban Wasm target.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
stellar contract build --locked
RUGBURN_TEST_WASM=target/wasm32v1-none/release/rugburn_attest.wasm cargo test --locked
sha256sum -c --strict examples/SHA256SUMS
```

Read the compiled interface with `stellar contract info interface --wasm target/wasm32v1-none/release/rugburn_attest.wasm`.

## Integrate

- [Architecture](docs/architecture.md) covers the asset key, publisher authorization, storage, revocation, and evidence boundary.
- [Verification](docs/verification.md) covers the archived testnet record, byte-for-byte verification, and offline Passport fixtures.

## Deployment status

This source uses atomic publisher setup for future deployments. The historical deployment used an earlier interface. Its testnet entries were archived at the latest documented check. The Horizon and SEP-1 adapter and the report publisher run outside this repository.

Licensed under [Apache-2.0](LICENSE).
