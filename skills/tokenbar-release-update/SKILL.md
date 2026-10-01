---
name: tokenbar-release-update
description: Use when updating the private Syrtis build from an upstream release. GitHub-first, engine-first, independent audit, production acceptance, rollback, and post-audit cleanup.
---

# Syrtis private release update

The production Syrtis on this machine is **not** a pure upstream installation. It is an upstream `Nanako0129/syrtis` release, reviewed and merged into the private canonical `HCH725/TokenBar-custom` repository, plus the smallest private delta that keeps the downstream contract in [`CUSTOM.md`](../../CUSTOM.md) true. The accepted **merged private revision** is the deployment authority; a custom tag is optional and is created only when the release workflow explicitly requires one. The private repository name is intentionally retained for continuity.

## Authority model (read first)

- This repo-tracked skill is the **canonical operational procedure** (how to perform an update).
- [`CUSTOM.md`](../../CUSTOM.md) is the **downstream contract** (what must remain true). [`docs/knowledge/`](../../docs/knowledge/README.md) owns the canonical project facts (architecture, workflow, verification, release, shared engine). On any conflict, **stop the update**, reconcile this skill with the canonical source, and only then continue.
- Never store secrets, tokens, credential values, or credential locations in this skill or anywhere in the repository.
- This tracked file is always the source of truth. Local entrypoints, symlinks, and agent-profile routing are machine-local concerns configured outside the repository, and are never recorded here.
- **Completion invariant:** an upgrade is not complete because a build passed. It is complete only when **private repos are reviewed/audited and merged → production is independently accepted → disposable upgrade artifacts are cleaned → the canonical repo/submodule state is clean**.

## Source hierarchy

| Level | Repository / artifact | Authority |
|---|---|---|
| 1 | `Nanako0129/syrtis` upstream stable tag (commit, release notes, official artifact) | Top source of truth for release behavior |
| 2 | `HCH725/TokenBar-custom` (remote `origin`) — reviewed and merged `main` revision; optional private release tag only when explicitly required | Source of truth for downstream policy; nothing reaches production until repo review/audit and merge are complete |
| 3 | `HCH725/tokscale-core-custom` — consumed as the `vendor/tokscale-core` gitlink | Owns parsers, scanning, cache, pricing, aggregation |
| 4 | `/Applications/Syrtis.app` — the installed private build | What actually runs; only ever replaced by a bundle built from level 2 |

- Production is built from an exact accepted private merge revision (or an explicitly accepted release tag pointing to it) — never from an unreviewed branch head, an old worktree, Homebrew, or the official Sparkle feed.
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

# level 2: current canonical accepted private revision
git rev-parse origin/main

# Optional provenance reference only — a tag is not the deployment authority unless explicitly accepted
git tag -l 'v*-custom.*' --sort=-v:refname | head -1

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
2. In the clean accepted-revision workflow below, `BUILD=$(git rev-list --count <accepted-revision>)` is the build-number authority because the bundle script accepts the build number as caller-supplied argument 2. Do **not** use `CFBundleVersion` alone to attribute an already-installed historical/private build. Attribute an installed private artifact using the accepted merged revision (or explicitly accepted tag) together with the recorded bundle/binary SHA-256 and build/install provenance; if those sources disagree or cannot establish one revision, mark the installed build unattributed and stop for an explicit decision. The newest tag is never automatically what is installed.
3. Rollback artifact: identify the previous accepted app bundle and record its path, version, build, and binary sha256. If no usable artifact exists, the install step below MUST create the timestamped backup before the swap; without it there is no rollback path and the update does not proceed.
4. If the gitlink and the `vendor/README.md` recorded pin disagree, stop and reconcile the pin record first (an advanced gitlink with a stale recorded pin is an unreviewed engine change).
5. If this skill and `CUSTOM.md`/`docs/knowledge/` disagree, stop and reconcile before any further step.

## Required release workflow

