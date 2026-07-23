/*
 * services/render/replay/mod.rs
 *
 * Read-only corpus render convergence controller.
 */

mod cluster;
mod database;
mod ddmin;
mod features;
mod model;
mod settings;
mod supervisor;
mod worker;

use self::cluster::{build_clusters, failure_cluster_fingerprint};
use self::database::{
    ReplayCandidate, expand_candidate, list_candidates, select_import_run,
};
use self::ddmin::ddmin_lines;
use self::model::{
    FailureSignature, REPLAY_SCHEMA, RenderReplaySummary, ReplayCaseObservation,
    ReplayMinimization, ReplayOutcome, ReplayStage, ReplayWorkerRequest,
    StageMeasurement, WorkerEvent, WorkerResult, sha256_hex,
};
pub(crate) use self::settings::RenderReplaySettings;
use self::supervisor::run_isolated_worker;
use crate::error::prelude::*;
use crate::runtime::ServerState;
use crate::services::render::CorpusReplayExpandedWikitext;
use futures::{StreamExt, stream};
use serde::{Serialize, de::DeserializeOwned};
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) struct RenderReplayService;

impl RenderReplayService {
    pub(crate) async fn run(
        state: &ServerState,
        mut settings: RenderReplaySettings,
    ) -> Result<RenderReplaySummary> {
        let started = Instant::now();
        settings.artifact_dir = prepare_root_artifact_dir(&settings.artifact_dir)?;
        let import_run_id = select_import_run(state, &settings).await?;
        let candidates = match import_run_id {
            Some(import_run_id) => {
                list_candidates(state, import_run_id, &settings).await?
            }
            None => Vec::new(),
        };
        let candidate_count = candidates.len();
        let timeout = settings.timeout;
        let artifact_dir_for_tasks = settings.artifact_dir.clone();
        let state_for_tasks = state.clone();
        let mut cases = stream::iter(candidates)
            .map(move |candidate| {
                let state = state_for_tasks.clone();
                let artifact_dir = artifact_dir_for_tasks.clone();
                async move { run_case(&state, candidate, timeout, &artifact_dir).await }
            })
            .buffer_unordered(settings.concurrency);

        let observations_path = settings.artifact_dir.join("observations.jsonl");
        let mut observations_file =
            BufWriter::new(open_new_artifact_file(&observations_path)?);
        let mut observations = Vec::with_capacity(candidate_count);
        while let Some(observation) = cases.next().await {
            let observation = observation?;
            serde_json::to_writer(&mut observations_file, &observation)
                .or_raise(replay_error)?;
            observations_file.write_all(b"\n").or_raise(replay_error)?;
            observations.push(observation);
        }
        observations_file.flush().or_raise(replay_error)?;
        drop(observations_file);
        observations.sort_by(|left, right| left.case_id.cmp(&right.case_id));
        let clusters = build_clusters(&observations);
        let minimizations = if settings.ddmin {
            minimize_clusters(&settings, &clusters).await?
        } else {
            Vec::new()
        };
        let unverified_minimizations = minimizations
            .iter()
            .filter(|minimization| !minimization.verified)
            .count();

        let passed = observations
            .iter()
            .filter(|item| matches!(item.result.outcome, ReplayOutcome::Pass))
            .count();
        let parser_errors = observations
            .iter()
            .filter(|item| matches!(item.result.outcome, ReplayOutcome::ParserErrors))
            .count();
        let compatibility_fallback = observations
            .iter()
            .filter(|item| {
                matches!(item.result.outcome, ReplayOutcome::CompatibilityFallback)
            })
            .count();
        let timed_out = observations
            .iter()
            .filter(|item| matches!(item.result.outcome, ReplayOutcome::Timeout { .. }))
            .count();
        let crashed = observations
            .iter()
            .filter(|item| matches!(item.result.outcome, ReplayOutcome::Crash { .. }))
            .count();
        let preparation_errors = observations
            .iter()
            .filter(|item| {
                matches!(item.result.outcome, ReplayOutcome::PreparationError { .. })
            })
            .count();
        let protocol_errors = observations
            .iter()
            .filter(|item| {
                matches!(item.result.outcome, ReplayOutcome::ProtocolError { .. })
            })
            .count();
        debug_assert_eq!(
            passed
                + compatibility_fallback
                + parser_errors
                + timed_out
                + crashed
                + preparation_errors
                + protocol_errors,
            candidate_count,
        );
        let gate_failures = gate_failures(
            import_run_id,
            candidate_count,
            passed,
            compatibility_fallback,
            parser_errors,
            timed_out,
            crashed,
            preparation_errors,
            protocol_errors,
            unverified_minimizations,
        );
        let gate_passed = gate_failures.is_empty();
        let summary = RenderReplaySummary {
            schema: REPLAY_SCHEMA,
            import_run_id,
            concurrency: settings.concurrency,
            timeout_ms: duration_millis(settings.timeout),
            candidates: candidate_count,
            passed,
            compatibility_fallback,
            parser_errors,
            timed_out,
            crashed,
            preparation_errors,
            protocol_errors,
            elapsed_ms: started.elapsed().as_millis(),
            artifact_dir: settings.artifact_dir.display().to_string(),
            clusters,
            minimizations,
            unverified_minimizations,
            gate_passed,
            gate_failures,
            observations,
        };
        write_json(&settings.artifact_dir.join("summary.json"), &summary)?;
        Ok(summary)
    }
}

