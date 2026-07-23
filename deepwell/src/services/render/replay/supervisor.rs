/*
 * services/render/replay/supervisor.rs
 *
 * Isolated worker lifecycle. A deadline always ends in a force-kill followed
 * by wait/reap; dropping the future is also guarded by kill_on_drop.
 */

use super::features::timeout_signature;
use super::model::{
    FailureSignature, ReplayOutcome, ReplayStage, ReplayWorkerRequest, WorkerEvent,
    WorkerResult, normalize_diagnostic, sha256_hex,
};
use std::io;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::timeout;

const MAX_CAPTURE_BYTES: usize = 2 * 1024 * 1024;
const REAP_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub(crate) struct WorkerRun {
    pub result: WorkerResult,
    pub events: Vec<WorkerEvent>,
    #[cfg_attr(not(test), allow(dead_code))]
    pub pid: u32,
}

pub(crate) async fn run_isolated_worker(
    request: &ReplayWorkerRequest,
    deadline: Duration,
) -> WorkerRun {
    let input = match serde_json::to_vec(request) {
        Ok(input) => input,
        Err(error) => {
            return protocol_run(
                ReplayStage::Load,
                format!("failed to serialize worker request: {error}"),
            );
        }
    };
    let executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(error) => {
            return protocol_run(
                ReplayStage::Load,
                format!("failed to locate current executable: {error}"),
            );
        }
    };
    let mut command = Command::new(executable);
    command
        .env_clear()
        .env("DEEPWELL_RUNTIME_ACTION", "render-replay-worker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    supervise_command(command, input, deadline).await
}

async fn supervise_command(
    mut command: Command,
    input: Vec<u8>,
    deadline: Duration,
) -> WorkerRun {
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return protocol_run(
                ReplayStage::Load,
                format!("failed to spawn worker: {error}"),
            );
        }
    };
    let pid = child.id().unwrap_or(0);
    let mut stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let writer = tokio::spawn(async move {
        let Some(mut stdin) = stdin.take() else {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "worker stdin was not piped",
            ));
        };
        stdin.write_all(&input).await?;
        stdin.shutdown().await
    });
    let stdout_reader = tokio::spawn(async move {
        match stdout {
            Some(stdout) => read_capped(stdout).await,
            None => Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "worker stdout was not piped",
            )),
        }
    });
    let stderr_reader = tokio::spawn(async move {
        match stderr {
            Some(stderr) => read_capped(stderr).await,
            None => Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "worker stderr was not piped",
            )),
        }
    });

    let (status, timed_out, killed, reaped) = match timeout(deadline, child.wait()).await
    {
        Ok(result) => match result {
            Ok(status) => (Some(status), false, false, true),
            Err(_) => (None, false, false, false),
        },
        Err(_) => {
            // Account for the narrow race in which the process exits as the
            // timer fires. Otherwise send SIGKILL (on Unix) and always reap.
            match child.try_wait() {
                Ok(Some(status)) => (Some(status), false, false, true),
                _ => {
                    let killed = child.start_kill().is_ok();
                    let status = timeout(REAP_TIMEOUT, child.wait())
                        .await
                        .ok()
                        .and_then(Result::ok);
                    let reaped = status.is_some();
                    (status, true, killed, reaped)
                }
            }
        }
    };

    drop(child);
    if timed_out && !reaped {
        // No task may keep a pipe to an unconfirmed process alive past the
        // deadline. Dropping Child invokes kill_on_drop; aborting all pipe
        // tasks makes this failure path bounded even if the OS kill failed.
        writer.abort();
        stdout_reader.abort();
        stderr_reader.abort();
    }

    let mut transport_errors = Vec::new();
    collect_task_result("stdin writer", writer.await, &mut transport_errors);
    let stdout = collect_reader_result(
        "stdout reader",
        stdout_reader.await,
        &mut transport_errors,
    );
    let stderr = collect_reader_result(
        "stderr reader",
        stderr_reader.await,
        &mut transport_errors,
    );
    let (events, event_errors) = parse_events(&stdout);
    let last_stage = last_started_stage(&events);
    if timed_out {
        let (prepared_sha256, features) = prepared_event(&events)
            .unwrap_or_else(|| (String::new(), Default::default()));
        let signature = timeout_signature(last_stage, &features);
        return WorkerRun {
            result: WorkerResult {
                outcome: ReplayOutcome::Timeout {
                    stage: last_stage,
                    timeout_ms: duration_millis(deadline),
                    killed,
                    reaped,
                },
                signature,
                features,
                parse_errors: Vec::new(),
                parse_error_count: 0,
                prepared_sha256,
                ftml_core_rendered_sha256: None,
            },
            events,
            pid,
        };
    }

    if !transport_errors.is_empty() {
        return protocol_run_with_context(
            last_stage,
            format!("worker transport failure: {}", transport_errors.join("; ")),
            events,
            pid,
        );
    }
    if !event_errors.is_empty() {
        return protocol_run_with_context(
            last_stage,
            format!("malformed worker event stream: {}", event_errors.join("; ")),
            events,
            pid,
        );
    }

    if status.as_ref().is_some_and(ExitStatus::success)
        && let Some(result) = completed_result(&events)
    {
        return WorkerRun {
            result,
            events,
            pid,
        };
    }

    let stderr = String::from_utf8_lossy(&stderr);
    let normalized = normalize_diagnostic(&stderr);
    let signal = status.as_ref().and_then(exit_signal);
    let exit_code = status.as_ref().and_then(ExitStatus::code);
    let fingerprint = sha256_hex(normalized.as_bytes());
    let (prepared_sha256, features) =
        prepared_event(&events).unwrap_or_else(|| (String::new(), Default::default()));
    let signature = FailureSignature {
        class: "crash".to_owned(),
        stage: last_stage,
        key: fingerprint.clone(),
    };
    WorkerRun {
        result: WorkerResult {
            outcome: ReplayOutcome::Crash {
                exit_code,
                signal,
                stderr_fingerprint: fingerprint,
            },
            signature,
            features,
            parse_errors: Vec::new(),
            parse_error_count: 0,
            prepared_sha256,
            ftml_core_rendered_sha256: None,
        },
        events,
        pid,
    }
}

