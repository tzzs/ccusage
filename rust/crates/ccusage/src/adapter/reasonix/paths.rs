use std::{collections::HashSet, env, path::PathBuf};

use crate::Result;

const REASONIX_HOME_ENV: &str = "REASONIX_HOME";
const USAGE_FILE: &str = "usage.jsonl";

pub(super) fn reasonix_usage_paths() -> Result<Vec<PathBuf>> {
    let homes = if let Ok(paths) = env::var(REASONIX_HOME_ENV) {
        paths
            .split(',')
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .collect::<Vec<_>>()
    } else {
        let home =
            crate::home::home_dir().ok_or_else(|| crate::cli_error("home directory is not set"))?;
        vec![home.join(".reasonix")]
    };
    let mut seen = HashSet::new();
    Ok(homes
        .into_iter()
        .map(|home| home.join(USAGE_FILE))
        .filter(|path| path.is_file())
        .filter(|path| seen.insert(path.clone()))
        .collect())
}