#[allow(clippy::too_many_arguments)]
fn gate_failures(
    import_run_id: Option<i64>,
    candidates: usize,
    passed: usize,
    compatibility_fallback: usize,
    parser_errors: usize,
    timed_out: usize,
    crashed: usize,
    preparation_errors: usize,
    protocol_errors: usize,
    unverified_minimizations: usize,
) -> Vec<&'static str> {
    let mut failures = Vec::new();
    if import_run_id.is_none() {
        failures.push("missing_import_run");
    }
    if candidates == 0 {
        failures.push("empty_selection");
    }
    if passed != candidates {
        failures.push("not_all_candidates_passed");
    }
    for (count, reason) in [
        (compatibility_fallback, "compatibility_fallback"),
        (parser_errors, "parser_errors"),
        (timed_out, "timed_out"),
        (crashed, "crashed"),
        (preparation_errors, "preparation_errors"),
        (protocol_errors, "protocol_errors"),
        (unverified_minimizations, "unverified_minimizations"),
    ] {
        if count != 0 {
            failures.push(reason);
        }
    }
    failures
}

pub(crate) use self::worker::run_worker_action;

async fn run_case(
    state: &ServerState,
    candidate: ReplayCandidate,
    deadline: Duration,
    artifact_dir: &Path,
) -> Result<ReplayCaseObservation> {
    let started = Instant::now();
    let case_id = format!("page-{}", candidate.page_id);
    let expand_started = Instant::now();
    let expanded = match expand_candidate(state, &candidate).await {
        Ok(expanded) => expanded,
        Err(error) => {
            let message = format!("{error:?}");
            let signature = FailureSignature::protocol(ReplayStage::Expand, &message);
            return Ok(ReplayCaseObservation {
                case_id,
                source_fullname: candidate.source_fullname,
                page_id: candidate.page_id,
                site_id: candidate.site_id,
                state: candidate.state,
                expanded_sha256: String::new(),
                prepared_sha256: String::new(),
                included_page_count: 0,
                preparation_stages: vec![StageMeasurement {
                    stage: ReplayStage::Expand,
                    elapsed_us: elapsed_micros(expand_started),
                    input_bytes: 0,
                    output_bytes: 0,
                }],
                result: WorkerResult {
                    outcome: ReplayOutcome::PreparationError { message },
                    signature,
                    features: Default::default(),
                    parse_errors: Vec::new(),
                    parse_error_count: 0,
                    prepared_sha256: String::new(),
                    ftml_core_rendered_sha256: None,
                },
                duration_ms: duration_millis(started.elapsed()),
            });
        }
    };
    let expand_elapsed_us = elapsed_micros(expand_started);
    let expanded_sha256 = sha256_hex(expanded.wikitext.as_bytes());
    let expanded_bytes = expanded.wikitext.len();
    let included_page_count = expanded.included_page_count();
    write_bytes(
        &artifact_dir.join(format!("{case_id}.expanded.wikidot")),
        expanded.wikitext.as_bytes(),
    )?;
    write_canonical_json(
        &artifact_dir.join(format!("{case_id}.capsule.json")),
        &expanded,
    )?;
    let request = ReplayWorkerRequest {
        schema: REPLAY_SCHEMA.to_owned(),
        request_id: case_id.clone(),
        expanded,
        emit_prepared_path: Some(
            artifact_dir.join(format!("{case_id}.preprocessed.wikidot")),
        ),
    };
    let run = run_isolated_worker(&request, deadline).await;
    let mut preparation_stages = vec![StageMeasurement {
        stage: ReplayStage::Expand,
        elapsed_us: expand_elapsed_us,
        input_bytes: 0,
        output_bytes: expanded_bytes,
    }];
    preparation_stages.extend(stage_measurements(&run.events));
    let prepared_sha256 = run.result.prepared_sha256.clone();
    Ok(ReplayCaseObservation {
        case_id,
        source_fullname: candidate.source_fullname,
        page_id: candidate.page_id,
        site_id: candidate.site_id,
        state: candidate.state,
        expanded_sha256,
        prepared_sha256,
        included_page_count,
        preparation_stages,
        result: run.result,
        duration_ms: duration_millis(started.elapsed()),
    })
}

