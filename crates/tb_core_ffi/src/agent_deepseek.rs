//! DeepSeek Official balance — the prepaid account balance for the DeepSeek
//! API key Hermes already holds.
//!
//! DeepSeek publishes no token-usage or subscription-window endpoint for this
//! account, but `GET /user/balance` returns the amount left to spend:
//! `{is_available, balance_infos[{currency, total_balance, granted_balance,
//! topped_up_balance}]}`. The amounts are currency amounts, not percentages, so
//! they map to `BalanceSnapshot` rather than to a `UsageWindow` — a window is a
//! percent with a reset, and inventing one here would make the tray report a
//! quota the provider never stated.
//!
//! Credential handling: the request-time read of `DEEPSEEK_API_KEY` from the
//! process environment or `~/.hermes/.env` is the whole of it. The value is
//! never copied, never persisted, never logged, and never written back; a
//! missing key is an error-only card rather than a fabricated zero.

use crate::agent_account_scope::{self, AccountScope, AccountScopeError};
use crate::agent_usage::{
    provider_http_client_builder, read_response_body, request_after_verified_binding,
    AgentIdentity, BalanceSnapshot, ProviderCacheBinding, ProviderFetchFailure,
    ResponseReadFailure, TransportErrorFacts, TransportPhase,
};
use std::path::{Path, PathBuf};

const DEEPSEEK_BALANCE_URL: &str = "https://api.deepseek.com/user/balance";
const DEEPSEEK_KEY_NAME: &str = "DEEPSEEK_API_KEY";

pub(crate) struct DeepSeekData {
    pub identity: Option<AgentIdentity>,
    pub account_scope: Result<AccountScope, AccountScopeError>,
    pub cache_binding: ProviderCacheBinding,
    pub balance: BalanceSnapshot,
}

/// The Hermes credential file this reads a single variable out of. Hermes
/// loads it into its own process environment; TokenBar is a separate process,
/// so the file is the durable location the variable actually lives in.
fn hermes_env_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(Path::new(&home).join(".hermes").join(".env"))
}

/// One variable, no interpolation, no shell parsing beyond the quoting a
/// `.env` writer produces. Comments and blank lines are skipped; `export ` is
/// tolerated because the same file may be sourced by a shell.
fn api_key_from_env_file(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line).trim_start();
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        if name.trim() != DEEPSEEK_KEY_NAME {
            continue;
        }
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix('"'))
            .or_else(|| {
                value
                    .strip_prefix('\'')
                    .and_then(|rest| rest.strip_suffix('\''))
            })
            .unwrap_or(value);
        let value = value.trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

/// Request-time credential resolution. The environment wins because that is
/// the same resolution order Hermes' own provider loader uses.
fn resolve_api_key() -> Option<String> {
    if let Ok(value) = std::env::var(DEEPSEEK_KEY_NAME) {
        let value = value.trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    api_key_from_env_file(&hermes_env_path()?)
}

pub(crate) async fn fetch() -> Result<DeepSeekData, ProviderFetchFailure> {
    let Some(api_key) = resolve_api_key() else {
        return Err(ProviderFetchFailure::terminal(
            "DeepSeek API key is not configured.",
        ));
    };
    // `semantic_source`/`canonical_location` describe where the credential came
    // from without carrying it; the key itself is the transient marker the
    // account scope is derived from. Nothing here reaches the wire or a log.
    let verified = agent_account_scope::resolve_credential(
        "deepseek",
        "env",
        "hermes-env",
        api_key.as_bytes(),
    )
    .map(|account_scope| {
        let cache_binding = ProviderCacheBinding::primary(account_scope.clone());
        (account_scope, cache_binding)
    })
    .map_err(|_| {
        ProviderFetchFailure::terminal("DeepSeek account identity could not be verified.")
    });
    let (account_scope, cache_binding, response) =
        request_after_verified_binding(verified, |(account_scope, cache_binding)| async move {
            let client = provider_http_client_builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|_| {
                    ProviderFetchFailure::terminal("DeepSeek balance client could not be created.")
                })?;
            let response = client
                .get(DEEPSEEK_BALANCE_URL)
                .header(reqwest::header::AUTHORIZATION, format!("Bearer {api_key}"))
                .header(reqwest::header::ACCEPT, "application/json")
                .send()
                .await
                .map_err(|error| {
                    ProviderFetchFailure::from_send_error(
                        "DeepSeek balance request failed. Retrying automatically.",
                        Some(cache_binding.clone()),
                        &error,
                    )
                })?;
            Ok((account_scope, cache_binding, response))
        })
        .await?;
    let status = response.status().as_u16();
    let body = read_response_body(status, false, || async {
        response.text().await.map_err(|error| {
            TransportErrorFacts::from_reqwest(&error, TransportPhase::ResponseBody)
        })
    })
    .await
    .map_err(|failure| match failure {
        ResponseReadFailure::Transient(diagnostic) => ProviderFetchFailure::transient(
            "DeepSeek balance request failed. Retrying automatically.",
            Some(cache_binding.clone()),
            diagnostic,
        ),
        ResponseReadFailure::Terminal(401 | 403) => {
            ProviderFetchFailure::terminal("DeepSeek API key was rejected.")
        }
        ResponseReadFailure::Terminal(status) => ProviderFetchFailure::terminal(format!(
            "DeepSeek balance API rejected the request (status {status})."
        )),
    })?;
    let balance = decode_balance_response(&body)?;
    Ok(DeepSeekData {
        identity: None,
        account_scope: Ok(account_scope),
        cache_binding,
        balance,
    })
}

