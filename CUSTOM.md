# Syrtis Custom

This private repository is the canonical source for the Syrtis build installed in this environment. The repository name remains `HCH725/TokenBar-custom` for continuity; renaming the repository is intentionally deferred.

## Repository relationship

- `origin`: `HCH725/TokenBar-custom` — private canonical downstream.
- `upstream`: `Nanako0129/syrtis` — official source.
- `vendor/tokscale-core`: pinned to the private canonical `HCH725/tokscale-core-custom` mirror.

Do not deploy a local patch that is absent from this repository. Upstream releases are reviewed and merged here first; production is built only from an accepted private revision/tag.

## Current downstream contract

1. CatDesk's append-only `~/.catdesk/usage.jsonl` is ingested by the custom `tokscale-core` dependency as an additional **Hermes** usage source.
2. CatDesk does not become a separate Syrtis client. Its records are attributed as `client=hermes`, `model=catdesk-mcp`, `provider=catdesk`.
3. CatDesk tokens count in Hermes totals and Syrtis grand totals. The displayed source remains `catdesk-mcp`/`catdesk`, while cost is estimated through tokscale's normal pricing service using the ledger `pricingModel` identity; legacy rows without that field use the explicitly confirmed historical `gpt-5.6-sol` fallback. MCP direction is translated before pricing (`outputTokens` → model input, `inputTokens` → model output).
4. Hermes usage attributed to `provider=opencode-go` / `opencode_go` uses OpenCode Go's official usage-value pricing for the covered models before the generic catalog. Provider-reported cost remains authoritative; unknown Go models fall back to the existing pricing service. DeepSeek V4 uses the official weekday UTC peak windows and weekends are off-peak.
5. Hermes `openai-codex` rows explicitly marked `cost_status=included` with `billing_mode=subscription_included` or `codex_responses` keep authoritative raw incremental cost `$0`. Attribution/quota views use a separate recorded-model ChatGPT Work/Codex rate-card equivalent; they never infer 5-hour or weekly quota depletion from those dollars.
6. Syrtis never writes to Hermes `state.db` or CatDesk's ledger.
7. Private production bundles must not auto-update from the official Nanako0129 Sparkle feed. Autostart remains available.
8. Once the private build is installed, the official Homebrew cask must no longer own `/Applications/Syrtis.app`.

## Update workflow

The operational procedure is [`skills/tokenbar-release-update/SKILL.md`](skills/tokenbar-release-update/SKILL.md); this file remains the contract (what must remain true) and wins on any conflict about it, while authorization gates stay owned by [`docs/knowledge/workflow.md`](docs/knowledge/workflow.md).

1. Fetch the official upstream release and review release notes and architecture changes.
2. Update `HCH725/tokscale-core-custom` first when shared parser/scanning/cache/pricing/aggregation behavior changes; keep the custom delta minimal, run its tests, open a PR, complete independent audit/remediation, and merge the engine before advancing the consumer gitlink.
3. Port only the smallest Syrtis-specific custom changes still required, including the official-update guardrail, then run the full consumer build/selftest/smoke/bundled/docs gates.
4. Push a consumer branch and open a PR. GitHub CI and an independent consumer audit must pass before merge. Private engine checkout in Actions uses a read-only deploy key scoped to `HCH725/tokscale-core-custom`; parent-history fetches do not recurse through historical submodule pins.
5. Merge the accepted consumer PR into private `main`. Production is built from that exact merged revision; a custom tag is optional and is created only when explicitly required by the release workflow.
6. Build in a clean detached worktree, verify candidate identity/hash, create an attributed rollback backup, then install. A normal Syrtis→Syrtis update replaces `/Applications/Syrtis.app`; a first TokenBar→Syrtis migration must temporarily install the accepted bundle under `/Applications/TokenBar.app` and let upstream `BundleRename` move/relaunch it as `/Applications/Syrtis.app`.
7. Verify production identity/hash/process, updater disabled, autostart contract intact, no Homebrew ownership, and run only safe available private-data probes. Missing credentials or hanging macOS GUI/login-item probes are recorded as limitations, not invented PASS/FAIL results.
8. A separate production auditor must PASS before cleanup. Only then remove update-created worktrees/build/staging artifacts, preserve the rollback backup and user data, run `git worktree prune`, realign `vendor/tokscale-core` to the `main` gitlink, and require a clean canonical repo.
9. Commit, push, PR, merge, tag, release, install, and production replacement each require the applicable explicit user authorization; implementation, local build, and verification do not.

Never let Homebrew or the official Sparkle feed silently move production ahead of the private canonical repository. An upgrade is complete only after reviewed/merged private repos, independent production audit PASS, cleanup of disposable upgrade artifacts, and a clean canonical repo/submodule state.
