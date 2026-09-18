# Shared Rust core pin

TokenBar consumes the private canonical
[`HCH725/tokscale-core-custom`](https://github.com/HCH725/tokscale-core-custom)
repository as a Git submodule; that repository tracks the public `Nanako0129/tokscale-core` baseline. Consumer integration rules are documented in
[`docs/knowledge/vendor-tokscale.md`](../docs/knowledge/vendor-tokscale.md).

| Field | Value |
|---|---|
| Path | `vendor/tokscale-core` |
| Repository | `https://github.com/HCH725/tokscale-core-custom.git` |
| Reviewed pin | `dad446d40dfd90ddfb049f9f2d10d1ae20c9baad` |
| Upstream and local-patch ledger | Immutable [`UPSTREAM.md`](https://github.com/HCH725/tokscale-core-custom/blob/dad446d40dfd90ddfb049f9f2d10d1ae20c9baad/UPSTREAM.md) plus private contract [`CUSTOM.md`](https://github.com/HCH725/tokscale-core-custom/blob/dad446d40dfd90ddfb049f9f2d10d1ae20c9baad/CUSTOM.md) |

## Ownership

The shared repository owns parsers, scanning, cache behavior, pricing, and
aggregation. TokenBar owns the gitlink, root `Cargo.lock`,
`crates/tb_core_ffi`, `Sources/CTB/include/ctb.h`, Swift code, and application
build wiring.

Do not edit shared Rust source inside the TokenBar submodule. Land and verify
engine changes in `tokscale-core`, then advance this pin to the reviewed engine
commit and run the TokenBar consumer gates.

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
