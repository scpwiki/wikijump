use super::model::MAX_REPLAY_CONCURRENCY;
use crate::error::prelude::{Error, ErrorType, ExnError, Result};
use std::env;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_CONCURRENCY: usize = 8;
const DEFAULT_TIMEOUT_MS: u64 = 10_000;
const DEFAULT_DDMIN_PROBES: usize = 128;

#[derive(Debug, Clone)]
pub(crate) struct RenderReplaySettings {
    pub import_run_id: Option<i64>,
    pub states: Vec<String>,
    pub concurrency: usize,
    pub timeout: Duration,
    pub ddmin: bool,
    pub ddmin_max_probes: usize,
    pub artifact_dir: PathBuf,
}

impl RenderReplaySettings {
    pub(crate) fn from_env() -> Result<Self> {
        let import_run_id = optional_positive_i64("DEEPWELL_REPLAY_IMPORT_RUN_ID")?;
        let states = parse_states(env::var("DEEPWELL_REPLAY_STATES").ok())?;
        let concurrency =
            positive_usize("DEEPWELL_REPLAY_CONCURRENCY", DEFAULT_CONCURRENCY)?;
        if concurrency > MAX_REPLAY_CONCURRENCY {
            return Err(config_error(format!(
                "DEEPWELL_REPLAY_CONCURRENCY must be <= {MAX_REPLAY_CONCURRENCY}"
            )));
        }
        let timeout_ms = positive_u64("DEEPWELL_REPLAY_TIMEOUT_MS", DEFAULT_TIMEOUT_MS)?;
        let ddmin = boolish("DEEPWELL_REPLAY_DDMIN", true)?;
        let ddmin_max_probes =
            positive_usize("DEEPWELL_REPLAY_DDMIN_MAX_PROBES", DEFAULT_DDMIN_PROBES)?;
        let artifact_dir = env::var_os("DEEPWELL_REPLAY_ARTIFACT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(format!(
                    "/tmp/deepwell-render-replay-{}-{}",
                    std::process::id(),
                    uuid::Uuid::new_v4(),
                ))
            });

        Ok(Self {
            import_run_id,
            states,
            concurrency,
            timeout: Duration::from_millis(timeout_ms),
            ddmin,
            ddmin_max_probes,
            artifact_dir,
        })
    }
}

pub(super) fn states_sql(states: &[String]) -> String {
    states
        .iter()
        .map(|state| format!("'{state}'"))
        .collect::<Vec<_>>()
        .join(",")
}

fn parse_states(value: Option<String>) -> Result<Vec<String>> {
    const ALLOWED: [&str; 10] = [
        "pending",
        "shell_ready",
        "snapshot_ready",
        "parent_pending",
        "render_pending",
        "render_running",
        "rendered",
        "render_failed",
        "done",
        "failed",
    ];
    let value = value.unwrap_or_else(|| "render_failed".to_owned());
    let mut states = value
        .split(',')
        .map(str::trim)
        .filter(|state| !state.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    states.sort();
    states.dedup();
    if states.is_empty()
        || states
            .iter()
            .any(|state| !ALLOWED.contains(&state.as_str()))
    {
        return Err(config_error(format!(
            "invalid DEEPWELL_REPLAY_STATES: {value}"
        )));
    }
    Ok(states)
}

fn optional_positive_i64(name: &str) -> Result<Option<i64>> {
    env::var(name)
        .ok()
        .map(|value| parse_positive(name, &value))
        .transpose()
}

fn positive_i64(name: &str, default: i64) -> Result<i64> {
    match env::var(name) {
        Ok(value) => parse_positive(name, &value),
        Err(_) => Ok(default),
    }
}

fn positive_u64(name: &str, default: u64) -> Result<u64> {
    let value = positive_i64(name, i64::try_from(default).unwrap_or(i64::MAX))?;
    u64::try_from(value).map_err(|_| config_error(format!("{name} is too large")))
}

fn positive_usize(name: &str, default: usize) -> Result<usize> {
    let value = positive_i64(name, i64::try_from(default).unwrap_or(i64::MAX))?;
    usize::try_from(value).map_err(|_| config_error(format!("{name} is too large")))
}

fn parse_positive(name: &str, value: &str) -> Result<i64> {
    let parsed = value
        .trim()
        .parse::<i64>()
        .map_err(|_| config_error(format!("{name} must be a positive integer")))?;
    if parsed > 0 {
        Ok(parsed)
    } else {
        Err(config_error(format!("{name} must be a positive integer")))
    }
}

fn boolish(name: &str, default: bool) -> Result<bool> {
    match env::var(name) {
        Err(_) => Ok(default),
        Ok(value) if matches!(value.trim(), "1" | "true" | "yes" | "on") => Ok(true),
        Ok(value) if matches!(value.trim(), "0" | "false" | "no" | "off") => Ok(false),
        Ok(_) => Err(config_error(format!("{name} must be a boolean"))),
    }
}

fn config_error(message: String) -> ExnError {
    Error::new(message, ErrorType::ConfigSetup).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_are_validated_before_sql_interpolation() {
        assert_eq!(
            parse_states(Some("render_failed,render_running".to_owned())).unwrap(),
            vec!["render_failed", "render_running"],
        );
        assert!(
            parse_states(Some("render_failed'); DROP TABLE page;--".to_owned())).is_err()
        );
    }

    #[test]
    fn maximum_concurrency_is_sixteen() {
        assert_eq!(MAX_REPLAY_CONCURRENCY, 16);
    }
}
