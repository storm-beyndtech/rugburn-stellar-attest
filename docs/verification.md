# Verify a report

## Identify the registry and asset

Choose the registry and network from a trusted source. Use the registry’s `admin()` as a check on who can publish, revoke, or rotate the publisher at the time of the read. For a classic issued asset, derive the key from the exact asset code and issuer StrKey.

```sh
printf '%s' 'stellar:USDC:GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN' | sha256sum
```

Expected key

```
b6af926fc3e9e1cd79421959f7acc2cd634377dc310a923de765b72277661850
```

On a live registry, use `get(asset_key)` in Stellar Lab or the [Stellar CLI](https://developers.stellar.org/docs/tools/cli). With the CLI, use `--send=no` before `--` to simulate the call without submitting a transaction. Check the record’s `revoked`, `scanned_at`, and `score_version` fields. `NotFound` means the contract lookup found no record. Treat archive and RPC errors as unavailable evidence.

## Compare the report bytes

Obtain the original exported file from the publisher. Hash it as received, including its exact encoding and line endings. Reformatting or reserializing the JSON changes the digest.

```sh
sha256sum passport.json
```

On Windows PowerShell

```powershell
(Get-FileHash -Algorithm SHA256 -LiteralPath ./passport.json).Hash.ToLowerInvariant()
```

Compare the digest with the hex representation of `report_hash` returned by `get`. Review the ledger observations independently and apply your own freshness policy before using the assessment in an acceptance decision.

## Historical testnet record

The [historical Rugburn testnet registry](https://lab.stellar.org/r/testnet/contract/CA3WLC2KFMJ2KQ5HHWGP5WRS3LX3HJF5ZVW2APRBRUYDV73OUGYG5P2K) used an earlier initialization interface. On 21 November 2026, Stellar’s public testnet RPC returned the contract instance, Wasm entry, and the USDC record for the key above with `liveUntilLedgerSeq` equal to `0` at ledger `5,846,489`. These are archived entries. [Stellar RPC `getLedgerEntries`](https://developers.stellar.org/docs/data/apis/rpc/api-reference/methods/getLedgerEntries)

| Field | Archived value |
| --- | --- |
| Score | `45` |
| Report hash | `3fa4363a7cdfb60ca7b024e0d6c63591894d85772c4ebb0f795e4146e3ab69cc` |
| Scan time | `1783178884` (Unix seconds) |
| Model version | `stellar_intel_v0_1` |
| Revoked at last write | `false` |

The archived record proves that the digest and associated values are present in the registry’s historical state. Verifying the underlying report requires its original bytes. The offline examples below have their own hashes.

## Offline fixtures

`examples/legacy-report.json` illustrates the historical score-report format. `examples/passport-before.json` and `examples/passport-after.json` are synthetic reports. They share a timestamp and change only the `auth_revocable` observation, so a digest difference can be attributed to that field. Each fixture has no terminating newline.

```sh
sha256sum -c --strict examples/SHA256SUMS
```

`cargo test --locked` also writes both fixture digests to a local Soroban test host and reads them back. Set the Wasm path as shown in the README to run the same tests against the compiled contract. The fixtures have never been submitted to testnet.
