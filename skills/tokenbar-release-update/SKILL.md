---
name: tokenbar-release-update
description: Use when updating the private Syrtis build from an upstream release. Preflight, release-by-release delta review, engine-first rule, gates, bundle/install/rollback.
---

# Syrtis private release update

The production Syrtis on this machine is **not** a pure upstream installation. It is an upstream `Nanako0129/syrtis` release, reviewed and merged into the private canonical `HCH725/TokenBar-custom` repository, tagged `vX.Y.Z-custom.N`, plus the smallest private delta that keeps the downstream contract in [`CUSTOM.md`](../../CUSTOM.md) true. The private repository name is intentionally retained for continuity.

## Authority model (read first)

- This repo-tracked skill is the **canonical operational procedure** (how to perform an update).
- [`CUSTOM.md`](../../CUSTOM.md) is the **downstream contract** (what must remain true). [`docs/knowledge/`](../../docs/knowledge/README.md) owns the canonical project facts (architecture, workflow, verification, release, shared engine). On any conflict, **stop the update**, reconcile this skill with the canonical source, and only then continue.
- Never store secrets, tokens, credential values, or credential locations in this skill or anywhere in the repository.
- This tracked file is always the source of truth. Local entrypoints, symlinks, and agent-profile routing are machine-local concerns configured outside the repository, and are never recorded here.

## Source hierarchy

| Level | Repository / artifact | Authority |
|---|---|---|
| 1 | `Nanako0129/syrtis` upstream stable tag (commit, release notes, official artifact) | Top source of truth for release behavior |
| 2 | `HCH725/TokenBar-custom` (remote `origin`) — reviewed `main` plus the current accepted `vX.Y.Z-custom.N` tag | Source of truth for downstream policy; nothing is accepted until it passes acceptance and independent audit |
| 3 | `HCH725/tokscale-core-custom` — consumed as the `vendor/tokscale-core` gitlink | Owns parsers, scanning, cache, pricing, aggregation |
| 4 | `/Applications/Syrtis.app` — the installed private build | What actually runs; only ever replaced by a bundle built from level 2 |

- Production is built from an accepted private revision/tag — never by following `origin/main` head, an old worktree, a `git pull`, Homebrew, or the official Sparkle feed.
- `dist/Syrtis.app` is a disposable candidate/release artifact, built from the accepted revision and copied into `/Applications` only under install authorization; it is never production in place. `dist/selftest/Syrtis.app` deliberately shares the shipping name and identifier — it must never be installed.
- The engine's public release train is not a Syrtis release: new engine work is not part of the private Syrtis build until a consumer change advances the gitlink and passes the consumer gates.

## Fixed downstream contract (must survive every update)

Owned by [`CUSTOM.md`](../../CUSTOM.md); this skill only fixes how an update keeps it true.

| Contract | Update must not break | Required check |
|---|---|---|
| CatDesk ingestion | `~/.catdesk/usage.jsonl` stays an additional **Hermes** source: `client=hermes`, `model=catdesk-mcp`, `provider=catdesk`; tokens count in Hermes and grand totals; cost via the ledger `pricingModel` identity (legacy rows without it use the confirmed historical `gpt-5.6-sol` fallback); MCP direction is translated before pricing (`outputTokens` → model input, `inputTokens` → model output) | Engine + consumer tests for the CatDesk lane, then the authorized live probe |
| CatDesk is not a client | CatDesk never becomes a separate Syrtis client or attribution bucket | Report/graph assertions on the same lane |
| OpenCode Go pricing | Hermes `provider=opencode-go` rows price through the provider-scoped official usage-value table before the generic catalog; provider-reported cost stays authoritative; unknown Go models fall back to the existing pricing service; DeepSeek V4 keeps the official weekday UTC peak windows and off-peak weekends | Pricing fixtures plus the live probe where Go rows exist |
| Codex subscription equivalents | `openai-codex` rows with `cost_status=included` and `billing_mode=subscription_included`/`codex_responses` keep authoritative incremental cost `$0`; attribution/quota views use the separate recorded-model ChatGPT Work/Codex rate-card equivalent; quota depletion is never inferred from those dollars | Cost fixtures plus the live probe on Codex rows |
| No foreign writes | Syrtis never writes Hermes `state.db` or the CatDesk ledger; both stay read-only inputs | Release-delta inspection for new file-write paths (do not mutate those files to prove it) |
| No official auto-update | The private bundle never follows `Nanako0129`'s Sparkle feed: `scripts/bundle.sh` writes `TokenBarOfficialUpdatesEnabled=false` and `UpdaterService.isAvailable` requires it; autostart stays available independently | Assert both private hunks survived an upstream change to those files, and re-check the installed `Info.plist` key after install |
| No Homebrew ownership | Once the private build is installed, the official Homebrew cask must not own `/Applications/Syrtis.app`; `brew upgrade`/`brew install` are never part of this flow | `brew list --cask`, Caskroom receipt, and the installed app's provenance |
| Shared-engine boundary | Shared Rust lands in `HCH725/tokscale-core-custom` first; Syrtis owns only the gitlink, FFI, C ABI, Swift, and build wiring | Gitlink review plus consumer gates |