1. **Release-by-release upstream review.** Take each upstream stable release in order; for every one, read the release notes and the *actual* diff, and classify each change (already present / take / adapt / defer) using the selective-port method in [`docs/knowledge/vendor-tokscale.md`](../../docs/knowledge/vendor-tokscale.md). Never skip a release's delta review, and never treat the change list as additive-only — unrelated fixes can ride along in a release.
2. **Smallest private delta.** Re-derive only the private hunks still required onto the new upstream base, in a clean worktree. Do not copy old source files wholesale over newer source; preserve upstream features and adapt to the new architecture.
3. **Engine-first for shared behavior.** If the change touches parsers, scanning, cache identity/schema, pricing, or aggregation, land and verify it in `HCH725/tokscale-core-custom` first. The engine gets its own PR, CI, independent audit, remediation/re-audit if needed, and merge. Only then advance the Syrtis gitlink to the accepted private engine commit. Files inside `vendor/tokscale-core` are never edited from the consumer.
4. **Consumer verification.** Run the gates in the next section on the candidate revision; record command output, not summaries. Distinguish large upstream rename/assets churn from the actual downstream delta before judging whether the private customization is still thin.
5. **GitHub-first PR and CI.** Push the consumer candidate to a branch and open a PR before production work. CI, review, and independent audit must all close before merge. For the private `vendor/tokscale-core` submodule, use a **read-only deploy key scoped only to `HCH725/tokscale-core-custom`** (stored as an Actions secret); do not rely on the parent repo's default `GITHUB_TOKEN` to read another private repo. Parent-history fetches used by knowledge/release checks must use `fetch.recurseSubmodules=false` so historical superproject commits do not ask the private engine remote for old public gitlinks. Never print or commit the private key.
6. **Independent consumer audit.** One implementer profile implements; an independent auditor verifies the private delta, evidence, engine pin, updater/autostart guard, and upstream-first architecture. The implementer never declares audit PASS. A FAIL returns to remediation and a narrowly scoped re-audit. External code review may supplement this but does not replace the independent auditor.
7. **Merge before local deployment.** Only an audited, CI-green PR is merged into private `main`. Production deployment is built from that exact merged revision. Do not create a custom tag merely out of habit: if `release.yml` treats `v*` tags as formal release triggers, tagging is a separate explicitly authorized release action and may be intentionally omitted.
8. **Cache / parser consequence.** Decide and record before installing: does the release change parser output, dedup keys, attribution, or serialized layout (schema bump → cold rebuild for existing users, with a same-fingerprint stale-cache regression), or only post-cache pricing/report arithmetic and independently fingerprinted sources (no schema change)? The engine owns the cache-format/parser-identity counters — do not mirror or infer them. Never assume warm-cache parity you did not observe, and tell the operator the rebuild cost (time/disk) the update will cause.
9. **Production deployment and live probe.** After merge and explicit authorization, build from a clean detached worktree at the exact accepted revision, create the rollback backup, install, then verify production identity/hash/process. Live private-data probes are read-only. Missing credentials or empty private data are environmental limitations — never fabricated PASS/FAIL. GUI/login-item tools that hang are also limitations; stop the probe instead of turning acceptance into a debugging project.
10. **Independent production audit.** After deployment evidence is complete, a separate auditor re-reads the installed source revision, engine pin, version/build, updater flag, binary SHA, process path/stability, rollback backup, Homebrew ownership, and updater/autostart contract. **Cleanup is forbidden until this production audit returns PASS.**
11. **Cleanup and canonical-state check.** After production audit PASS, remove only update-created worktrees/build artifacts/staging files, run `git worktree prune`, measure before/after disk usage, then explicitly restore/check the canonical submodule working tree against the main gitlink and require a clean `git status`. Preserve user data, shared caches, and rollback backup unless a later explicit retention decision says otherwise.
12. **Acceptance is a separate authorization.** Local verification and builds are implementation work; commit, push, PR, merge, tag, release, install, and production replacement each require the applicable explicit user authorization (see [`docs/knowledge/workflow.md`](../../docs/knowledge/workflow.md)). Finishing implementation is not authorization.

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

Install is the last implementation step that touches production; it happens only after the private repo PR is audited, CI-green, merged, and explicitly authorized. Production acceptance and an independent production audit come after install; deployment is never automatic.

1. **Bundle from the exact accepted merged revision.** Create a clean detached worktree at the merged private `main` commit; initialize the private engine submodule and verify its checked-out HEAD equals the gitlink before building. A fresh worktree avoids stale `.build/`/`target/` state.
   ```bash
   ACCEPTED=<merged-private-revision>
   VERSION=<upstream release version>        # e.g. 2.2.0
   BUILD=$(git rev-list --count "$ACCEPTED")
   scripts/bundle.sh "$VERSION" "$BUILD"
   shasum -a 256 dist/Syrtis.app/Contents/MacOS/Syrtis
   ```
   `scripts/bundle.sh` must fail if the bundled Sparkle lacks installed-name normalization; a bundle that skipped that assertion is not installable. Do not create a release tag merely to obtain a build number.
