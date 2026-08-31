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
3. CatDesk input/output tokens count in Hermes totals and TokenBar grand totals. First-version CatDesk cost remains `0`/unknown rather than inventing provider pricing.
4. TokenBar never writes to Hermes `state.db` or CatDesk's ledger.
5. Private production bundles must not auto-update from the official Nanako0129 Sparkle feed. Autostart remains available.
6. Once the private build is installed, the official Homebrew cask must no longer own `/Applications/TokenBar.app`.

## Update workflow

1. Fetch the official upstream release and review release notes and architecture changes.
2. Update `HCH725/tokscale-core-custom` first when the parser boundary changes; keep the custom delta minimal and run its complete test suite.
3. Merge the reviewed tokscale revision and update TokenBar's submodule pointer to that accepted commit.
4. Port only the smallest TokenBar-specific custom changes still required, including the official-update guardrail.
5. Run TokenBar self-tests/smoke tests and verify Hermes totals include CatDesk ledger data without creating a CatDesk client.
6. Perform an independent audit before accepting the private release.
7. Merge/tag the accepted private revision, build the `.app`, then replace local production. Keep the previous working app available for rollback.

Never let Homebrew or the official Sparkle feed silently move production ahead of the private canonical repository.