async fn read_capped(mut reader: impl AsyncRead + Unpin) -> io::Result<Vec<u8>> {
    let mut kept = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        let count = reader.read(&mut chunk).await?;
        if count == 0 {
            break;
        }
        let remaining = MAX_CAPTURE_BYTES.saturating_sub(kept.len());
        kept.extend_from_slice(&chunk[..count.min(remaining)]);
    }
    Ok(kept)
}

fn parse_events(output: &[u8]) -> (Vec<WorkerEvent>, Vec<String>) {
    let mut events = Vec::new();
    let mut errors = Vec::new();
    for (index, line) in output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .enumerate()
    {
        match serde_json::from_slice(line) {
            Ok(event) => events.push(event),
            Err(error) => errors.push(format!("line {}: {error}", index + 1)),
        }
    }
    (events, errors)
}

fn collect_task_result(
    name: &str,
    result: Result<io::Result<()>, tokio::task::JoinError>,
    errors: &mut Vec<String>,
) {
    match result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => errors.push(format!("{name}: {error}")),
        Err(error) => errors.push(format!("{name} join: {error}")),
    }
}

fn collect_reader_result(
    name: &str,
    result: Result<io::Result<Vec<u8>>, tokio::task::JoinError>,
    errors: &mut Vec<String>,
) -> Vec<u8> {
    match result {
        Ok(Ok(output)) => output,
        Ok(Err(error)) => {
            errors.push(format!("{name}: {error}"));
            Vec::new()
        }
        Err(error) => {
            errors.push(format!("{name} join: {error}"));
            Vec::new()
        }
    }
}

fn last_started_stage(events: &[WorkerEvent]) -> ReplayStage {
    events
        .iter()
        .fold(ReplayStage::Load, |_stage, event| match event {
            WorkerEvent::StageStarted { stage } => *stage,
            WorkerEvent::StageFinished { stage, .. } => *stage,
            WorkerEvent::Prepared { .. } => ReplayStage::Preprocess,
            WorkerEvent::Completed { .. } => ReplayStage::Complete,
        })
}

fn prepared_event(
    events: &[WorkerEvent],
) -> Option<(String, super::model::SyntaxFeatures)> {
    events.iter().rev().find_map(|event| match event {
        WorkerEvent::Prepared { sha256, features } => {
            Some((sha256.clone(), features.clone()))
        }
        _ => None,
    })
}

fn completed_result(events: &[WorkerEvent]) -> Option<WorkerResult> {
    events.iter().rev().find_map(|event| match event {
        WorkerEvent::Completed { result } => Some(result.clone()),
        _ => None,
    })
}

fn protocol_run(stage: ReplayStage, message: String) -> WorkerRun {
    protocol_run_with_context(stage, message, Vec::new(), 0)
}

fn protocol_run_with_context(
    stage: ReplayStage,
    message: String,
    events: Vec<WorkerEvent>,
    pid: u32,
) -> WorkerRun {
    WorkerRun {
        result: WorkerResult {
            outcome: ReplayOutcome::ProtocolError {
                message: message.clone(),
            },
            signature: FailureSignature::protocol(stage, &message),
            features: Default::default(),
            parse_errors: Vec::new(),
            parse_error_count: 0,
            prepared_sha256: String::new(),
            ftml_core_rendered_sha256: None,
        },
        events,
        pid,
    }
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(unix)]
fn exit_signal(status: &ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

#[cfg(not(unix))]
fn exit_signal(_status: &ExitStatus) -> Option<i32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[tokio::test]
    async fn deadline_force_kills_and_reaps_worker() {
        let mut command = Command::new("/bin/sh");
        command
            .arg("-c")
            .arg("while :; do :; done")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let run = supervise_command(command, Vec::new(), Duration::from_millis(50)).await;

        assert!(matches!(
            run.result.outcome,
            ReplayOutcome::Timeout {
                killed: true,
                reaped: true,
                ..
            }
        ));
        assert_ne!(run.pid, 0);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn crash_after_preparation_retains_prepared_identity() {
        let features = super::super::model::SyntaxFeatures {
            bytes: 42,
            lines: 3,
            ..Default::default()
        };
        let prepared = serde_json::to_string(&WorkerEvent::Prepared {
            sha256: "prepared-sha256".to_owned(),
            features: features.clone(),
        })
        .unwrap();
        let mut command = Command::new("/bin/sh");
        command
            .arg("-c")
            .arg("printf '%s\\n' \"$1\"; printf 'worker crashed' >&2; exit 17")
            .arg("render-replay-worker-test")
            .arg(prepared)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let run = supervise_command(command, Vec::new(), Duration::from_secs(2)).await;

        assert!(matches!(
            run.result.outcome,
            ReplayOutcome::Crash {
                exit_code: Some(17),
                ..
            }
        ));
        assert_eq!(run.result.prepared_sha256, "prepared-sha256");
        assert_eq!(run.result.features, features);
    }
}
