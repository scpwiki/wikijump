/*
 * services/render/replay/worker.rs
 *
 * Credential-free, one-request worker. The controller always runs this in a
 * separate process so parser CPU cannot survive a deadline.
 */

use super::features::{error_sites, parser_error_signature, syntax_features};
use super::model::{
    FailureSignature, REPLAY_SCHEMA, ReplayOutcome, ReplayStage, ReplayWorkerRequest,
    WorkerEvent, WorkerResult, sha256_hex,
};
use crate::services::render::{CorpusReplayPreparationStage, RenderService};
use ftml::render::Render;
use ftml::render::html::HtmlRender;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::sync::LazyLock;
use std::time::Instant;

static RANDOM_PROTECTION_SENTINEL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?P<prefix>WIKIJUMPWIKIDOT(?:COMPATLINK|COMPATHTML|COLORSPAN|INLINEHTML|LISTPAGESELLIPSIS))(?P<nonce>[0-9a-f]{32})X",
    )
    .unwrap()
});
static FTML_RANDOM_HTML_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?P<attribute>(?:id|aria-controls|aria-labelledby)=")(?P<id>wj-id-[A-Za-z0-9]{16})"#,
    )
    .unwrap()
});

pub(crate) fn run_worker_action() -> i32 {
    match run_worker() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("render-replay worker failed: {error}");
            2
        }
    }
}

fn run_worker() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    let request: ReplayWorkerRequest = serde_json::from_slice(&input)?;
    if request.schema != REPLAY_SCHEMA {
        return Err(format!("unsupported replay schema: {}", request.schema).into());
    }

    let preparation_input_bytes = request.expanded.wikitext.len();
    let prepared = RenderService::prepare_corpus_replay_wikitext_with_observer(
        request.expanded,
        |stage| {
            let stage = match stage {
                CorpusReplayPreparationStage::Preprocess => ReplayStage::Preprocess,
                CorpusReplayPreparationStage::Normalization
                | CorpusReplayPreparationStage::OuterProtection
                | CorpusReplayPreparationStage::FallbackCheck
                | CorpusReplayPreparationStage::InnerProtection => ReplayStage::Protect,
            };
            let _ = emit(&WorkerEvent::StageStarted { stage });
        },
    );
    let canonical_prepared = canonicalize_replay_sentinels(&prepared.wikitext);
    if prepared.preprocessed
        && let Some(path) = request.emit_prepared_path
    {
        write_atomic(&path, canonical_prepared.as_bytes())?;
    }
    let protection_us = prepared
        .timings
        .normalization_us
        .saturating_add(prepared.timings.outer_protection_us)
        .saturating_add(prepared.timings.fallback_check_us)
        .saturating_add(prepared.timings.inner_protection_us);
    emit(&WorkerEvent::StageFinished {
        stage: ReplayStage::Protect,
        elapsed_us: protection_us,
        input_bytes: preparation_input_bytes,
        output_bytes: prepared.wikitext.len(),
    })?;
    if prepared.preprocessed {
        emit(&WorkerEvent::StageFinished {
            stage: ReplayStage::Preprocess,
            elapsed_us: prepared.timings.preprocess_us,
            input_bytes: prepared.wikitext.len(),
            output_bytes: prepared.wikitext.len(),
        })?;
    }

    let features = syntax_features(&canonical_prepared);
    let prepared_sha256 = sha256_hex(canonical_prepared.as_bytes());
    emit(&WorkerEvent::Prepared {
        sha256: prepared_sha256.clone(),
        features: features.clone(),
    })?;
    if prepared.compatibility_fallback {
        return emit(&WorkerEvent::Completed {
            result: WorkerResult {
                outcome: ReplayOutcome::CompatibilityFallback,
                signature: FailureSignature {
                    class: "compatibility_fallback".to_owned(),
                    stage: ReplayStage::Complete,
                    key: "compatibility_fallback".to_owned(),
                },
                features,
                parse_errors: Vec::new(),
                parse_error_count: 0,
                prepared_sha256,
                ftml_core_rendered_sha256: None,
            },
        });
    }

    emit(&WorkerEvent::StageStarted {
        stage: ReplayStage::Tokenize,
    })?;
    let started = Instant::now();
    let tokens = ftml::tokenize(&prepared.wikitext);
    emit(&WorkerEvent::StageFinished {
        stage: ReplayStage::Tokenize,
        elapsed_us: elapsed_micros(started),
        input_bytes: prepared.wikitext.len(),
        output_bytes: tokens.tokens().len(),
    })?;

    emit(&WorkerEvent::StageStarted {
        stage: ReplayStage::Parse,
    })?;
    let started = Instant::now();
    let result = ftml::parse(&tokens, &prepared.page_info, &prepared.settings);
    let (tree, errors) = result.into();
    emit(&WorkerEvent::StageFinished {
        stage: ReplayStage::Parse,
        elapsed_us: elapsed_micros(started),
        input_bytes: tokens.tokens().len(),
        output_bytes: errors.len(),
    })?;

    let sites = error_sites(&canonical_prepared, &errors);
    let signature = if errors.is_empty() {
        FailureSignature::pass()
    } else {
        parser_error_signature(&sites)
    };

    emit(&WorkerEvent::StageStarted {
        stage: ReplayStage::Render,
    })?;
    let started = Instant::now();
    let html = HtmlRender.render(&tree, &prepared.page_info, &prepared.settings);
    emit(&WorkerEvent::StageFinished {
        stage: ReplayStage::Render,
        elapsed_us: elapsed_micros(started),
        input_bytes: prepared.wikitext.len(),
        output_bytes: html.body.len(),
    })?;

    emit(&WorkerEvent::Completed {
        result: WorkerResult {
            outcome: if errors.is_empty() {
                ReplayOutcome::Pass
            } else {
                ReplayOutcome::ParserErrors
            },
            signature,
            features,
            parse_errors: sites,
            parse_error_count: errors.len(),
            prepared_sha256,
            ftml_core_rendered_sha256: Some(sha256_hex(
                canonicalize_ftml_core_html(&html.body).as_bytes(),
            )),
        },
    })
}

