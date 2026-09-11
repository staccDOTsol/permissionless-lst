# Integration handoff

## Positioning

A risk-capped memecoin launchpad on Solana, made w <3 by stacc, powered by [venue]. This is proposed positioning, with venue attribution only after agreement. Backing is denominated in SOL, trading and protocol fees apply, liquid redemption capacity is finite, and there is no guarantee against principal loss or contract failure.

Token: permissionlessLST (pLST), `99rvR9fXWZqc1hNrDQYkYKFVXZSUFWBa6XHkoCVt348`.
Application: https://eat.ag/lst/
Source: https://github.com/staccDOTsol/permissionless-lst
Controller candidate: `GSsMfxpbN3h7jwkhJkbZPErjFo22a6MAgr6npcp56KZc` (**not deployed**).

## Routing semantics

The controller holds LST reserves valued from validated stake-pool state. Exact-in consumes gross source atoms, subtracts that mint's epoch transfer fee, values the net reserve credit, and subtracts the output mint's fee from the user's final output. Exact-out inverts those fees. Use integer math; do not convert token quantities through f64. Quote from one coherent account snapshot and epoch. Slippage limits are net output / gross input. Account order is in the permissionless IDL.

pLST is a preferred hub asset. It is not the controller's LP receipt mint and does not need to be INF. Each mint keeps its actual legacy/Token-2022 program ID. Never apply transfer fees twice if a venue wrapper already does so.

The old `stkitr…` route and the existing `Swap::SanctumS` Jupiter enum are not automatic integrations for this new program. A venue must explicitly accept and wire the controller CPI.

## Jupiter

Official request form: https://support.jup.ag/requests/new/amm-integrators (Others).
Official interface and test kit: https://github.com/jup-ag/jupiter-amm-interface

The form was inspected; required fields include email, project name, Telegram, subject and description. Contact details are pending from the project owner. No request has been submitted.

The monorepo's `s-jup-interface` remains on 0.4. Its pure quote code and native instruction builders now understand transfer fees. A discovery service can supply `params = {"programId":"...", "lstList":[...]}` rather than relying on Sanctum's bundled catalog; recreate the AMM when discovery adds a new asset. No network I/O belongs inside the AMM implementation.

Before production submission: port to the current 0.6.1 interface, use its `Swap::Placeholder { data }` during test-kit verification, commit account/binary fixtures, prove quote/execution parity on the actual fork and mixed legacy/Token-2022 routes, publish a funded live pool and report audit status honestly. Existing legacy-interface tests are not a substitute for the current kit.

## OKX

Official integration documentation: https://web3.okx.com/zh-hant/build/dev-docs/dex-api/dex-integration

No OKX-specific adapter has been implemented or submitted. The swap API SDK is a consumer client, not the DEX integration adapter. Provide the venue-compatible Rust quote/account/discovery/CPI implementation and request the maintained Solana adapter interface through their integration process. Do not label old Sanctum routing as support for this fork.

## Titan

Official guidelines: https://titan-exchange.gitbook.io/titan/getting-started/titan-dex-integrations
Official template: https://github.com/Titan-Pathfinder/integration-template

The current template requires `TradingVenue`, account loading, exact-in quote math, raw-atom marginal price, instruction/route/CPI parity and LiteSVM tests. Titan's TokenInfo layer already handles transfer-fee metadata; avoid double charging it. A dedicated adapter and those tests remain to be implemented. No submission has been made.

## Other venues

DFlow is another candidate. Raydium is relevant to the launchpad's market liquidity and the founder's historical discussion; no current support or endorsement is assumed. Further submissions should use each venue's actual integration channel and report this controller as pre-deployment until it is funded and deployed.