async fn minimize_clusters(
    settings: &RenderReplaySettings,
    clusters: &[model::ReplayCluster],
) -> Result<Vec<ReplayMinimization>> {
    let mut outputs = Vec::new();
    for cluster in clusters {
        if let Some(output) = minimize_cluster(settings, cluster).await? {
            outputs.push(output);
        }
    }
    outputs.sort_by(|left, right| left.cluster_id.cmp(&right.cluster_id));
    Ok(outputs)
}

async fn minimize_cluster(
    settings: &RenderReplaySettings,
    cluster: &model::ReplayCluster,
) -> Result<Option<ReplayMinimization>> {
    let capsule_path = settings
        .artifact_dir
        .join(format!("{}.capsule.json", cluster.representative_case_id,));
    if !capsule_path.exists() {
        return Ok(None);
    }
    let base: CorpusReplayExpandedWikitext = read_json(&capsule_path)?;
    let target_fingerprint = cluster.failure_fingerprint.clone();
    let base_for_probes = base.clone();
    let timeout = settings.timeout;
    let probe_concurrency = minimization_probe_concurrency(cluster, settings.concurrency);
    let result = ddmin_lines(
        &base.wikitext,
        settings.ddmin_max_probes,
        probe_concurrency,
        move |candidate| {
            let mut expanded = base_for_probes.clone();
            expanded.wikitext = candidate;
            let target_fingerprint = target_fingerprint.clone();
            async move {
                let request = ReplayWorkerRequest {
                    schema: REPLAY_SCHEMA.to_owned(),
                    request_id: "ddmin-probe".to_owned(),
                    expanded,
                    emit_prepared_path: None,
                };
                let result = run_isolated_worker(&request, timeout).await.result;
                failure_cluster_fingerprint(&result) == target_fingerprint
            }
        },
    )
    .await;

    let cluster_dir = settings.artifact_dir.join(&cluster.cluster_id);
    create_artifact_dir(&cluster_dir)?;
    let expanded_path = cluster_dir.join("min.expanded.wikidot");
    let prepared_path = cluster_dir.join("min.preprocessed.wikidot");
    debug_assert!(!prepared_path.exists());
    write_bytes(&expanded_path, result.minimized.as_bytes())?;
    let mut minimized = base.clone();
    minimized.wikitext = result.minimized.clone();
    let request = ReplayWorkerRequest {
        schema: REPLAY_SCHEMA.to_owned(),
        request_id: "ddmin-final".to_owned(),
        expanded: minimized,
        emit_prepared_path: Some(prepared_path.clone()),
    };
    let final_run = run_isolated_worker(&request, timeout).await;
    let verification_failure_fingerprint = failure_cluster_fingerprint(&final_run.result);
    let prepared_present = prepared_path.exists();
    let verified = verification_failure_fingerprint == cluster.failure_fingerprint
        && (cluster.signature.stage < ReplayStage::Tokenize || prepared_present);

    Ok(Some(ReplayMinimization {
        cluster_id: cluster.cluster_id.clone(),
        representative_case_id: cluster.representative_case_id.clone(),
        probe_concurrency,
        original_lines: result.original_lines,
        minimized_lines: result.minimized_lines,
        probes: result.probes,
        cache_hits: result.cache_hits,
        budget_exhausted: result.budget_exhausted,
        verified,
        verification_failure_fingerprint,
        verification_signature: final_run.result.signature,
        verification_outcome: final_run.result.outcome,
        expanded_artifact: expanded_path.display().to_string(),
        prepared_artifact: if prepared_present {
            prepared_path.display().to_string()
        } else {
            String::new()
        },
    }))
}

