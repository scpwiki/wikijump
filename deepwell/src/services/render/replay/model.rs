/*
 * services/render/replay/model.rs
 *
 * Serializable protocol and evidence types for corpus render replay.
 */

use crate::services::render::CorpusReplayExpandedWikitext;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(crate) const REPLAY_SCHEMA: &str = "deepwell.render-replay.v1";
pub(crate) const MAX_REPLAY_CONCURRENCY: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct ReplayWorkerRequest {
    pub schema: String,
    pub request_id: String,
    pub expanded: CorpusReplayExpandedWikitext,
    pub emit_prepared_path: Option<PathBuf>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReplayStage {
    Load,
    Expand,
    Protect,
    Preprocess,
    Tokenize,
    Parse,
    Render,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub(crate) enum WorkerEvent {
    StageStarted {
        stage: ReplayStage,
    },
    StageFinished {
        stage: ReplayStage,
        elapsed_us: u64,
        input_bytes: usize,
        output_bytes: usize,
    },
    Prepared {
        sha256: String,
        features: SyntaxFeatures,
    },
    Completed {
        result: WorkerResult,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct WorkerResult {
    pub outcome: ReplayOutcome,
    pub signature: FailureSignature,
    pub features: SyntaxFeatures,
    pub parse_errors: Vec<ErrorSite>,
    pub parse_error_count: usize,
    pub prepared_sha256: String,
    pub ftml_core_rendered_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ReplayOutcome {
    Pass,
    CompatibilityFallback,
    ParserErrors,
    Timeout {
        stage: ReplayStage,
        timeout_ms: u64,
        killed: bool,
        reaped: bool,
    },
    Crash {
        exit_code: Option<i32>,
        signal: Option<i32>,
        stderr_fingerprint: String,
    },
    ProtocolError {
        message: String,
    },
    PreparationError {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) struct FailureSignature {
    pub class: String,
    pub stage: ReplayStage,
    pub key: String,
}

impl FailureSignature {
    pub(crate) fn pass() -> Self {
        Self {
            class: "pass".to_owned(),
            stage: ReplayStage::Complete,
            key: "pass".to_owned(),
        }
    }

    pub(crate) fn protocol(stage: ReplayStage, message: &str) -> Self {
        Self {
            class: "protocol_error".to_owned(),
            stage,
            key: sha256_hex(normalize_diagnostic(message).as_bytes()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct ErrorSite {
    pub rule: String,
    pub kind: String,
    pub token: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
    pub line_shape: String,
    pub context_hash: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct SyntaxFeatures {
    pub bytes: usize,
    pub lines: usize,
    pub max_line_bytes: usize,
    pub max_quote_depth: usize,
    pub marker_counts: BTreeMap<String, usize>,
    pub unbalanced_mask: Vec<String>,
    pub dominant_line_shape: Option<String>,
    pub dominant_line_repetitions: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct ReplayCaseObservation {
    pub case_id: String,
    pub source_fullname: String,
    pub page_id: i64,
    pub site_id: i64,
    pub state: String,
    pub expanded_sha256: String,
    pub prepared_sha256: String,
    pub included_page_count: usize,
    pub preparation_stages: Vec<StageMeasurement>,
    pub result: WorkerResult,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct StageMeasurement {
    pub stage: ReplayStage,
    pub elapsed_us: u64,
    pub input_bytes: usize,
    pub output_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct ReplayCluster {
    pub cluster_id: String,
    pub failure_fingerprint: String,
    pub signature: FailureSignature,
    pub case_ids: Vec<String>,
    pub source_fullnames: Vec<String>,
    pub representative_case_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct RenderReplaySummary {
    pub schema: &'static str,
    pub import_run_id: Option<i64>,
    pub concurrency: usize,
    pub timeout_ms: u64,
    pub candidates: usize,
    pub passed: usize,
    pub compatibility_fallback: usize,
    pub parser_errors: usize,
    pub timed_out: usize,
    pub crashed: usize,
    pub preparation_errors: usize,
    pub protocol_errors: usize,
    pub elapsed_ms: u128,
    pub artifact_dir: String,
    pub clusters: Vec<ReplayCluster>,
    pub minimizations: Vec<ReplayMinimization>,
    pub unverified_minimizations: usize,
    pub gate_passed: bool,
    pub gate_failures: Vec<&'static str>,
    pub observations: Vec<ReplayCaseObservation>,
}

impl RenderReplaySummary {
    pub(crate) fn gate_passed(&self) -> bool {
        self.gate_passed
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct ReplayMinimization {
    pub cluster_id: String,
    pub representative_case_id: String,
    pub probe_concurrency: usize,
    pub original_lines: usize,
    pub minimized_lines: usize,
    pub probes: usize,
    pub cache_hits: usize,
    pub budget_exhausted: bool,
    pub verified: bool,
    pub verification_failure_fingerprint: String,
    pub verification_signature: FailureSignature,
    pub verification_outcome: ReplayOutcome,
    pub expanded_artifact: String,
    pub prepared_artifact: String,
}

pub(crate) fn sha256_hex(input: &[u8]) -> String {
    hex::encode(Sha256::digest(input))
}

pub(crate) fn normalize_diagnostic(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len().min(4096));
    let mut in_digits = false;
    for character in input.chars().take(4096) {
        if character.is_ascii_digit() {
            if !in_digits {
                normalized.push('#');
                in_digits = true;
            }
        } else {
            in_digits = false;
            normalized.push(character);
        }
    }
    normalized
}