2. **Verify the candidate before touching production.** Record bundle identifier, version/build, `TokenBarOfficialUpdatesEnabled=false`, codesign status, and candidate binary SHA-256. Ensure the bundle is `dist/Syrtis.app`, not `dist/selftest/Syrtis.app`.
3. **Backup the previous accepted app** (timestamped, outside the repository) and record path, version, build, and binary SHA-256. Keep it through independent production audit PASS. If production is already `/Applications/Syrtis.app`, back up that bundle. If this is a first-name migration and production is still `/Applications/TokenBar.app`, preserve that old leaf in the backup name.
4. **Choose the correct install path.**
   - **Normal Syrtis→Syrtis upgrade:** quit the running app, replace `/Applications/Syrtis.app` with the accepted candidate, verify pre-launch identity/hash, then launch normally.
   - **First TokenBar→Syrtis rename migration:** do **not** install a parallel `/Applications/Syrtis.app` while `/Applications/TokenBar.app` remains. Quit old TokenBar, copy the accepted `dist/Syrtis.app` bundle into `/Applications/TokenBar.app`, verify that pre-launch bundle's version/build/updater flag/hash, then launch `/Applications/TokenBar.app` normally. Upstream `BundleRename` must move/relaunch it as `/Applications/Syrtis.app`. Final state must be exactly `Syrtis.app` present, `TokenBar.app` absent, with the running process path inside the installed Syrtis bundle.
5. **Production acceptance.** Verify bundle id `com.nyanako.tokenbar`, expected version/build, updater flag false, installed binary SHA equal to the candidate, readable codesign state, no Homebrew ownership, and no immediate crash loop. Use the release-specific smoke list plus only the private live probes that are safe and actually available. If `sfltool`, `osascript`, or another macOS login-item probe hangs or requires GUI automation, terminate that read-only probe and record the limitation; do not block a healthy deployment on unreliable GUI tooling. The code contract must still show `UpdaterService.isAvailable = isBundled && officialUpdatesEnabled` and `AutostartService.isAvailable = isBundled`.
6. **Rollback is explicit and manual.** There is **no auto-rollback**. On failed production acceptance, preserve evidence and report PARTIAL/FAIL for operator judgment. Restore only the attributed backup whose version/build/SHA were recorded; never "roll forward" to the newest tag as a fix.
7. **Independent production audit before cleanup.** A separate auditor re-reads source revision, engine gitlink, installed identity/hash/process, updater/autostart contract, rollback backup, Homebrew ownership, and evidence preservation. Missing account credentials (for example a DeepSeek key) are an environmental limitation, not a fabricated failure. Cleanup does not begin until this audit returns PASS.
8. **Cleanup after audit PASS only.** Measure candidate sizes first, then remove only artifacts created for the update: disposable implementation/deploy/audit worktrees or clones, `dist/`/`dist/selftest/`, Swift `.build/`, Rust `target/` directories (including submodule targets), downloaded release assets, and update-specific staging/log files. Run `git worktree prune`. Preserve canonical source/history, user configuration/data, shared caches, and the rollback backup.
9. **Canonical clean-state verification.** After removing worktrees, explicitly run `git submodule sync` + `git submodule update --init --recursive vendor/tokscale-core`, verify the checked-out submodule HEAD equals the `main` gitlink, and require `git status` clean. Cleanup is not complete if the canonical repo is left with `M vendor/tokscale-core`. Record both `du` totals for removed candidates and `df` before/after; on APFS, immediate `df` recovery may be smaller than the summed deleted sizes.

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
- Private submodule CI cannot assume the parent repo's default `GITHUB_TOKEN` can read `HCH725/tokscale-core-custom`. Use a read-only deploy key scoped to that engine repo, keep the private key only in Actions secrets, fail closed when the secret is absent, and never print it.
- When a shallow parent checkout needs history, `git fetch --unshallow` can recurse into the private submodule and request historical public gitlinks that do not exist in the private remote (`not our ref`). Fetch superproject history with `fetch.recurseSubmodules=false`; unshallow the current engine separately with its deploy-key auth.
- The first TokenBar→Syrtis rename is a migration, not an ordinary reinstall: install the accepted Syrtis bundle under the old `/Applications/TokenBar.app` leaf, launch normally, and let upstream `BundleRename` move/relaunch it as `/Applications/Syrtis.app`. Parallel old/new app bundles are not accepted.
- `sfltool dumpbtm`, `osascript`, or similar macOS login-item readbacks can hang behind GUI/Automation state. Stop a hung read-only probe, record the limitation, and verify the updater/autostart code contract instead of turning the release into an unrelated macOS debugging task.
- Removing old worktrees can expose a stale canonical submodule checkout even when `main` is correct. Always finish cleanup with `git submodule sync`, `git submodule update --init --recursive vendor/tokscale-core`, and a clean `git status`.
- A very large consumer diff can be mostly upstream rename/assets churn. Audit the downstream delta against the new upstream base before calling the customization large or over-engineered.
- This skill is not covered by `scripts/check_knowledge.py` (the script validates `docs/knowledge/`, the adapters, and the ledger). Keep its relative links valid by hand when editing it.

## Scope guard

One procedure, one file. Do not add a manager, service, controller, daemon, install helper, or a second SOP; do not restate project facts that belong in `docs/knowledge/`. `CUSTOM.md` and `AGENTS.md` carry only a routing reference to this file.