fn minimization_probe_concurrency(
    cluster: &model::ReplayCluster,
    requested: usize,
) -> usize {
    if cluster.signature.class == "timeout" {
        // Wall-clock timeout probes contend for CPU when run together. That
        // can turn a fast candidate into a false reproducer which immediately
        // passes or changes class during the required single final check.
        1
    } else {
        requested.max(1)
    }
}

fn stage_measurements(events: &[WorkerEvent]) -> Vec<StageMeasurement> {
    events
        .iter()
        .filter_map(|event| match event {
            WorkerEvent::StageFinished {
                stage,
                elapsed_us,
                input_bytes,
                output_bytes,
            } => Some(StageMeasurement {
                stage: *stage,
                elapsed_us: *elapsed_us,
                input_bytes: *input_bytes,
                output_bytes: *output_bytes,
            }),
            _ => None,
        })
        .collect()
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let file = open_new_artifact_file(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value).or_raise(replay_error)?;
    writer.flush().or_raise(replay_error)
}

fn write_canonical_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut value = serde_json::to_value(value).or_raise(replay_error)?;
    sort_json_object_keys(&mut value);
    write_json(path, &value)
}

fn sort_json_object_keys(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                sort_json_object_keys(value);
            }
        }
        serde_json::Value::Object(object) => {
            let mut entries = std::mem::take(object).into_iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            for (key, mut value) in entries {
                sort_json_object_keys(&mut value);
                object.insert(key, value);
            }
        }
        _ => {}
    }
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let file = File::open(path).or_raise(replay_error)?;
    serde_json::from_reader(file).or_raise(replay_error)
}

fn write_bytes(path: &Path, data: &[u8]) -> Result<()> {
    let mut file = open_new_artifact_file(path)?;
    file.write_all(data).or_raise(replay_error)
}

fn open_new_artifact_file(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).or_raise(replay_error)
}

fn create_artifact_dir(path: &Path) -> Result<()> {
    let mut builder = DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path).or_raise(replay_error)
}