fn canonicalize_replay_sentinels(value: &str) -> String {
    let mut ordinals = BTreeMap::<String, usize>::new();
    let mut next_ordinal = 1usize;
    RANDOM_PROTECTION_SENTINEL_REGEX
        .replace_all(value, |captures: &regex::Captures<'_>| {
            let original = captures.get(0).expect("sentinel capture exists").as_str();
            let ordinal = *ordinals.entry(original.to_owned()).or_insert_with(|| {
                let ordinal = next_ordinal;
                next_ordinal += 1;
                ordinal
            });
            format!("{}{:032x}X", &captures["prefix"], ordinal)
        })
        .into_owned()
}

fn canonicalize_ftml_core_html(value: &str) -> String {
    let value = canonicalize_replay_sentinels(value);
    let mut ordinals = BTreeMap::<String, usize>::new();
    let mut next_ordinal = 1usize;
    FTML_RANDOM_HTML_ID_REGEX
        .replace_all(&value, |captures: &regex::Captures<'_>| {
            let original = captures["id"].to_owned();
            let ordinal = *ordinals.entry(original).or_insert_with(|| {
                let ordinal = next_ordinal;
                next_ordinal += 1;
                ordinal
            });
            format!("{}wj-id-{ordinal:016x}\"", &captures["attribute"])
        })
        .into_owned()
}

fn emit(event: &WorkerEvent) -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    serde_json::to_writer(&mut stdout, event)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

fn elapsed_micros(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX)
}

fn write_atomic(path: &std::path::Path, contents: &[u8]) -> io::Result<()> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(format!(
        ".tmp-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4(),
    ));
    let temporary = std::path::PathBuf::from(temporary);
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        file.write_all(contents)?;
        file.flush()?;
        drop(file);

        // hard_link is an atomic no-replace publish: unlike rename, it fails
        // if the final artifact name already exists, including as a symlink.
        std::fs::hard_link(&temporary, path)
    })();
    match result {
        Ok(()) => std::fs::remove_file(temporary),
        Err(error) => {
            let _ = std::fs::remove_file(temporary);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_protection_sentinels_have_stable_distinct_canonical_ids() {
        let first = concat!(
            "WIKIJUMPWIKIDOTCOMPATHTMLaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaX ",
            "WIKIJUMPWIKIDOTCOMPATLINKbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbX ",
            "WIKIJUMPWIKIDOTCOMPATHTMLaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaX",
        );
        let second = concat!(
            "WIKIJUMPWIKIDOTCOMPATHTML11111111111111111111111111111111X ",
            "WIKIJUMPWIKIDOTCOMPATLINK22222222222222222222222222222222X ",
            "WIKIJUMPWIKIDOTCOMPATHTML11111111111111111111111111111111X",
        );

        let first = canonicalize_replay_sentinels(first);
        let second = canonicalize_replay_sentinels(second);
        assert_eq!(first, second);
        assert_eq!(
            first.matches("00000000000000000000000000000001X").count(),
            2,
        );
        assert_eq!(
            first.matches("00000000000000000000000000000002X").count(),
            1,
        );
    }

    #[test]
    fn ftml_random_html_ids_and_references_have_stable_canonical_ids() {
        let first = concat!(
            r#"<wj-tabs-button id="wj-id-zvGvLlhGI6VEZFKj" aria-controls="wj-id-e9pQyKaPmLnulpgn">"#,
            r#"</wj-tabs-button><div id="wj-id-e9pQyKaPmLnulpgn" "#,
            r#"aria-labelledby="wj-id-zvGvLlhGI6VEZFKj"></div>"#,
        );
        let second = concat!(
            r#"<wj-tabs-button id="wj-id-1111111111111111" aria-controls="wj-id-2222222222222222">"#,
            r#"</wj-tabs-button><div id="wj-id-2222222222222222" "#,
            r#"aria-labelledby="wj-id-1111111111111111"></div>"#,
        );

        let first = canonicalize_ftml_core_html(first);
        let second = canonicalize_ftml_core_html(second);
        assert_eq!(first, second);
        assert_eq!(first.matches("wj-id-0000000000000001").count(), 2);
        assert_eq!(first.matches("wj-id-0000000000000002").count(), 2);
    }

    #[test]
    fn prepared_artifact_publish_never_replaces_an_existing_name() {
        let directory = std::env::temp_dir().join(format!(
            "deepwell-replay-worker-artifact-{}",
            uuid::Uuid::new_v4(),
        ));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("prepared.wikidot");

        write_atomic(&path, b"first").unwrap();
        assert!(write_atomic(&path, b"second").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"first");

        std::fs::remove_dir_all(directory).unwrap();
    }
}
