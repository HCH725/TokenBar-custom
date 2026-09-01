# TokenBar Custom

This private repository is the canonical source for the TokenBar build installed in this environment.

## Repository relationship

- `origin`: `HCH725/TokenBar-custom` — private canonical downstream.
- `upstream`: `Nanako0129/TokenBar` — official source.
- `vendor/tokscale-core`: pinned to the private canonical `HCH725/tokscale-core-custom` mirror.

Do not deploy a local patch that is absent from this repository. Upstream releases are reviewed and merged here first; production is built only from an accepted private revision/tag.

## Current downstream contract

1. CatDesk's append-only `~/.catdesk/usage.jsonl` is ingested by the custom `tokscale-core` dependency as an additional **Hermes** usage source.
2. CatDesk does not become a separate TokenBar client. Its records are attributed as `client=hermes`, `model=catdesk-mcp`, `provider=catdesk`.
3. CatDesk tokens count in Hermes totals and TokenBar grand totals. The displayed source remains `catdesk-mcp`/`catdesk`, while cost is estimated through tokscale's normal pricing service using the ledger `pricingModel` identity; legacy rows without that field use the explicitly confirmed historical `gpt-5.6-sol` fallback. MCP direction is translated before pricing (`outputTokens` → model input, `inputTokens` → model output).
4. Hermes usage attributed to `provider=opencode-go` / `opencode_go` uses OpenCode Go's official usage-value pricing for the covered models before the generic catalog. Provider-reported cost remains authoritative; unknown Go models fall back to the existing pricing service. DeepSeek V4 uses the official weekday UTC peak windows and weekends are off-peak.
5. Hermes `openai-codex` rows explicitly marked `cost_status=included` with `billing_mode=subscription_included` or `codex_responses` keep authoritative raw incremental cost `$0`. Attribution/quota views use a separate recorded-model ChatGPT Work/Codex rate-card equivalent; they never infer 5-hour or weekly quota depletion from those dollars.
6. TokenBar never writes to Hermes `state.db` or CatDesk's ledger.
7. Private production bundles must not auto-update from the official Nanako0129 Sparkle feed. Autostart remains available.
8. Once the private build is installed, the official Homebrew cask must no longer own `/Applications/TokenBar.app`.

## Update workflow

1. Fetch the official upstream release and review release notes and architecture changes.
2. Update `HCH725/tokscale-core-custom` first when the parser boundary changes; keep the custom delta minimal and run its complete test suite.
3. Merge the reviewed tokscale revision and update TokenBar's submodule pointer to that accepted commit.
4. Port only the smallest TokenBar-specific custom changes still required, including the official-update guardrail.
5. Run TokenBar self-tests/smoke tests and verify Hermes totals include CatDesk ledger data without creating a CatDesk client, and that OpenCode Go rows use the provider-scoped official usage-value pricing.
6. Perform an independent audit before accepting the private release.
7. Merge/tag the accepted private revision, build the `.app`, then replace local production. Keep the previous working app available for rollback.

Never let Homebrew or the official Sparkle feed silently move production ahead of the private canonical repository.
