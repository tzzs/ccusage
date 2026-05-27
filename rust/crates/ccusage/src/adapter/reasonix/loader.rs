use crate::{cli::SharedArgs, LoadedEntry, PricingMap, Result};

use super::{parser::to_loaded_entry, paths::reasonix_usage_paths};

pub(crate) fn load_entries(shared: &SharedArgs, _pricing: &PricingMap) -> Result<Vec<LoadedEntry>> {
    crate::progress::track_usage_load(
        crate::progress::UsageLoadAgent::Reasonix,
        shared.json,
        || load_entries_inner(shared),
    )
}

fn load_entries_inner(shared: &SharedArgs) -> Result<Vec<LoadedEntry>> {
    let tz = crate::parse_tz(shared.timezone.as_deref());
    let mut entries = Vec::new();

    for path in reasonix_usage_paths()? {
        crate::debug_log(
            shared,
            format!("Reading Reasonix usage file: {}", path.display()),
        );
        let content = std::fs::read_to_string(&path)?;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Ok(entry) = serde_json::from_str::<super::parser::ReasonixEntry>(line) else {
                crate::debug_log(shared, format!("Skipping invalid Reasonix line: {line}"));
                continue;
            };
            if let Some(loaded) = to_loaded_entry(entry, tz.as_ref()) {
                entries.push(loaded);
            }
        }
    }

    crate::debug_log(
        shared,
        format!("Loaded {} Reasonix usage entries", entries.len()),
    );

    entries.sort_by_key(|entry| entry.timestamp);
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccusage_test_support::fs_fixture;

    #[test]
    fn loads_entries_from_usage_file() {
        let fixture = fs_fixture!({
            ".reasonix/usage.jsonl": [
                r#"{"ts":1779912030974,"session":"code-test","model":"deepseek-v4-pro","promptTokens":15899,"completionTokens":333,"cacheHitTokens":0,"cacheMissTokens":15899,"costUsd":0.0072}"#,
                r#"{"ts":1779912054433,"session":"code-test","model":"deepseek-v4-flash","promptTokens":103405,"completionTokens":2987,"cacheHitTokens":88064,"cacheMissTokens":15341,"costUsd":0.00323,"kind":"subagent"}"#,
                "",
                "invalid json",
                r#"{"ts":0,"session":"skip","model":"skip"}"#,
            ]
            .join("\n"),
        });

        let shared = crate::cli::SharedArgs {
            timezone: Some("UTC".to_string()),
            ..Default::default()
        };

        // Override REASONIX_HOME for the test
        let previous = std::env::var("REASONIX_HOME").ok();
        std::env::set_var("REASONIX_HOME", fixture.path(".reasonix").to_string_lossy().as_ref());

        let entries = load_entries_inner(&shared).unwrap();

        if let Some(previous) = previous {
            std::env::set_var("REASONIX_HOME", previous);
        } else {
            std::env::remove_var("REASONIX_HOME");
        }

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].session_id.as_ref(), "code-test");
        assert_eq!(entries[0].model.as_deref(), Some("deepseek-v4-pro"));
        assert_eq!(entries[0].cost, 0.0072);
        assert!(entries[1].model.as_deref() == Some("deepseek-v4-flash"));
        assert_eq!(entries[1].cost, 0.00323);
    }
}
