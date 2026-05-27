use std::sync::Arc;

use jiff::tz::TimeZone as JiffTimeZone;
use serde::Deserialize;

use crate::{format_date_tz, LoadedEntry, TimestampMs, TokenUsageRaw, UsageEntry, UsageMessage};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(super) struct ReasonixEntry {
    pub(super) ts: f64,
    #[serde(default)]
    pub(super) session: Option<String>,
    pub(super) model: Option<String>,
    #[serde(default)]
    pub(super) prompt_tokens: u64,
    #[serde(default)]
    pub(super) completion_tokens: u64,
    #[serde(default)]
    pub(super) cache_hit_tokens: u64,
    #[serde(default)]
    pub(super) cache_miss_tokens: u64,
    pub(super) cost_usd: Option<f64>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(super) claude_equiv_usd: Option<f64>,
    #[serde(default)]
    pub(super) kind: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(super) subagent: Option<serde_json::Value>,
    #[serde(default)]
    pub(super) message_count: Option<u64>,
}

impl ReasonixEntry {
    pub(super) fn session(&self) -> &str {
        self.session.as_deref().unwrap_or("unknown")
    }

    pub(super) fn model(&self) -> Option<&str> {
        self.model.as_deref().filter(|model| !model.is_empty())
    }

    #[allow(dead_code)]
    pub(super) fn is_subagent(&self) -> bool {
        self.kind.as_deref() == Some("subagent")
    }
}

pub(super) fn to_loaded_entry(
    entry: ReasonixEntry,
    tz: Option<&JiffTimeZone>,
) -> Option<LoadedEntry> {
    let timestamp = timestamp_from_millis(entry.ts)?;
    let date = format_date_tz(timestamp, tz);
    let cost_usd = entry.cost_usd.unwrap_or(0.0);
    let session_id = entry.session().to_string();
    let model = entry.model().map(str::to_string);

    let data = UsageEntry {
        session_id: Some(session_id.clone()),
        timestamp: crate::format_rfc3339_millis(timestamp),
        version: None,
        message: UsageMessage {
            usage: TokenUsageRaw {
                input_tokens: entry.cache_miss_tokens,
                output_tokens: entry.completion_tokens,
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: entry.cache_hit_tokens,
                speed: None,
            },
            model: model.clone(),
            id: Some(format!("reasonix:{}:{}", session_id, entry.ts as i64)),
        },
        cost_usd: Some(cost_usd),
        request_id: None,
        is_api_error_message: None,
        is_sidechain: None,
    };

    let project: Arc<str> = Arc::from("reasonix");
    let session_arc: Arc<str> = Arc::from(session_id.as_str());
    let project_path: Arc<str> = Arc::from("Reasonix");

    Some(LoadedEntry {
        data,
        timestamp,
        date,
        project,
        session_id: session_arc,
        project_path,
        cost: cost_usd,
        extra_total_tokens: 0,
        credits: None,
        message_count: entry.message_count,
        model,
        usage_limit_reset_time: None,
    })
}

fn timestamp_from_millis(ms: f64) -> Option<TimestampMs> {
    if !ms.is_finite() || ms <= 0.0 {
        return None;
    }
    Some(TimestampMs::from_millis(ms as i64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_usage_line_with_camelcase_keys() {
        let json = r#"{"ts":1779912030974,"session":"code-test","model":"deepseek-v4-pro","promptTokens":15899,"completionTokens":333,"cacheHitTokens":0,"cacheMissTokens":15899,"costUsd":0.0072,"claudeEquivUsd":0.0527}"#;
        let entry: ReasonixEntry = serde_json::from_str(json).unwrap();

        assert_eq!(entry.ts, 1779912030974.0);
        assert_eq!(entry.session(), "code-test");
        assert_eq!(entry.model(), Some("deepseek-v4-pro"));
        assert_eq!(entry.prompt_tokens, 15899);
        assert_eq!(entry.completion_tokens, 333);
        assert_eq!(entry.cache_hit_tokens, 0);
        assert_eq!(entry.cache_miss_tokens, 15899);
        assert!(!entry.is_subagent());
    }

    #[test]
    fn parses_subagent_entry() {
        let json = r#"{"ts":1779912054433,"session":"code-test","model":"deepseek-v4-flash","promptTokens":103405,"completionTokens":2987,"cacheHitTokens":88064,"cacheMissTokens":15341,"costUsd":0.00323,"kind":"subagent","subagent":{"skillName":"explore"}}"#;
        let entry: ReasonixEntry = serde_json::from_str(json).unwrap();

        assert!(entry.is_subagent());
    }

    #[test]
    fn parses_minimal_entry_with_defaults() {
        let json = r#"{"ts":1779912030974,"promptTokens":100,"completionTokens":50,"cacheHitTokens":20,"cacheMissTokens":80}"#;
        let entry: ReasonixEntry = serde_json::from_str(json).unwrap();

        assert_eq!(entry.prompt_tokens, 100);
        assert_eq!(entry.completion_tokens, 50);
        assert_eq!(entry.cache_hit_tokens, 20);
        assert_eq!(entry.cache_miss_tokens, 80);
        assert_eq!(entry.cost_usd, None);
        assert_eq!(entry.session(), "unknown");
    }

    #[test]
    fn skips_invalid_timestamps() {
        assert!(timestamp_from_millis(0.0).is_none());
        assert!(timestamp_from_millis(-1.0).is_none());
        assert!(timestamp_from_millis(f64::NAN).is_none());
    }

    #[test]
    fn converts_to_loaded_entry_with_correct_token_mapping() {
        let json = r#"{"ts":1779912030974,"session":"code-test","model":"deepseek-v4-pro","promptTokens":15899,"completionTokens":333,"cacheHitTokens":5000,"cacheMissTokens":10899,"costUsd":0.0072}"#;
        let entry: ReasonixEntry = serde_json::from_str(json).unwrap();

        let tz = crate::parse_tz(Some("UTC"));
        let loaded = to_loaded_entry(entry, tz.as_ref()).unwrap();

        assert_eq!(loaded.session_id.as_ref(), "code-test");
        assert_eq!(loaded.model.as_deref(), Some("deepseek-v4-pro"));
        assert_eq!(loaded.data.message.usage.input_tokens, 10899);
        assert_eq!(loaded.data.message.usage.output_tokens, 333);
        assert_eq!(loaded.data.message.usage.cache_read_input_tokens, 5000);
        assert_eq!(loaded.data.message.usage.cache_creation_input_tokens, 0);
        assert_eq!(loaded.cost, 0.0072);
    }
}
