/*
 * services/render/replay/ddmin.rs
 *
 * Deterministic line-oriented delta debugging. The caller supplies the exact
 * failure-signature predicate, so minimization cannot drift to an unrelated
 * timeout or parser error.
 */

use super::model::sha256_hex;
use futures::{StreamExt, stream};
use std::collections::BTreeMap;
use std::future::Future;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct DdminResult {
    pub minimized: String,
    pub original_lines: usize,
    pub minimized_lines: usize,
    pub probes: usize,
    pub cache_hits: usize,
    pub budget_exhausted: bool,
}

pub(crate) async fn ddmin_lines<F, Fut>(
    input: &str,
    max_probes: usize,
    concurrency: usize,
    predicate: F,
) -> DdminResult
where
    F: Fn(String) -> Fut + Clone,
    Fut: Future<Output = bool>,
{
    let original_lines = split_lines(input).len();
    let mut current = split_lines(input);
    let mut granularity = 2;
    let mut probes = 0;
    let mut cache_hits = 0;
    let mut cache = BTreeMap::<String, bool>::new();

    while current.len() >= 2 && probes < max_probes {
        let chunk_size = current.len().div_ceil(granularity);
        let mut reduced = false;

        let candidates = (0..current.len())
            .step_by(chunk_size)
            .filter_map(|start| {
                let end = (start + chunk_size).min(current.len());
                let candidate = current[..start]
                    .iter()
                    .chain(&current[end..])
                    .copied()
                    .collect::<String>();
                if candidate.is_empty() {
                    None
                } else {
                    Some((start, end, candidate))
                }
            })
            .collect::<Vec<_>>();
        let outcomes = probe_batch(
            candidates
                .iter()
                .map(|(_, _, candidate)| candidate.clone())
                .collect(),
            &mut cache,
            &mut probes,
            &mut cache_hits,
            max_probes,
            concurrency,
            predicate.clone(),
        )
        .await;
        for ((start, end, _), matches) in candidates.into_iter().zip(outcomes) {
            if matches {
                current.drain(start..end);
                granularity = granularity.saturating_sub(1).max(2);
                reduced = true;
                break;
            }
        }

        if !reduced {
            if granularity >= current.len() {
                break;
            }
            granularity = (granularity * 2).min(current.len());
        }
    }

    // Close the small gap left by chunk partitioning and make the artifact
    // explicitly 1-minimal with respect to line deletion.
    while current.len() > 1 && probes < max_probes {
        let candidates = (0..current.len())
            .map(|index| {
                current[..index]
                    .iter()
                    .chain(&current[index + 1..])
                    .copied()
                    .collect::<String>()
            })
            .collect::<Vec<_>>();
        let outcomes = probe_batch(
            candidates,
            &mut cache,
            &mut probes,
            &mut cache_hits,
            max_probes,
            concurrency,
            predicate.clone(),
        )
        .await;
        if let Some(index) = outcomes.iter().position(|matches| *matches) {
            current.remove(index);
        } else {
            break;
        }
    }

    let minimized = current.concat();
    DdminResult {
        minimized_lines: current.len(),
        minimized,
        original_lines,
        probes,
        cache_hits,
        budget_exhausted: probes >= max_probes,
    }
}

async fn probe_batch<F, Fut>(
    candidates: Vec<String>,
    cache: &mut BTreeMap<String, bool>,
    probes: &mut usize,
    cache_hits: &mut usize,
    max_probes: usize,
    concurrency: usize,
    predicate: F,
) -> Vec<bool>
where
    F: Fn(String) -> Fut + Clone,
    Fut: Future<Output = bool>,
{
    let mut outcomes = vec![None; candidates.len()];
    let mut jobs = Vec::new();
    let mut waiting = BTreeMap::<String, Vec<usize>>::new();
    for (index, candidate) in candidates.into_iter().enumerate() {
        let digest = sha256_hex(candidate.as_bytes());
        if let Some(result) = cache.get(&digest) {
            *cache_hits += 1;
            outcomes[index] = Some(*result);
        } else if let Some(indices) = waiting.get_mut(&digest) {
            // Repeated source lines can produce the same candidate many times
            // in one partition. One worker result is sufficient for all of
            // them and must not consume the probe budget repeatedly.
            *cache_hits += 1;
            indices.push(index);
        } else if *probes + jobs.len() < max_probes {
            waiting.insert(digest.clone(), vec![index]);
            jobs.push((digest, candidate));
        }
    }
    *probes += jobs.len();
    let completed = stream::iter(jobs)
        .map(|(digest, candidate)| {
            let predicate = predicate.clone();
            async move { (digest, predicate(candidate).await) }
        })
        .buffer_unordered(concurrency.max(1))
        .collect::<Vec<_>>()
        .await;
    for (digest, result) in completed {
        if let Some(indices) = waiting.remove(&digest) {
            for index in indices {
                outcomes[index] = Some(result);
            }
        }
        cache.insert(digest, result);
    }
    outcomes
        .into_iter()
        .map(Option::unwrap_or_default)
        .collect()
}

fn split_lines(input: &str) -> Vec<&str> {
    if input.is_empty() {
        Vec::new()
    } else {
        input.split_inclusive('\n').collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::{Duration, sleep};

    #[tokio::test]
    async fn minimizes_to_required_trigger_lines_deterministically() {
        let input = "noise-a\nBEGIN\nnoise-b\nTRIGGER\nnoise-c\nEND\nnoise-d\n";
        let result = ddmin_lines(input, 100, 4, |candidate| async move {
            candidate.contains("BEGIN\n")
                && candidate.contains("TRIGGER\n")
                && candidate.contains("END\n")
        })
        .await;

        assert_eq!(result.minimized, "BEGIN\nTRIGGER\nEND\n");
        assert_eq!(result.minimized_lines, 3);
        assert!(!result.budget_exhausted);
    }

    #[tokio::test]
    async fn obeys_probe_budget() {
        let result = ddmin_lines("a\nb\nc\nd\n", 1, 4, |_| async { false }).await;
        assert_eq!(result.probes, 1);
        assert!(result.budget_exhausted);
    }

    #[tokio::test]
    async fn probe_parallelism_is_bounded() {
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let active_for_probe = Arc::clone(&active);
        let maximum_for_probe = Arc::clone(&maximum);
        let _ = ddmin_lines("a\nb\nc\nd\ne\nf\ng\nh\n", 8, 3, move |_| {
            let active = Arc::clone(&active_for_probe);
            let maximum = Arc::clone(&maximum_for_probe);
            async move {
                let count = active.fetch_add(1, Ordering::SeqCst) + 1;
                maximum.fetch_max(count, Ordering::SeqCst);
                sleep(Duration::from_millis(10)).await;
                active.fetch_sub(1, Ordering::SeqCst);
                false
            }
        })
        .await;

        assert!(maximum.load(Ordering::SeqCst) > 1);
        assert!(maximum.load(Ordering::SeqCst) <= 3);
    }

    #[tokio::test]
    async fn repeated_candidates_share_one_probe_within_a_batch() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_probe = Arc::clone(&calls);
        let result = ddmin_lines("same\nsame\nsame\nsame\n", 32, 4, move |_| {
            let calls = Arc::clone(&calls_for_probe);
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                false
            }
        })
        .await;

        assert_eq!(result.probes, calls.load(Ordering::SeqCst));
        assert!(result.cache_hits > 0);
        assert!(result.probes < 10);
    }
}