The private delta is **not** disposable scaffolding: the guardrail hunks above, the CatDesk/Go/Codex lanes, and their fixtures must be re-derived onto every new upstream base. Never copy an old tree wholesale over a newer release.

## Preflight before any update

Never rely on memory, a stale worktree, or a previous session's values. Work in a clean worktree; a dirty checkout is read-only.

```bash
cd "$(git rev-parse --show-toplevel)"; git status --porcelain   # must be empty
git fetch --tags origin && git fetch --tags upstream
git remote -v                                                  # origin=HCH725/TokenBar-custom, upstream=Nanako0129/syrtis
git log --oneline -1 origin/main

# level 2: current accepted custom tag and the build number it would produce
git tag -l 'v*-custom.*' --sort=-v:refname | head -1
git rev-list --count <custom-tag>

# level 4: installed production identity
/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier'            /Applications/Syrtis.app/Contents/Info.plist
/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString'    /Applications/Syrtis.app/Contents/Info.plist
/usr/libexec/PlistBuddy -c 'Print :CFBundleVersion'               /Applications/Syrtis.app/Contents/Info.plist
/usr/libexec/PlistBuddy -c 'Print :TokenBarOfficialUpdatesEnabled' /Applications/Syrtis.app/Contents/Info.plist  # must be false
codesign -dv --verbose=4 /Applications/Syrtis.app 2>&1 | grep -E 'Identifier|Signature|CDHash'
shasum -a 256 /Applications/Syrtis.app/Contents/MacOS/Syrtis

# level 3: submodule pin
git submodule status vendor/tokscale-core                      # no '-'/'+' prefix; checked-out HEAD == gitlink
git -C vendor/tokscale-core rev-parse HEAD
git -C vendor/tokscale-core status --porcelain                 # must be empty
git ls-tree HEAD vendor/tokscale-core                          # must equal the recorded pin in vendor/README.md
```

1. Record every value with its source; do not carry a value forward from an earlier run.
2. In the clean-tag release workflow below, `BUILD=$(git rev-list --count <accepted-tag>)` is the build-number authority because `release.yml` computes `git rev-list --count HEAD` on that same clean tag checkout. Do **not** use `CFBundleVersion` alone to attribute an already-installed historical/private build: `scripts/bundle.sh` accepts the build number as caller-supplied argument 2. Attribute an installed private artifact using the accepted tag/revision record together with the recorded bundle/binary SHA-256 and build/install provenance; if those sources disagree or cannot establish one revision, mark the installed build unattributed and stop for an explicit decision. The newest custom tag is not automatically what is installed.
3. Rollback artifact: identify the previous accepted app bundle and record its path, version, build, and binary sha256. If no usable artifact exists, the install step below MUST create the timestamped backup before the swap; without it there is no rollback path and the update does not proceed.
4. If the gitlink and the `vendor/README.md` recorded pin disagree, stop and reconcile the pin record first (an advanced gitlink with a stale recorded pin is an unreviewed engine change).
5. If this skill and `CUSTOM.md`/`docs/knowledge/` disagree, stop and reconcile before any further step.

## Required release workflow

