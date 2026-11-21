# Architecture

## Boundaries

| Layer | Responsibility |
| --- | --- |
| Stellar Passport adapter | Records Horizon and SEP-1 observations for a specific asset and ledger context. Distinguishes measured controls, missing evidence and interpretations such as a known-issuer match. |
| Report publisher | Serializes the report, computes SHA-256 over those exact bytes, authorizes the write and makes the report available to its consumers. |
| Soroban registry | Stores one authorized commitment per asset key. Records the score, report hash, publisher-supplied scan time, model-version symbol and revocation state. |
| Consumer | Chooses the trusted publisher and network, checks revocation and freshness, hashes the bytes, and decides whether the evidence supports acceptance. |

The contract handles no funds and stores neither customer data nor the report body. A matching digest identifies the bytes the chosen publisher committed to. Independent verification of Horizon observations, issuer identity and issuer actions remains a separate step.

## Asset key

For a classic issued Stellar asset, hash the UTF-8 bytes of `stellar:<ASSET_CODE>:<ISSUER_STRKEY>` with SHA-256. Use the exact case for the asset code and issuer StrKey. A home domain or asset code alone is insufficient. Network identity is established through the registry deployment and RPC configuration because the asset key omits the network.

Horizon account flags describe issuer-level capabilities. Trustline authorization and clawback eligibility may differ by trustline. SEP-1 domain and TOML declarations are evidence about the issuer's published metadata. The consumer can evaluate these alongside an independently maintained official-issuer reference. [Stellar asset controls](https://developers.stellar.org/docs/tokens/control-asset-access) · [SEP-1](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0001.md)

## Registry API

| Entry point | Access | Behavior |
| --- | --- | --- |
| `__constructor(admin)` | Host at deployment | Stores the initial publisher atomically with contract creation. |
| `admin()` | Public | Reads the current publisher. |
| `attest(asset_key, score, report_hash, scanned_at, score_version)` | Current publisher | Accepts a score in `0..=100` and replaces the previous value for that key. A successful refresh clears revocation. |
| `get(asset_key)` | Public | Returns the stored value, including its revocation flag, or `NotFound` for a missing key. |
| `revoke(asset_key)` | Current publisher | Sets `revoked` while preserving the stored report hash and scan time. |
| `set_admin(new_admin)` | Current publisher | Rotates the publisher address. |

`Attested` and `Revoked` use the asset key as an event topic. Persistent records have per-entry TTL in ledgers. Writes extend the TTL toward 1,555,200 ledgers when it falls below 900,000. Reads leave TTL alone. Archived persistent data can be restored, and ledger TTL remains separate from evidence freshness. The registry keeps the latest authorized write. Scan time is publisher-supplied, and consumers apply their own freshness window. [Stellar storage](https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage) · [Restoring archived entries](https://developers.stellar.org/docs/tools/cli/cookbook/restore-contract-storage)

The published source uses a constructor for fresh deployments. The historical testnet contract used a separate initialization call and has a different Wasm hash. [Soroban constructors](https://developers.stellar.org/docs/learn/fundamentals/contract-development/contract-interactions/cross-contract)

## Commitment versions

The historical publisher hashed a compact JSON score report containing chain, address, score, score version, risk summary, scan time and score breakdown. Its model version may be sanitized when written as a Soroban symbol. For example, the hyphen and dot in `stellar-intel-v0.1` become underscores in `stellar_intel_v0_1`.

The proposed Passport format can use the same 32-byte field while committing to structured issuer observations. A publisher using that format computes the digest of the exact exported Passport bytes. The consumer verifies those same bytes against the registry. The two synthetic examples use the same timestamp and differ only in `auth_revocable`. This isolates what the commitment can demonstrate. A live report should include the source and ledger context and mark unavailable observations explicitly.
