# permissionlessLST · eat.ag

A fork of [igneous-labs/S](https://github.com/igneous-labs/S), based on commit `66de438b9e049aacc193b5795d26ac055d86d770`.

Proposed launchpad pitch: **a risk-capped memecoin launchpad on Solana, made w <3 by stacc, powered by [integrating venue].** “Risk-capped” describes the intended SOL-backing design, not guaranteed principal, a dollar floor, or protection against fees, liquidity constraints and contract failures. Venue attribution is conditional on integration; no endorsement is claimed.

The existing SOL-backed token is **permissionlessLST / pLST** at `99rvR9fXWZqc1hNrDQYkYKFVXZSUFWBa6XHkoCVt348`. [The application](https://eat.ag/lst/) and renamed metadata are live. **This new controller is not deployed and no aggregator has accepted this fork.**

## What changes

Build with `--features permissionless`:

- Any payer can list a supported, verifiably backed LST. No curator signature or per-token pricing registration.
- Opcode **23, CreateFeeAccount** initializes the mint-specific protocol-fee ATA, including Token-2022 transfer-fee extensions. It works before listing, is idempotent, and accepts a prefunded System-owned ATA.
- Each leg uses its own token program: legacy WSOL can trade against a Token-2022 LST.
- Swap and liquidity accounting use net transfers. Exact-out quotes gross up the wallet debit; minimum-output checks use the actual destination credit.
- Supported stake-pool valuation runs inside the controller. It does not call Sanctum's calculator or depend on a manager-approved upgrade slot.
- Pricing is fixed at backing value with no additional controller pricing fee. Pool and token administrative pause gates, curator removal, external pricing changes and privileged rebalancing are unavailable.

Ownership, PDA derivation, mint authority, current-epoch accounting, reserve solvency, signatures and slippage checks remain enforced. Initial admission supports native legacy WSOL and the SPL / Sanctum SPL / Sanctum SPL Multi stake-pool ABIs. TransferFeeConfig and metadata extensions are supported; hooks, permanent delegates, freeze authorities and unsupported mint extensions are rejected from pool admission.

This is an S multi-LST reserve pool, not a change to the previously deployed `stkitr…` router. Its fee accumulator is an **ATA under this controller's `protocol-fee` PDA**. It is neither an old router LST-fee PDA nor the stake pool's manager-fee account.

## Hub and fees

pLST can be a reserve asset and the common asset across launchpad markets. INF is not required as an intermediate token. The S pool needs a **separate LP receipt mint**; the existing pLST remains a claim on its original SOL stake pool. Arbitrary memecoins do not become SOL-backed by listing them in this pool; their AMMs can pair against pLST.

Existing pLST policy is unchanged: 0.5% transfer, 0.25% mint/redemption, 50% of mint fees to the referrer (deployer by default), and the existing treasury burn/reward split. Zero additional controller pricing fees do not waive these fees. An empty reserve does not provide a redemption route; pool liquidity must be funded before swaps can execute.

## Build and verification

```sh
cargo test -p s-controller -p s-jup-interface --tests -- --test-threads=1
cargo test -p s-controller --features permissionless \
  --test permissionless_fee --test permissionless_admission \
  --test transfer_fee_routes -- --test-threads=1
cargo-build-sbf --manifest-path programs/s-controller/Cargo.toml \
  --features permissionless --sbf-out-dir target/permissionless
BPF_OUT_DIR="$PWD/target/permissionless" cargo test -p s-controller \
  --features permissionless --test permissionless_fee \
  --test permissionless_admission --test transfer_fee_routes -- --test-threads=1
```

On Apple Silicon, if the bundled LLVM cannot find system headers, set `CC_aarch64_apple_darwin=/usr/bin/clang HOST_CC=/usr/bin/clang` for `cargo-build-sbf`. The committed lockfile updates old ahash releases for modern SBF toolchains. See [verification](docs/permissionless-verification.md) for scope and limitations.

The permissionless build address is `GSsMfxpbN3h7jwkhJkbZPErjFo22a6MAgr6npcp56KZc`. It is an **allocated, undeployed address**, not an existing mainnet program. The default feature set retains upstream IDs for legacy fixture tests; do not deploy that build as this fork. New forks must generate their own program keypair and change both feature-gated IDs. Private keys are not included.

## Interfaces and integration

- [Permissionless IDL](idl/permissionless_controller.json), retaining upstream discriminator numbers.
- [Fee-account instruction builder](libs/s-controller-lib/src/instructions/create_fee_account.rs).
- [Integration handoff and status](docs/integrations.md).
- `s-jup-interface` is the existing **Jupiter 0.4** adapter, updated for transfer fees and supplied discovery catalogs. It has **not yet been ported to Jupiter 0.6.1** or verified with that interface's current test kit. Its old Sanctum route enum is not an accepted deployment of this new program.

Initial setup remains signed by the fork deployer. The fee beneficiary/admin controls and program upgrade authority are not automatically renounced; this is permissionless route admission, not a claim of complete immutability. No new audit or independent verified-build attestation is claimed. Historical upstream build attestations are preserved under `docs/upstream-verified-build.json`, not represented as verification of these changes.

[Upstream README and archive notice](docs/upstream-README.md). Existing source provenance and notices are retained.