1. **Release-by-release upstream review.** Take each upstream stable release in order; for every one, read the release notes and the *actual* diff, and classify each change (already present / take / adapt / defer) using the selective-port method in [`docs/knowledge/vendor-tokscale.md`](../../docs/knowledge/vendor-tokscale.md). Never skip a release's delta review, and never treat the change list as additive-only — unrelated fixes can ride along in a release.
2. **Smallest private delta.** Re-derive only the private hunks still required onto the new upstream base, in a clean worktree. Do not copy old source files wholesale over newer source; preserve upstream features and adapt to the new architecture.
3. **Engine-first for shared behavior.** If the change touches parsers, scanning, cache identity/schema, pricing, or aggregation, land and verify it in `HCH725/tokscale-core-custom` first (its own tests, review, and `UPSTREAM.md` ledger), then advance the Syrtis gitlink to the reviewed engine commit and re-assert the whole private contract. Files inside `vendor/tokscale-core` are never edited from the consumer.
4. **Consumer verification.** Run the gates in the next section on the candidate revision; record command output, not summaries.
5. **Cache / parser consequence.** Decide and record before installing: does the release change parser output, dedup keys, attribution, or serialized layout (schema bump → cold rebuild for existing users, with a same-fingerprint stale-cache regression), or only post-cache pricing/report arithmetic and independently fingerprinted sources (no schema change)? The engine owns the cache-format/parser-identity counters — do not mirror or infer them. Never assume warm-cache parity you did not observe, and tell the operator the rebuild cost (time/disk) the update will cause.
6. **Live probe (authorized per run).** With explicit authorization, verify on the production surface that Hermes/grand totals include the CatDesk rows attributed to Hermes, OpenCode Go rows price through the provider-scoped table, and Codex `included` rows keep `$0` raw cost with the separate equivalent display. Record observed values and the date. An authorization prompt, a missing local fixture, or empty private data is an environmental limitation recorded separately — never a fabricated PASS and never a reason to invent numbers.
7. **Independent audit.** One implementer profile implements; an independent auditor (a separate profile or an independent review path) verifies the delta, evidence, and contract; the implementer never declares audit PASS. A FAIL returns to the implementer, is fixed, and is re-audited before acceptance.
8. **Acceptance is a separate authorization.** Local verification and builds are implementation work; commit, push, PR, merge, tag, release, install, and production replacement each require the applicable explicit user authorization (see [`docs/knowledge/workflow.md`](../../docs/knowledge/workflow.md)). Finishing implementation is not authorization.

## Verification gates (from the canonical contract)

Gates are owned by [`docs/knowledge/verification.md`](../../docs/knowledge/verification.md); this is the candidate-run checklist, not a replacement.

```bash
cargo fmt --all -- --check
cargo test
cargo clippy --workspace --all-targets
make build
make selftest                  # = swift run Syrtis --selftest -AppleLanguages "(en)"
swift run Syrtis --smoke
make selftest-bundled          # same suite in the shipping configuration; not a superset of selftest

python3 scripts/check_knowledge.py --self-test
python3 -m py_compile scripts/check_knowledge.py
python3 scripts/check_knowledge.py
make check-docs
git diff --check origin/main...HEAD
```

| Evidence layer | What it can prove |
|---|---|
| Hermetic fixture | Old/new behavior diverges under the triggering condition (required for any parser, cache, or pricing change) |
| Core / consumer tests | Parser, fold, schema, and attribution contracts are stable |
| FFI smoke / selftest / bundled selftest | Rust → C ABI → Swift decodes end to end, in both debug and shipping configurations |
| Docs gate | Knowledge tree, links, privacy scan, and whitespace are intact |
| Live probe | The real Hermes/CatDesk data flow and production surface behave — with authorization for that run |

A `make selftest` failure caused by a non-English system locale is a harness artifact (`make selftest` pins `en`), not a product failure; record it as environmental. A green live run without a fixture does not close a data-dependent correctness issue.

## Bundle, install, acceptance, rollback

Install is the last and only step that touches production; it happens after acceptance, under explicit authorization, and it is never automatic.

1. **Bundle** from a clean worktree at the accepted tag. A fresh worktree has no `.build/`, so the Makefile's stale-artifact guards cannot be defeated. If reusing a worktree, run `make bundle` (guards included) first, or remove `.build/`.
   ```bash
   VERSION=<tag without -custom.N>          # e.g. v1.14.4-custom.4 -> 1.14.4
   BUILD=$(git rev-list --count <accepted-tag>)   # authority for this clean-tag release workflow; not a historical installed-build provenance decoder
   scripts/bundle.sh "$VERSION" "$BUILD"
   shasum -a 256 dist/Syrtis.app/Contents/MacOS/Syrtis
   ```
   `scripts/bundle.sh` must fail if the bundled Sparkle lacks installed-name normalization; a bundle that skipped that assertion is not installable.
