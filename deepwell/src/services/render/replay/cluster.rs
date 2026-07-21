/*
 * services/render/replay/cluster.rs
 *
 * Exact deterministic grouping. Heuristic syntax features are already part of
 * timeout signatures; parser errors use rule/kind/token/local-line shape.
 */

use super::model::{ReplayCaseObservation, ReplayCluster, ReplayOutcome, sha256_hex};
use std::collections::BTreeMap;

pub(crate) fn build_clusters(
    observations: &[ReplayCaseObservation],
) -> Vec<ReplayCluster> {
    let mut grouped = BTreeMap::new();
    for observation in observations {
        if matches!(
            observation.result.outcome,
            ReplayOutcome::Pass | ReplayOutcome::CompatibilityFallback
        ) {
            continue;
        }
        grouped
            .entry(failure_cluster_key(&observation.result))
            .or_insert_with(Vec::new)
            .push(observation);
    }

    let mut clusters = grouped
        .into_iter()
        .map(|(cluster_key, mut members)| {
            let signature = members[0].result.signature.clone();
            members.sort_by(|left, right| {
                left.result
                    .features
                    .bytes
                    .cmp(&right.result.features.bytes)
                    .then_with(|| left.case_id.cmp(&right.case_id))
            });
            let representative_case_id = members[0].case_id.clone();
            let failure_fingerprint = sha256_hex(cluster_key.as_bytes());
            let cluster_id = format!(
                "cluster-{}",
                &sha256_hex(
                    format!("{}|{:?}|{}", signature.class, signature.stage, cluster_key)
                        .as_bytes()
                )[..16],
            );
            let mut case_ids = members
                .iter()
                .map(|member| member.case_id.clone())
                .collect::<Vec<_>>();
            let mut source_fullnames = members
                .iter()
                .map(|member| member.source_fullname.clone())
                .collect::<Vec<_>>();
            case_ids.sort();
            source_fullnames.sort();
            ReplayCluster {
                cluster_id,
                failure_fingerprint,
                signature,
                case_ids,
                source_fullnames,
                representative_case_id,
            }
        })
        .collect::<Vec<_>>();
    clusters.sort_by(|left, right| {
        right
            .case_ids
            .len()
            .cmp(&left.case_ids.len())
            .then_with(|| left.cluster_id.cmp(&right.cluster_id))
    });
    clusters
}

pub(crate) fn failure_cluster_fingerprint(result: &super::model::WorkerResult) -> String {
    sha256_hex(failure_cluster_key(result).as_bytes())
}

fn failure_cluster_key(result: &super::model::WorkerResult) -> String {
    let signature = &result.signature;
    if matches!(result.outcome, ReplayOutcome::ParserErrors) {
        let line_shape = result
            .parse_errors
            .first()
            .map_or("none", |site| site.line_shape.as_str());
        return format!(
            "{}|{:?}|{}|{}",
            signature.class,
            signature.stage,
            signature.key,
            sha256_hex(line_shape.as_bytes()),
        );
    }
    if !matches!(result.outcome, ReplayOutcome::Timeout { .. }) {
        return format!(
            "{}|{:?}|{}",
            signature.class, signature.stage, signature.key
        );
    }
    let features = &result.features;
    let marker_presence = features
        .marker_counts
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{}|{:?}|{}|{}|{}",
        signature.class,
        signature.stage,
        features.dominant_line_shape.as_deref().unwrap_or("none"),
        features.unbalanced_mask.join(","),
        marker_presence,
    )
}

#[cfg(test)]
mod tests {
    use super::super::model::{
        FailureSignature, ReplayStage, SyntaxFeatures, WorkerResult,
    };
    use super::*;

    fn timeout_observation(shape: &str) -> ReplayCaseObservation {
        ReplayCaseObservation {
            case_id: shape.to_owned(),
            source_fullname: shape.to_owned(),
            page_id: 1,
            site_id: 1,
            state: "render_failed".to_owned(),
            expanded_sha256: String::new(),
            prepared_sha256: String::new(),
            included_page_count: 0,
            preparation_stages: Vec::new(),
            result: WorkerResult {
                outcome: ReplayOutcome::Timeout {
                    stage: ReplayStage::Parse,
                    timeout_ms: 100,
                    killed: true,
                    reaped: true,
                },
                signature: FailureSignature {
                    class: "timeout".to_owned(),
                    stage: ReplayStage::Parse,
                    key: "Parse".to_owned(),
                },
                features: SyntaxFeatures {
                    dominant_line_shape: Some(shape.to_owned()),
                    ..SyntaxFeatures::default()
                },
                parse_errors: Vec::new(),
                parse_error_count: 0,
                prepared_sha256: String::new(),
                ftml_core_rendered_sha256: None,
            },
            duration_ms: 100,
        }
    }

    #[test]
    fn stable_timeout_identity_can_split_into_feature_clusters() {
        let first = timeout_observation("quoted-align");
        let second = timeout_observation("ordered-size");

        assert_eq!(first.result.signature, second.result.signature);
        assert_ne!(
            failure_cluster_fingerprint(&first.result),
            failure_cluster_fingerprint(&second.result)
        );
    }
}
