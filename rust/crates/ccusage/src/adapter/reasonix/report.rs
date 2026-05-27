use serde_json::Value;

use crate::{
    cli::{AgentReportKind, WeekDay},
    summarize_by_key, summarize_summaries_by_bucket, totals_json, BucketKind, LoadedEntry, Result,
    UsageSummary,
};

pub(crate) fn report_from_rows(rows: &[UsageSummary], kind: AgentReportKind) -> Value {
    let rows_json = rows
        .iter()
        .map(|row| crate::adapter::opencode::agent_summary_json(row, kind, false))
        .collect::<Vec<_>>();
    serde_json::json!({
        rows_key(kind): rows_json,
        "totals": totals_json(rows),
    })
}

pub(crate) fn summary_period(row: &UsageSummary) -> &str {
    row.date
        .as_deref()
        .or(row.week.as_deref())
        .or(row.month.as_deref())
        .or(row.session_id.as_deref())
        .unwrap_or("")
}

pub(crate) fn summarize_entries(
    entries: &[LoadedEntry],
    kind: AgentReportKind,
) -> Result<Vec<UsageSummary>> {
    match kind {
        AgentReportKind::Daily => summarize_by_key(
            entries,
            |entry| entry.date.clone(),
            |date| (date.to_string(), None),
        ),
        AgentReportKind::Monthly => {
            let daily = summarize_entries(entries, AgentReportKind::Daily)?;
            Ok(summarize_summaries_by_bucket(
                &daily,
                BucketKind::Monthly,
                WeekDay::Sunday,
            ))
        }
        AgentReportKind::Session => summarize_by_key(
            entries,
            |entry| entry.session_id.to_string(),
            |session_id| (session_id.to_string(), None),
        )
        .map(|mut rows| {
            for row in &mut rows {
                row.session_id = row.date.take();
            }
            rows
        }),
        AgentReportKind::Weekly => {
            let daily = summarize_entries(entries, AgentReportKind::Daily)?;
            Ok(summarize_summaries_by_bucket(
                &daily,
                BucketKind::Weekly,
                WeekDay::Sunday,
            ))
        }
    }
}

fn rows_key(kind: AgentReportKind) -> &'static str {
    match kind {
        AgentReportKind::Daily => "daily",
        AgentReportKind::Weekly => "weekly",
        AgentReportKind::Monthly => "monthly",
        AgentReportKind::Session => "sessions",
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{TokenUsageRaw, UsageEntry, UsageMessage};

    #[test]
    fn daily_report_includes_all_token_fields() {
        let entry = LoadedEntry {
            data: UsageEntry {
                session_id: Some("code-test".to_string()),
                timestamp: "2026-05-28T10:00:00.000Z".to_string(),
                version: None,
                message: UsageMessage {
                    usage: TokenUsageRaw {
                        input_tokens: 10899,
                        output_tokens: 333,
                        cache_creation_input_tokens: 0,
                        cache_read_input_tokens: 5000,
                        speed: None,
                    },
                    model: Some("deepseek-v4-pro".to_string()),
                    id: Some("reasonix:code-test:1779912030974".to_string()),
                },
                cost_usd: Some(0.0072),
                request_id: None,
                is_api_error_message: None,
                is_sidechain: None,
            },
            timestamp: crate::parse_ts_timestamp("2026-05-28T10:00:00.000Z").unwrap(),
            date: "2026-05-28".to_string(),
            project: Arc::from("reasonix"),
            session_id: Arc::from("code-test"),
            project_path: Arc::from("Reasonix"),
            cost: 0.0072,
            credits: None,
            extra_total_tokens: 0,
            message_count: None,
            model: Some("deepseek-v4-pro".to_string()),
            usage_limit_reset_time: None,
        };

        let rows = summarize_entries(&[entry], AgentReportKind::Daily).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].input_tokens, 10899);
        assert_eq!(rows[0].output_tokens, 333);
        assert_eq!(rows[0].cache_read_tokens, 5000);
        assert_eq!(rows[0].cache_creation_tokens, 0);
        assert_eq!(rows[0].total_tokens(), 16232);

        let report = report_from_rows(&rows, AgentReportKind::Daily);
        assert_eq!(report["daily"][0]["date"], "2026-05-28");
        assert_eq!(report["daily"][0]["inputTokens"], 10899);
        assert_eq!(report["daily"][0]["totalTokens"], 16232);
        assert_eq!(report["daily"][0]["totalCost"], serde_json::json!(0.0072));
    }

    #[test]
    fn session_report_groups_by_session_id() {
        let session_a = loaded_entry("session-a", "model-a", "2026-05-28", 0.01);
        let session_b = loaded_entry("session-b", "model-b", "2026-05-28", 0.02);

        let rows = summarize_entries(&[session_a, session_b], AgentReportKind::Session).unwrap();
        assert_eq!(rows.len(), 2);
    }

    fn loaded_entry(session_id: &str, model: &str, date: &str, cost: f64) -> LoadedEntry {
        LoadedEntry {
            data: UsageEntry {
                session_id: Some(session_id.to_string()),
                timestamp: format!("{date}T10:00:00.000Z"),
                version: None,
                message: UsageMessage {
                    usage: TokenUsageRaw {
                        input_tokens: 100,
                        output_tokens: 50,
                        cache_creation_input_tokens: 0,
                        cache_read_input_tokens: 10,
                        speed: None,
                    },
                    model: Some(model.to_string()),
                    id: Some(format!("reasonix:{session_id}:1")),
                },
                cost_usd: Some(cost),
                request_id: None,
                is_api_error_message: None,
                is_sidechain: None,
            },
            timestamp: crate::parse_ts_timestamp(&format!("{date}T10:00:00.000Z")).unwrap(),
            date: date.to_string(),
            project: Arc::from("reasonix"),
            session_id: Arc::from(session_id),
            project_path: Arc::from("Reasonix"),
            cost,
            credits: None,
            extra_total_tokens: 0,
            message_count: None,
            model: Some(model.to_string()),
            usage_limit_reset_time: None,
        }
    }
}