fn prepare_root_artifact_dir(path: &Path) -> Result<PathBuf> {
    let Some(name) = path.file_name() else {
        bail!(Error::new(
            format!(
                "render-replay artifact directory must name a new leaf: {}",
                path.display()
            ),
            ErrorType::Render,
        ));
    };
    if name == "." || name == ".." {
        bail!(Error::new(
            format!(
                "render-replay artifact directory has an unsafe leaf: {}",
                path.display()
            ),
            ErrorType::Render,
        ));
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = std::fs::canonicalize(parent).or_raise(replay_error)?;
    validate_artifact_parent(&parent)?;
    let root = parent.join(name);
    create_artifact_dir(&root)?;
    Ok(root)
}

#[cfg(unix)]
fn validate_artifact_parent(parent: &Path) -> Result<()> {
    use std::os::unix::fs::MetadataExt;

    #[cfg(target_os = "linux")]
    let effective_uid = std::fs::metadata("/proc/self")
        .or_raise(replay_error)?
        .uid();
    for ancestor in parent.ancestors() {
        let metadata = std::fs::metadata(ancestor).or_raise(replay_error)?;
        let mode = metadata.mode();
        if mode & 0o022 != 0 && mode & 0o1000 == 0 {
            bail!(Error::new(
                format!(
                    "render-replay artifact parent is writable without sticky protection: {}",
                    ancestor.display()
                ),
                ErrorType::Render,
            ));
        }
        #[cfg(target_os = "linux")]
        if metadata.uid() != 0 && metadata.uid() != effective_uid {
            bail!(Error::new(
                format!(
                    "render-replay artifact parent is owned by an untrusted user: {}",
                    ancestor.display()
                ),
                ErrorType::Render,
            ));
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_artifact_parent(_parent: &Path) -> Result<()> {
    Ok(())
}

fn replay_error() -> Error {
    Error::new("failed to run corpus render replay", ErrorType::Render)
}

fn elapsed_micros(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX)
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cluster(class: &str) -> model::ReplayCluster {
        model::ReplayCluster {
            cluster_id: "cluster-test".to_owned(),
            failure_fingerprint: "fingerprint".to_owned(),
            signature: FailureSignature {
                class: class.to_owned(),
                stage: ReplayStage::Parse,
                key: "key".to_owned(),
            },
            case_ids: vec!["page-1".to_owned()],
            source_fullnames: vec!["example".to_owned()],
            representative_case_id: "page-1".to_owned(),
        }
    }

    #[test]
    fn timeout_minimization_uses_one_worker_to_avoid_contention_false_positives() {
        assert_eq!(
            minimization_probe_concurrency(&test_cluster("timeout"), 16),
            1
        );
        assert_eq!(
            minimization_probe_concurrency(&test_cluster("parser_errors"), 16),
            16,
        );
    }

    #[test]
    fn replay_gate_fails_closed_for_empty_failure_and_unverified_runs() {
        assert_eq!(
            gate_failures(None, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            vec!["missing_import_run", "empty_selection"],
        );
        assert_eq!(
            gate_failures(Some(217), 1, 0, 0, 0, 1, 0, 0, 0, 1),
            vec![
                "not_all_candidates_passed",
                "timed_out",
                "unverified_minimizations",
            ],
        );
        assert!(gate_failures(Some(217), 1, 1, 0, 0, 0, 0, 0, 0, 0).is_empty(),);
    }

    #[test]
    fn artifact_root_rejects_stale_files() {
        let path = std::env::temp_dir()
            .join(format!("deepwell-replay-stale-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("old-summary.json"), b"stale").unwrap();

        assert!(prepare_root_artifact_dir(&path).is_err());

        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn artifact_root_rejects_a_precreated_empty_directory() {
        let path = std::env::temp_dir().join(format!(
            "deepwell-replay-precreated-{}",
            uuid::Uuid::new_v4(),
        ));
        std::fs::create_dir(&path).unwrap();

        assert!(prepare_root_artifact_dir(&path).is_err());

        std::fs::remove_dir(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn artifact_root_and_files_are_private_and_no_replace() {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let parent = std::env::temp_dir().join(format!(
            "deepwell-replay-security-test-{}",
            uuid::Uuid::new_v4(),
        ));
        std::fs::create_dir(&parent).unwrap();
        let requested = parent.join("artifacts");
        let root = prepare_root_artifact_dir(&requested).unwrap();
        assert_eq!(
            std::fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            0o700,
        );

        let target = parent.join("target");
        std::fs::write(&target, b"untouched").unwrap();
        let artifact = root.join("summary.json");
        symlink(&target, &artifact).unwrap();
        assert!(write_bytes(&artifact, b"replacement").is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"untouched");

        std::fs::remove_dir_all(parent).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn artifact_root_rejects_a_non_sticky_writable_parent() {
        use std::os::unix::fs::PermissionsExt;

        let parent = std::env::temp_dir().join(format!(
            "deepwell-replay-untrusted-parent-{}",
            uuid::Uuid::new_v4(),
        ));
        std::fs::create_dir(&parent).unwrap();
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o777))
            .unwrap();

        assert!(prepare_root_artifact_dir(&parent.join("artifacts")).is_err());

        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700))
            .unwrap();
        std::fs::remove_dir(parent).unwrap();
    }
}