2. **Backup the previous accepted app** (timestamped, outside the repository) and record its version, build, and binary sha256 next to the new digest. Keep it until the new build passes acceptance.
   ```bash
   ditto /Applications/Syrtis.app "$HOME/Syrtis-backups/Syrtis-$(date +%Y%m%d-%H%M%S).app"
   ```
3. **Install the candidate**: quit the running app, replace `/Applications/Syrtis.app` with the accepted bundle, then verify identity before relaunching — identifier `com.nyanako.tokenbar`, expected version/build, `TokenBarOfficialUpdatesEnabled=false`, ad-hoc signature intact, binary sha256 equal to the bundle you just built. The only installable artifact is the bundle built from the accepted revision in step 1; `dist/selftest/Syrtis.app` must never be installed, and the app is never installed or advanced through Homebrew or its own updater.
4. **Production acceptance**: launch the installed app and run the release-specific smoke list plus the fixed contract probes on the real production surface. Confirm it is running (menu-bar process present, no crash loop) and that autostart state is unchanged.
5. **Rollback is explicit and manual**: restore the previous accepted backup over `/Applications/Syrtis.app` and re-verify identity. There is **no auto-rollback** and no implicit rollback on a failed gate; a failed acceptance is reported as PARTIAL and the decision belongs to the operator. Never roll back to an unattributed build and never "roll forward" to the newest tag as a fix.
6. **Clean up only after production acceptance passes.** Remove artifacts created by this update: disposable upgrade/audit worktrees or clones, `dist/` and `dist/selftest/` bundles, Swift `.build/`, Rust `target/` directories (including submodule targets), downloaded DMG/ZIP/release assets, and update-specific audit/staging directories. Run `git worktree prune`, then record `du`/`df` before-versus-after evidence. Preserve canonical source/tags, user configuration/data, and the rollback backup; never delete the rollback backup before production acceptance, and never sweep shared caches or unrelated files merely to reclaim space.

## Release-specific smoke checklist

Derive this from each release's notes and the reviewed delta; never reuse the previous release's list as the baseline.

| Item | Source (release note / delta line) | Method (production surface) | Observed |
|---|---|---|---|
| One check per user-visible change | `<line from notes/diff>` | Installed app, popover/dashboard/Settings as the change requires | `<value, not a sentence>` |
| CatDesk lane | contract probe | Totals include CatDesk-attributed Hermes rows | `<numbers>` |
| OpenCode Go pricing | contract probe | Go rows use provider-scoped pricing | `<numbers>` |
| Codex equivalents | contract probe | `$0` raw cost + separate equivalent, no quota inference | `<numbers>` |
| Updater guard | `Info.plist` key | Official feed disabled on the installed bundle | `false` |

Every row gets a value from a real readback. A release that cannot be checked on the production surface records that limitation explicitly instead of a PASS.

## Recorded pitfalls

- An upstream release can silently drop a private hunk (for example an upstream rewrite of `scripts/bundle.sh` that omits the guardrail key). Re-assert both the bundle hunk and the `UpdaterService` reader after any change to those files.
- An advanced gitlink with a stale `vendor/README.md` pin, or conflicting/missing tag/revision, SHA-256, and build/install provenance for an installed private artifact, means provenance is unproven. `CFBundleVersion` is metadata, not source-commit proof; reconcile before building/installing or accepting the artifact.
- A successful upstream checksum is evidence about the upstream artifact only, never about local custom requirements.
- `dist/selftest/Syrtis.app` shares the shipping identity with `dist/Syrtis.app` but is a different gate; it must never be installed. The accepted-revision bundle in `dist/` leaves that directory only as a copy into `/Applications` under install authorization.
- This skill is not covered by `scripts/check_knowledge.py` (the script validates `docs/knowledge/`, the adapters, and the ledger). Keep its relative links valid by hand when editing it.

## Scope guard

One procedure, one file. Do not add a manager, service, controller, daemon, install helper, or a second SOP; do not restate project facts that belong in `docs/knowledge/`. `CUSTOM.md` and `AGENTS.md` carry only a routing reference to this file.
