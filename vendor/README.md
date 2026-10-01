---
status: active
id: vendor-readme
kind: reference
scope: repository
read_when: advancing or auditing the vendor/tokscale-core consumer pin
last_verified: 2026-10-01
sources: [".gitmodules", "vendor/tokscale-core", "docs/knowledge/vendor-tokscale.md"]
---

# Shared Rust core pin

The private Syrtis build consumes
[`HCH725/tokscale-core-custom`](https://github.com/HCH725/tokscale-core-custom)
as its canonical Git submodule. That private engine is rebased onto the public
[`Nanako0129/tokscale-core`](https://github.com/Nanako0129/tokscale-core)
baseline; consumer integration rules are documented in
[`docs/knowledge/vendor-tokscale.md`](../docs/knowledge/vendor-tokscale.md).

| Field | Value |
|---|---|
| Path | `vendor/tokscale-core` |
| Repository | `https://github.com/HCH725/tokscale-core-custom.git` |
| Reviewed pin | `ad24f3c06b3c08893323e059bbae50ebbdde1441` |
| Public upstream baseline | `319ffa8ca75f6cd2bfaf96ae0d295a8fa618ec2c` |
| Upstream ledger | Immutable public [`UPSTREAM.md` at `319ffa8`](https://github.com/Nanako0129/tokscale-core/blob/319ffa8ca75f6cd2bfaf96ae0d295a8fa618ec2c/UPSTREAM.md) |
| Private downstream contract | Engine [`CUSTOM.md`](https://github.com/HCH725/tokscale-core-custom/blob/main/CUSTOM.md) |

## Ownership

The private shared-engine repository owns parsers, scanning, cache behavior,
pricing, and aggregation. Syrtis owns the gitlink, root `Cargo.lock`,
`crates/tb_core_ffi`, `Sources/CTB/include/ctb.h`, Swift code, and application
build wiring.

Do not edit shared Rust source from the consumer change. Land and independently
verify engine changes in `HCH725/tokscale-core-custom`, then advance this pin to
the reviewed private engine commit and run the Syrtis consumer gates.

## Checkout

Clone recursively:

```bash
git clone --recurse-submodules https://github.com/HCH725/TokenBar-custom.git
```

For an existing checkout:

```bash
git submodule update --init --recursive
```

The submodule must be clean and its checked-out `HEAD` must equal the gitlink
before building or releasing.
