# S controller embedded in the eat.ag engine

Source: https://github.com/igneous-labs/S at `66de438b9e049aacc193b5795d26ac055d86d770`.
Permissionless / Token-2022 changes: https://github.com/staccDOTsol/permissionless-lst
at `f9dc0a07a3de9b71c4b7322be85d9e9f6833e82a`.

This is the actual S controller source, including SwapExactIn, SwapExactOut,
AddLiquidity, RemoveLiquidity, SyncSolValue and its reserve/list/fee PDA state.
It is linked into `EMd4gqQTubN5wycXoVmy9PyybXATmyYrdvsb4B5QEYvC` with
`permissionless` and `no-entrypoint`. The permissionless program constants use
that existing engine ID. S's instruction bytes are prefixed with byte 37 by
external callers; the dispatcher removes the prefix before calling S's processor.

The native Pinocchio engine instructions retain tags 0–36. S's native inner
discriminators and state layouts are unchanged. This directory retains upstream
provenance; the surrounding repository's Apache license is not a relicensing of S.