#[derive(Debug, serde::Deserialize)]
struct BalanceResponse {
    #[serde(default)]
    is_available: bool,
    #[serde(default)]
    balance_infos: Vec<BalanceInfo>,
}

#[derive(Debug, serde::Deserialize)]
struct BalanceInfo {
    currency: String,
    total_balance: String,
    #[serde(default)]
    granted_balance: Option<String>,
    #[serde(default)]
    topped_up_balance: Option<String>,
}

/// The provider's own amounts are decimal strings; a value that is absent or
/// not a finite non-negative number leaves that field out rather than reading
/// as zero. `total` is required — a balance card without a total has nothing
/// to display and is the error-only shape instead.
fn parse_amount(raw: &str) -> Option<f64> {
    let value: f64 = raw.trim().parse().ok()?;
    (value.is_finite() && value >= 0.0).then_some(value)
}

pub(crate) fn decode_balance_response(body: &str) -> Result<BalanceSnapshot, ProviderFetchFailure> {
    let response: BalanceResponse = serde_json::from_str(body).map_err(|_| {
        ProviderFetchFailure::terminal("DeepSeek balance response could not be decoded.")
    })?;
    if !response.is_available {
        return Err(ProviderFetchFailure::terminal(
            "DeepSeek account is not available for balance reporting.",
        ));
    }
    let row = response.balance_infos.first().ok_or_else(|| {
        ProviderFetchFailure::terminal("DeepSeek balance response carried no balance.")
    })?;
    let currency = row.currency.trim().to_uppercase();
    if currency.is_empty() {
        return Err(ProviderFetchFailure::terminal(
            "DeepSeek balance response carried no currency.",
        ));
    }
    let total = parse_amount(&row.total_balance).ok_or_else(|| {
        ProviderFetchFailure::terminal("DeepSeek balance response carried no usable total.")
    })?;
    Ok(BalanceSnapshot {
        currency,
        total,
        granted: row.granted_balance.as_deref().and_then(parse_amount),
        topped_up: row.topped_up_balance.as_deref().and_then(parse_amount),
        is_available: response.is_available,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(body: &str) -> Result<BalanceSnapshot, ProviderFetchFailure> {
        decode_balance_response(body)
    }

    #[test]
    fn official_fixture_decodes_every_amount() {
        let balance = decode(
            r#"{"is_available":true,"balance_infos":[{"currency":"USD","total_balance":"3.41","granted_balance":"1.00","topped_up_balance":"2.41"}]}"#,
        )
        .unwrap();
        assert_eq!(balance.currency, "USD");
        assert!((balance.total - 3.41).abs() < 1e-9);
        assert_eq!(balance.granted, Some(1.0));
        assert_eq!(balance.topped_up, Some(2.41));
        assert!(balance.is_available);
        let wire = serde_json::to_value(&balance).unwrap();
        assert_eq!(wire["currency"], "USD");
        assert_eq!(wire["total"], 3.41);
        assert_eq!(wire["granted"], 1.0);
        assert_eq!(wire["toppedUp"], 2.41);
        assert_eq!(wire["isAvailable"], true);
        // No percentage may be invented from a currency amount.
        assert!(serde_json::to_string(&balance)
            .unwrap()
            .find("Percent")
            .is_none());
    }

    #[test]
    fn missing_optional_amounts_stay_absent_instead_of_zero() {
        let balance = decode(
            r#"{"is_available":true,"balance_infos":[{"currency":"cny","total_balance":"110.00"}]}"#,
        )
        .unwrap();
        assert_eq!(balance.currency, "CNY");
        assert_eq!(balance.granted, None);
        assert_eq!(balance.topped_up, None);
        let wire = serde_json::to_value(&balance).unwrap();
        assert!(wire.get("granted").is_none(), "{wire}");
        assert!(wire.get("toppedUp").is_none(), "{wire}");
    }

    #[test]
    fn unusable_responses_fail_closed() {
        let cases = [
            r#"{"is_available":false,"balance_infos":[{"currency":"USD","total_balance":"3.41"}]}"#,
            r#"{"is_available":true,"balance_infos":[]}"#,
            r#"{"is_available":true}"#,
            r#"{"is_available":true,"balance_infos":[{"currency":"","total_balance":"3.41"}]}"#,
            r#"{"is_available":true,"balance_infos":[{"currency":"USD"}]}"#,
            r#"{"is_available":true,"balance_infos":[{"currency":"USD","total_balance":"NaN"}]}"#,
            r#"{"is_available":true,"balance_infos":[{"currency":"USD","total_balance":"-1.00"}]}"#,
            r#"{"is_available":true,"balance_infos":[{"currency":"USD","total_balance":3.41}]}"#,
            r#"not json"#,
        ];
        for body in cases {
            assert!(
                matches!(decode(body), Err(ProviderFetchFailure::Terminal { .. })),
                "{body}"
            );
        }
    }

    #[test]
    fn env_file_reader_takes_one_variable_and_ignores_noise() {
        let dir = std::env::temp_dir().join(format!("tokenbar-deepseek-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env");
        std::fs::write(
            &path,
            "# comment\nOTHER_KEY=1\nexport DEEPSEEK_API_KEY=\"sk-test-value\"\ntrailing=2\n",
        )
        .unwrap();
        assert_eq!(
            api_key_from_env_file(&path).as_deref(),
            Some("sk-test-value")
        );
        std::fs::write(&path, "DEEPSEEK_API_KEY=\n").unwrap();
        assert_eq!(api_key_from_env_file(&path), None);
        std::fs::write(&path, "OTHER=1\n").unwrap();
        assert_eq!(api_key_from_env_file(&path), None);
        std::fs::remove_dir_all(&dir).ok();
    }
}
