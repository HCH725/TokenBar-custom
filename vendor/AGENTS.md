# Shared-engine task routing

Read [`vendor/README.md`](README.md) before changing the
`vendor/tokscale-core` pin. Then read
[`docs/knowledge/vendor-tokscale.md`](../docs/knowledge/vendor-tokscale.md) for
the shared-engine boundary and
[`docs/knowledge/verification.md`](../docs/knowledge/verification.md) for
required consumer evidence. Engine implementation work lands in the private
`HCH725/tokscale-core-custom` repository and uses the public
[`AGENTS.md` at `319ffa8`](https://github.com/Nanako0129/tokscale-core/blob/319ffa8ca75f6cd2bfaf96ae0d295a8fa618ec2c/AGENTS.md)
as upstream baseline guidance.

## Invariants

| Boundary | Rule |
|---|---|
| Pin | `vendor/tokscale-core` must be a clean gitlink at the reviewed engine commit recorded in `vendor/README.md`. |
| Engine ownership | Do not edit shared Rust source as an untracked consumer-only patch. Land and verify the change in `HCH725/tokscale-core-custom`, then update this consumer pin. |
| Consumer ownership | Syrtis continues to own `crates/tb_core_ffi`, `Sources/CTB/include/ctb.h`, Swift code, build wiring, and the root `Cargo.lock`. |
| Parity | A pin update must verify materialized and streaming behavior, cache/schema consequences, FFI mappings, and the Rust-to-Swift app gates that the change can affect. |
| Ledger | Public `UPSTREAM.md` owns the upstream baseline history; the private engine `CUSTOM.md` owns the surviving downstream contract. `vendor/README.md` records Syrtis's private source and pin. |

No shared-engine task may push or merge by implication from a plan. Return the
diff and verification evidence to the user for authorization.
