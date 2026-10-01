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

The mandatory ordering is: **engine-first when shared behavior changes → consumer PR/CI/independent audit → merge private `main` → deploy the exact accepted merged revision → independent production audit → cleanup and canonical clean-state verification**. A custom tag is optional and is created only when the release workflow explicitly requires one.

Never let Homebrew or the official Sparkle feed silently move production ahead of the private canonical repository. An upgrade is complete only after reviewed/merged private repos, independent production audit PASS, cleanup of disposable upgrade artifacts, and a clean canonical repo/submodule state.
