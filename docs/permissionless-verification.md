# Verification — September 11, 2026

## Passed

- **63 default-feature regression tests:** 52 existing controller tests, 3 new fee-account tests, 4 transfer-fee/liquidity tests, and 4 existing Jupiter 0.4 quote/execution tests.
- **11 permissionless SBF execution tests:** 4 admission tests, 3 fee-account tests, and 4 swap/liquidity tests. ProgramTest loads `target/permissionless/s_controller.so` as the controller; SPL Token, Token-2022 and ATA processors execute as native test-runtime dependencies.
- Token-2022 transfer-fee ATA initialization by an arbitrary payer, repeat idempotence, prefunded System-account initialization, legacy Token compatibility, and wrong ATA rejection.
- Public listing with no admin signature. Spoofed backing owners, arbitrary valuation tags and freeze-authority mints are rejected.
- Exact-in and exact-out with 0.5% Token-2022 fees on both legs. Net minimum-output failure checks the specific slippage error and verifies balance rollback.
- LP deposit of 1,000,000 atoms credits 995,000 shares; redemption returns 990,025 atoms. Withheld fees are not counted as backing or LP principal.
- In the permissionless execution tests, external valuation calculator programs are absent. Supported stake-pool math executes inside the controller.

The swap fixtures are synthetic SPL-family stake pools at a 1:1 rate. Default-feature accounting tests use a 1:1 calculator test double; permissionless tests use the actual compiled internal stake-pool math. These are not mainnet executions, an audit, or current Jupiter 0.6 test-kit certification. The old Jupiter tests exercise their legacy fixture routes, not a venue integration of the new program.

## Build

Artifact details and SHA-256 are in `build-artifact.json`. Built with `cargo-build-sbf` from Solana 4.3.0-rc.0, platform-tools 1.57, and the committed lockfile. The artifact is 263,984 bytes.

The initial dependency build emitted stack-frame diagnostics in SPL Token-2022 confidential-transfer code. Confidential extensions are excluded from permissionless admission; those flows are not tested. All eleven supported controller SBF test paths completed successfully. This does not establish safety of untested code or replace an independent audit.

The historical upstream verifiable-build manifest is retained separately. There is no independent verified-build attestation for this fork and no deployed controller to match yet.

## Deployment and integration limits

At the recorded mainnet check, program-data rent alone was **1.34191756 SOL**. Deployment was not attempted. Loader/program account rent, transaction fees and initialization/liquidity budgets are additional. Underlying pLST metadata was renamed independently; that existing mint remains live.

The controller is not published on mainnet, no new S pool is initialized or funded, and no aggregator has accepted or activated this controller. Jupiter's legacy adapter update is covered by its existing tests; the current interface migration and venue-specific production submissions remain outstanding.

Admission covers supported SOL-backed stake-pool ABIs, not arbitrary unverifiable asset prices. Current epoch, token program, authority and backing checks remain. Deposit or redemption authorities on an underlying stake pool can still constrain that pool's own operations; the fork does not remove another program's authority requirements.
