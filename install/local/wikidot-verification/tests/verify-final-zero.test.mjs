import assert from "node:assert/strict";
import {execFile as execFileCallback} from "node:child_process";
import {createHash} from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {promisify} from "node:util";
import test from "node:test";

import {LOCKFILES, buildFinalFrozenReceipt} from "../scripts/emit-final-frozen-receipt.mjs";
import {main, parseArgs, verifyFinalZero} from "../scripts/verify-final-zero.mjs";

const execFile = promisify(execFileCallback);

const ftmlCommit = "3".repeat(40);
const ftmlTree = "4".repeat(40);
const runId = "candidate-run-000000000001";
const candidateArtifactKey = "c".repeat(64);
const catalogApiIds = [
  "categories-select", "deleted-methods", "files-get-meta", "files-get-one", "files-save-one", "files-select", "overview", "pages-get-meta", "pages-get-one", "pages-save-one", "pages-select", "posts-get", "posts-select", "tags-select", "users-get-me",
];
const framerailIds = [
  "categories.select", "files.get_meta", "files.get_one", "files.save_one", "files.select", "pages.get_meta", "pages.get_one", "pages.save_one", "pages.select", "posts.get", "posts.select", "system.listMethods", "system.methodHelp", "system.methodSignature", "system.multicall", "tags.select", "users.get_me",
];
const wikidotPyIds = [
  "changes/SiteChangesListModule:parameters=options,page,perpage", "dashboard/messages/DMInboxModule:parameters=page?", "dashboard/messages/DMSentModule:parameters=page?", "dashboard/messages/DMViewMessageModule:parameters=item", "edit/EditMetaModule:parameters=pageId", "files/PageFilesModule:parameters=page_id", "forum/ForumCommentsListModule:parameters=pageId", "forum/ForumStartModule:parameters=hidden", "forum/ForumViewCategoryModule:parameters=c,p", "forum/ForumViewThreadModule:parameters=t", "forum/ForumViewThreadPostsModule:parameters=pageNo,t", "forum/sub/ForumEditPostFormModule:parameters=postId,threadId", "forum/sub/ForumPostRevisionModule:parameters=revisionId", "forum/sub/ForumPostRevisionsModule:parameters=postId", "history/PageRevisionListModule:parameters=options,page_id,perpage", "history/PageSourceModule:parameters=revision_id", "history/PageVersionModule:parameters=revision_id", "list/ListPagesModule:parameters=module_body,p,pagetype,page_type,page-type,category,tags,tag,parent,created_at,createdat,updated_at,updatedat,created_by,createdby,rating,score,name,fullname,full_slug,fullslug,range,order,offset,limit,perpage,per_page,separate,wrapper,rss,rsstitle,rssdescription,rsshome,rsslimit,rssonly", "managesite/ManageSiteMembersApplicationsModule:parameters=(none)", "membership/MembersListModule:parameters=group,page", "pagerate/WhoRatedPageModule:parameters=pageId", "viewsource/ViewSourceModule:parameters=page_id",
];

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function writeBytes(root, name, value) {
  const file = path.join(root, name);
  const bytes = Buffer.isBuffer(value) ? value : Buffer.from(value);
  await fs.writeFile(file, bytes, {mode: 0o600});
  return {path: file, sha256: sha256(bytes)};
}

async function writeJson(root, name, value) {
  return writeBytes(root, name, `${JSON.stringify(value)}\n`);
}

function deferredRecords() {
  return [
    ...catalogApiIds.map((id) => ({source_local_id: `catalog-feature:api-${id}`, kind: "catalog_feature", deferred_owner: "wikijump.xmlrpc-api"})),
    ...framerailIds.map((id) => ({source_local_id: `framerail-xmlrpc:${id}`, kind: "framerail_xmlrpc_method", deferred_owner: "wikijump.xmlrpc-api"})),
    ...wikidotPyIds.map((id) => ({source_local_id: `wikidot-py-amc-module:${id}`, kind: "wikidot_py_amc_module_shape", deferred_owner: "external.wikidot-py"})),
  ];
}

async function git(repository, ...arguments_) {
  const {stdout} = await execFile("git", ["-C", repository, ...arguments_], {encoding: "utf8"});
  return stdout.trim();
}

async function createMergeRepository(root, {identicalMergedTree = false} = {}) {
  const repository = path.join(root, "repository");
  await fs.mkdir(repository);
  await git(repository, "init", "--quiet", "--initial-branch=develop");
  await git(repository, "config", "user.email", "test@example.invalid");
  await git(repository, "config", "user.name", "Final Zero Test");
  await fs.writeFile(path.join(repository, "README"), "base\n");
  const validationSources = {
    "scripts/preflight.sh": "preflight\n",
    "scripts/run-test-no-external-network.sh": "network guard\n",
    "install/local/wikidot-verification/scripts/run-deepwell-integration-validation.mjs": "deepwell validation\n",
  };
  for (const [relative, contents] of Object.entries(validationSources)) {
    const file = path.join(repository, relative);
    await fs.mkdir(path.dirname(file), {recursive: true});
    await fs.writeFile(file, contents);
  }
  await git(repository, "add", "README", ...Object.keys(validationSources));
  await git(repository, "commit", "--quiet", "-m", "base");
  await git(repository, "switch", "--quiet", "-c", "candidate");
  await fs.mkdir(path.join(repository, "install", "standing"), {recursive: true});
  await fs.writeFile(path.join(repository, "install", "standing", "fixture.txt"), "candidate\n");
  await git(repository, "add", "install/standing/fixture.txt");
  await git(repository, "commit", "--quiet", "-m", "candidate");
  const candidateCommit = await git(repository, "rev-parse", "HEAD");
  await git(repository, "switch", "--quiet", "develop");
  if (!identicalMergedTree) {
    await fs.mkdir(path.join(repository, "install", "standing"), {recursive: true});
    await fs.writeFile(path.join(repository, "install", "standing", "develop.txt"), "develop\n");
    await git(repository, "add", "install/standing/develop.txt");
    await git(repository, "commit", "--quiet", "-m", "develop");
  }
  const developCommit = await git(repository, "rev-parse", "HEAD");
  await git(repository, "merge", "--quiet", "--no-ff", "candidate", "-m", "merge candidate");
  return {
    path: repository,
    candidateCommit,
    developCommit,
    mergeCommit: await git(repository, "rev-parse", "HEAD"),
    mergeTree: await git(repository, "rev-parse", "HEAD^{tree}"),
  };
}

function promotionPrecondition({candidateCommit, tree}) {
  const imageRoles = ["cache", "caddy", "database", "deepwell", "files", "framerail", "wws"];
  return {
    schema: "wikijump.standing_promotion_precondition.v1",
    status: "pass",
    run_id: runId,
    verified_at: "2026-08-15T00:00:00.000Z",
    admission: Object.fromEntries(["candidate_parity_receipt_sha256", "candidate_identity_sha256", "live_reference_sha256", "live_completion_policy_sha256", "source_runner_sha256", "source_observation_sha256", "source_execution_identity_sha256"].map((name, index) => [name, String(index + 1).repeat(64)])),
    candidate: {artifact_key: candidateArtifactKey, wikijump_commit: candidateCommit, wikijump_tree: tree, ftml_sha: ftmlCommit, compose_project: "wikijump-candidate", expires_at: "2026-08-16T00:00:00.000Z"},
    final_frozen_receipt: {path: "/tmp/final-frozen-receipt.json", sha256: "9".repeat(64)},
    build: {
      seal_sha256: "a".repeat(64),
      evidence_manifest_sha256: "b".repeat(64),
      verdict_sha256: "d".repeat(64),
      final_images_sha256: "e".repeat(64),
      run_id: runId,
      wikijump_commit: candidateCommit,
      wikijump_tree: tree,
      ftml_sha: ftmlCommit,
      images: Object.fromEntries(imageRoles.map((role) => [role, `sha256:${{cache: "4", caddy: "5", database: "6", deepwell: "1", files: "7", framerail: "2", wws: "3"}[role].repeat(64)}`])),
    },
    staging_home: {manifest_sha256: "f".repeat(64)},
  };
}

async function fixtures(t, options = {}) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "final-zero-"));
  t.after(() => fs.rm(root, {recursive: true, force: true}));
  const repository = await createMergeRepository(root, options);
  const wikijumpCommit = repository.mergeCommit;
  const wikijumpTree = repository.mergeTree;
  const promotion = promotionPrecondition({candidateCommit: repository.candidateCommit, tree: wikijumpTree});
  const inventory = await writeBytes(root, "inventory.json", "{\"frozen\":true}\n");
  const candidateArtifact = await writeBytes(root, "candidate-artifact.json", "candidate\n");
  const standingArtifact = await writeBytes(root, "standing-artifact.json", "standing\n");
  const promotionArtifact = await writeJson(root, "promotion-precondition.json", promotion);
  const validationReceipt = (commit, baseCommit) => ({
    schema: "wikijump.consolidated_validation_receipt.v2",
    status: "pass",
    started_at: "2026-09-12T00:00:00.000Z",
    completed_at: "2026-09-12T01:00:00.000Z",
    wikijump_commit: commit,
    wikijump_tree: wikijumpTree,
    base_commit: baseCommit,
    mode: "final",
    groups: {deepwell: true, verification: true},
    checks: [{name: "deepwell full tests", status: "ok"}],
    tool_versions: {node: "v24"},
    preflight: {exit: 0, log: {path: "/log", sha256: "4".repeat(64)}, source_sha256: {}},
  });
  const candidateValidation = await writeJson(root, "candidate-validation.json", validationReceipt(repository.candidateCommit, "5".repeat(40)));
  const frozenImageProducer = await writeJson(root, "frozen-images.json", {
    status: "pass",
    wikijump_sha: repository.candidateCommit,
    wikijump_tree: wikijumpTree,
    ftml_sha: ftmlCommit,
    images: {deepwell: {id: `sha256:${"1".repeat(64)}`}, framerail: {id: `sha256:${"2".repeat(64)}`}, wws: {id: `sha256:${"3".repeat(64)}`}},
  });
  for (const lockfile of LOCKFILES) {
    await fs.mkdir(path.dirname(path.join(root, lockfile)), {recursive: true});
    await writeBytes(root, lockfile, `lock ${lockfile}\n`);
  }
  const frozenReceipt = await buildFinalFrozenReceipt({
    source: {wikijump_commit: repository.candidateCommit, wikijump_tree: wikijumpTree, ftml_sha: ftmlCommit},
    sourceRoot: root,
    validationReceiptPath: candidateValidation.path,
    imageProducerPath: frozenImageProducer.path,
  });
  const finalFrozen = await writeJson(root, "final-frozen.json", frozenReceipt);
  const activeImages = Object.fromEntries(["deepwell", "framerail", "wws"].map((service, index) => [service, {reference: `sha256:${String(index + 1).repeat(64)}`, id: `sha256:${String(index + 1).repeat(64)}`, repo_digests: [], labels: {}}]));
  const rollbackImages = Object.fromEntries(["deepwell", "framerail", "wws"].map((service, index) => [service, {reference: `sha256:${String(index + 7).repeat(64)}`, id: `sha256:${String(index + 7).repeat(64)}`, repo_digests: [], labels: {}}]));
  const prepared = await writeJson(root, "prepared-receipt.json", {schema_version: 1, kind: "standing-image-preparation", status: "pass", run_id: runId, wikijump_sha: wikijumpCommit, wikijump_tree: wikijumpTree, ftml_sha: ftmlCommit, dependency_lock_sha256: "8".repeat(64), promotion_precondition: promotionArtifact, images: activeImages});
  const runtimeIdentityValue = {schema: "wikijump_syntax_differential.wikijump_runtime_identity.v1", wikijump_sha: wikijumpCommit, ftml_sha: ftmlCommit, dependency_lock_sha256: "8".repeat(64), executable_sha256: "1".repeat(64), runtime_config_sha256: "9".repeat(64)};
  const runtimeIdentity = await writeJson(root, "runtime-identity.json", runtimeIdentityValue);
  const standingRefresh = await writeJson(root, "standing-refresh.json", {schema_version: 1, kind: "standing-promotion", status: "pass", run_id: runId, started_at: "2026-08-15T00:00:00.000Z", completed_at: "2026-08-15T00:01:00.000Z", activation_duration_seconds: 0, image_verification_duration_seconds: 0, compose_activation_duration_seconds: 0, health_duration_seconds: 0, canary_duration_seconds: 0, wikijump_sha: wikijumpCommit, wikijump_tree: wikijumpTree, ftml_sha: ftmlCommit, dependency_lock_sha256: "8".repeat(64), promotion_precondition: promotionArtifact, runtime_home: "/tmp/wikijump-standing", prepared_receipt: prepared, project_name: "wikijump-standing", network_name: "wikijump-standing_default", images: activeImages, rollback_images: rollbackImages, protected_volumes: ["runtime50x-postgres-data", "runtime50x-files-data"], saved_page_render_freshness: {status: "pass", expected_compiled_generator: "ftml v1.0.0; deepwell-render/v1", stale_pages_remaining: 0}, runtime_differential_identity: {path: runtimeIdentity.path, sha256: runtimeIdentity.sha256, identity: runtimeIdentityValue}, health: {deepwell: "healthy", framerail: "healthy", wws: "healthy"}, canary: {url: "http://scp-wiki.wikijump.localhost/scp-9506", status: "pass", required_markers: ["scp-9506", "page-content"]}, cleanup: {status: "pass", candidate_receipt: prepared, receipt: prepared, superseded_images: []}, resource_disposition: {active: Object.fromEntries(Object.entries(activeImages).map(([service, image]) => [service, {owner: "standing-runtime", keep_until: "2026-08-16T00:00:00+00:00", id: image.id}])), rollback: Object.fromEntries(Object.entries(rollbackImages).map(([service, image]) => [service, {owner: "standing-runtime-rollback", keep_until: "2026-08-16T00:00:00+00:00", id: image.id}])), volumes: "protected-and-untouched", worktrees: "none created", target_directories: "none created"}});
  const records = deferredRecords();
  const surfaceId = "surface:00000001";
  const sourceLocalId = "catalog-feature:module-example";
  const row = {
    surface_id: surfaceId,
    actor: {state: "missing", reason: "not_recorded"},
    input: {state: "missing", reason: "not_recorded"},
    observable_interval: {state: "missing", reason: "not_recorded"},
    result: {state: "missing", reason: "not_recorded"},
    source: {state: "present", bindings: [{source_manifest_id: "manifest:00000001", raw_record_id: "raw:00000001"}]},
    evidence: {state: "present", references: []},
    tests: {state: "present", references: ["test:review.js#case"]},
    owners: {state: "present", specification: ["specification:00000001"], implementation: ["implementation:00000001"]},
    issues: {state: "present", numbers: [1365]},
    blockers: {state: "none", numbers: []},
    candidate: {state: "pass", artifacts: [candidateArtifact]},
    standing: {state: "pass", artifacts: [standingArtifact]},
    closure: {state: "closed", references: ["test:review.js#case"]},
  };
  const ledger = {
    schema: "wikijump.compatibility_ledger.v1",
    counts: {raw_records: 1, public_inventory_records: 1, canonical_surfaces: 1, input_alias_edges: 0, deduplication_relationships: 0},
    inputs: {inventory, wikijump: {commit: wikijumpCommit, tree: wikijumpTree}, ftml: {commit: ftmlCommit, tree: ftmlTree}},
    source_manifests: [{source_manifest_id: "manifest:00000001", source_class: "wikijump-consolidated-inventory", schema_id: "wikijump.compatibility_surface_inventory.v2", repository: "Rokurolize/wikijump", commit: wikijumpCommit, tree: wikijumpTree, path: inventory.path, sha256: inventory.sha256}],
    raw_source_records: [{source_manifest_id: "manifest:00000001", raw_record_id: "raw:00000001", record_sha256: "1".repeat(64)}],
    source_local_identities: [{source_manifest_id: "manifest:00000001", raw_record_id: "raw:00000001", source_local_id: sourceLocalId}],
    surface_assignments: [{assignment_id: "assignment:00000001", surface_id: surfaceId, source_manifest_id: "manifest:00000001", raw_record_id: "raw:00000001"}],
    relationships: [],
    deferred_exclusions: {count: records.length, by_kind: {catalog_feature: 15, framerail_xmlrpc_method: 17, wikidot_py_amc_module_shape: 22}, by_owner: {"external.wikidot-py": 22, "wikijump.xmlrpc-api": 32}, records},
    rows: [row],
  };
  const semantics = {
    actor: "public actor defined by the feature contract",
    input: "public invocation or persisted state for catalog-feature:module-example",
    observable_interval: "public request and browser-visible lifecycle",
    result: "Wikidot-compatible observable behavior for catalog-feature:module-example",
  };
  row.actor = {state: "known", value: semantics.actor};
  row.input = {state: "known", value: semantics.input};
  row.observable_interval = {state: "known", value: semantics.observable_interval};
  row.result = {state: "known", value: semantics.result};
  const denominator = {schema: "wikijump.compatibility_final_zero_denominator.v1", status: "sealed", rows: [{surface_id: surfaceId, source_local_id: sourceLocalId, kind: "catalog_feature", ...semantics}]};
  const deferredDenominator = {schema: "wikijump.compatibility_deferred_denominator.v1", status: "sealed", rows: records.map(({source_local_id: sourceLocalId, kind}) => ({surface_id: sourceLocalId, source_local_id: sourceLocalId, kind}))};
  const deferredLedger = {schema: "wikijump.compatibility_deferred_ledger.v1", status: "sealed", rows: records.map(({source_local_id: sourceLocalId, kind, deferred_owner: deferredOwner}) => ({surface_id: sourceLocalId, source_local_id: sourceLocalId, kind, deferred_owner: deferredOwner}))};
  const matrix = {
    schema: "wikijump.compatibility_standing_matrix.v2",
    status: "pass",
    run_id: runId,
    merge_commit: wikijumpCommit,
    merge_tree: wikijumpTree,
    ftml_sha: ftmlCommit,
    ftml_tree: ftmlTree,
    candidate_commit: repository.candidateCommit,
    candidate_artifact_key: candidateArtifactKey,
    promotion_precondition: promotionArtifact,
    standing_refresh: standingRefresh,
    rows: [{surface_id: surfaceId, source_local_id: sourceLocalId, kind: "catalog_feature", status: "pass", artifacts: [standingArtifact]}],
  };
  const consolidatedValidation = validationReceipt(wikijumpCommit, await git(repository.path, "rev-parse", `${wikijumpCommit}^1`));
  consolidatedValidation.preflight.source_sha256 = Object.fromEntries(await Promise.all([
    "scripts/preflight.sh",
    "scripts/run-test-no-external-network.sh",
    "install/local/wikidot-verification/scripts/run-deepwell-integration-validation.mjs",
  ].map(async (relative) => [relative, sha256(await fs.readFile(path.join(repository.path, relative)))])));
  const openIssues = {
    schema: "wikijump.current_open_issues.v1",
    repository: "Rokurolize/wikijump",
    captured_at: "2026-09-07T00:00:00.000Z",
    tracking_issue: 1089,
    issues: [{number: 1089, title: "tracking", url: "https://github.com/Rokurolize/wikijump/issues/1089"}],
  };
  const paths = {
    root,
    ledger: (await writeJson(root, "ledger.json", ledger)).path,
    denominator: (await writeJson(root, "denominator.json", denominator)).path,
    deferredDenominator: (await writeJson(root, "deferred-denominator.json", deferredDenominator)).path,
    deferredLedger: (await writeJson(root, "deferred-ledger.json", deferredLedger)).path,
    standingMatrix: (await writeJson(root, "standing-matrix.json", matrix)).path,
    finalFrozen: finalFrozen.path,
    consolidatedValidation: (await writeJson(root, "consolidated-validation.json", consolidatedValidation)).path,
    openIssues: (await writeJson(root, "open-issues.json", openIssues)).path,
    repository: repository.path,
  };
  return {paths, ledger, denominator, deferredDenominator, deferredLedger, matrix, consolidatedValidation, openIssues, repository};
}

function inputMap(fixture) {
  return {repository: fixture.paths.repository, ledger: fixture.paths.ledger, denominator: fixture.paths.denominator, deferredDenominator: fixture.paths.deferredDenominator, deferredLedger: fixture.paths.deferredLedger, standingMatrix: fixture.paths.standingMatrix, finalFrozen: fixture.paths.finalFrozen, consolidatedValidation: fixture.paths.consolidatedValidation, openIssues: fixture.paths.openIssues};
}

test("final-zero reconciles the exact denominator, row artifacts, deferred union, and canonical promotion", async (t) => {
  const fixture = await fixtures(t);
  const receipt = await verifyFinalZero(inputMap(fixture));
  assert.equal(receipt.status, "pass");
  assert.equal(receipt.merge_commit, fixture.repository.mergeCommit);
  assert.deepEqual(Object.keys(receipt.inputs).sort(), ["consolidated_validation", "deferred_denominator", "deferred_ledger", "denominator", "final_frozen", "ledger", "open_issues", "repository", "standing_matrix", "standing_refresh"]);
  const output = path.join(fixture.paths.root, "receipt.json");
  const args = ["--ledger", fixture.paths.ledger, "--denominator", fixture.paths.denominator, "--deferred-denominator", fixture.paths.deferredDenominator, "--deferred-ledger", fixture.paths.deferredLedger, "--standing-matrix", fixture.paths.standingMatrix, "--final-frozen", fixture.paths.finalFrozen, "--consolidated-validation", fixture.paths.consolidatedValidation, "--open-issues", fixture.paths.openIssues, "--repository", fixture.paths.repository, "--output", output];
  assert.equal(await main(args, {stdout: () => {}}), 0);
  assert.equal(await main(args, {stdout: () => {}}), 0);
});

test("final-zero accepts a normal merge whose tree is identical to the candidate tree", async (t) => {
  const fixture = await fixtures(t, {identicalMergedTree: true});
  assert.equal(fixture.repository.mergeTree, await git(fixture.repository.path, "rev-parse", `${fixture.repository.candidateCommit}^{tree}`));
  const receipt = await verifyFinalZero(inputMap(fixture));
  assert.equal(receipt.status, "pass");
  assert.equal(receipt.merge_commit, fixture.repository.mergeCommit);
});

test("final-zero rejects a currently open product issue even when its ledger row says closed", async (t) => {
  const fixture = await fixtures(t);
  fixture.openIssues.issues.push({number: 1365, title: "still open", url: "https://github.com/Rokurolize/wikijump/issues/1365"});
  await writeJson(fixture.paths.root, "open-issues.json", fixture.openIssues);
  await assert.rejects(verifyFinalZero(inputMap(fixture)), /complete_product_rows_open_or_unreconciled=1/u);
});

test("final-zero rejects a currently open product issue missing from the ledger", async (t) => {
  const fixture = await fixtures(t);
  fixture.openIssues.issues.push({number: 1999, title: "unowned", url: "https://github.com/Rokurolize/wikijump/issues/1999"});
  await writeJson(fixture.paths.root, "open-issues.json", fixture.openIssues);
  await assert.rejects(
    verifyFinalZero(inputMap(fixture)),
    /complete_product_rows_open_or_unreconciled=1, unrepresented_charter_requirements=1/u,
  );
});

test("final-zero rejects missing or extra current rows", async (t) => {
  const fixture = await fixtures(t);
  fixture.denominator.rows.push({surface_id: "surface:00000002", source_local_id: "catalog-feature:extra", kind: "catalog_feature", actor: "public actor", input: "public input", observable_interval: "public request", result: "public result"});
  await writeJson(fixture.paths.root, "denominator.json", fixture.denominator);
  await assert.rejects(verifyFinalZero(inputMap(fixture)), /missing or extra rows/u);
});

test("final-zero rejects missing or drifted semantic row values", async (t) => {
  const missing = await fixtures(t);
  missing.ledger.rows[0].actor = {state: "missing", reason: "not_recorded"};
  await writeJson(missing.paths.root, "ledger.json", missing.ledger);
  await assert.rejects(verifyFinalZero(inputMap(missing)), /semantic value is missing or drifted/u);

  const drifted = await fixtures(t);
  drifted.ledger.rows[0].result = {state: "known", value: "different result"};
  await writeJson(drifted.paths.root, "ledger.json", drifted.ledger);
  await assert.rejects(verifyFinalZero(inputMap(drifted)), /semantic value is missing or drifted/u);
});

test("final-zero treats public tests and the frozen validation as distinct gates", async (t) => {
  const missingTest = await fixtures(t);
  missingTest.ledger.rows[0].tests = {state: "missing", reason: "not_written"};
  await writeJson(missingTest.paths.root, "ledger.json", missingTest.ledger);
  await assert.rejects(verifyFinalZero(inputMap(missingTest)), /unrepresented_charter_requirements=1/u);

  const failedValidation = await fixtures(t);
  const frozen = JSON.parse(await fs.readFile(failedValidation.paths.finalFrozen, "utf8"));
  const validationPath = frozen.validation.path;
  const validationValue = JSON.parse(await fs.readFile(validationPath, "utf8"));
  validationValue.status = "fail";
  validationValue.checks = [{name: "deepwell full tests", status: "failed"}];
  await fs.writeFile(validationPath, `${JSON.stringify(validationValue)}\n`);
  await assert.rejects(verifyFinalZero(inputMap(failedValidation)), /final frozen consolidated validation is stale|did not pass/u);
});

test("final-zero rejects a non-exact deferred wikidot.py/XML-RPC exclusion", async (t) => {
  const fixture = await fixtures(t);
  fixture.deferredDenominator.rows[0].source_local_id = "catalog-feature:api-not-frozen";
  await writeJson(fixture.paths.root, "deferred-denominator.json", fixture.deferredDenominator);
  await assert.rejects(verifyFinalZero(inputMap(fixture)), /unknown or wrong deferred identity/u);
});

test("final-zero rejects any catalog XML-RPC row leaking into the current ledger", async (t) => {
  for (const id of catalogApiIds) {
    const fixture = await fixtures(t);
    const sourceLocalId = `catalog-feature:api-${id}`;
    fixture.ledger.source_local_identities[0].source_local_id = sourceLocalId;
    await writeJson(fixture.paths.root, "ledger.json", fixture.ledger);
    await assert.rejects(verifyFinalZero(inputMap(fixture)), new RegExp(`contains deferred work: ${sourceLocalId}`));
  }
});

test("final-zero rejects deferred omission and reclassification", async (t) => {
  const omitted = await fixtures(t);
  omitted.deferredDenominator.rows.pop();
  await writeJson(omitted.paths.root, "deferred-denominator.json", omitted.deferredDenominator);
  await assert.rejects(verifyFinalZero(inputMap(omitted)), /exactly 54 rows/u);

  const reclassified = await fixtures(t);
  reclassified.deferredLedger.rows[0].kind = "framerail_xmlrpc_method";
  await writeJson(reclassified.paths.root, "deferred-ledger.json", reclassified.deferredLedger);
  await assert.rejects(verifyFinalZero(inputMap(reclassified)), /unknown, wrong, or duplicate row/u);
});

test("final-zero binds candidate artifact_key through the canonical promotion validator", async (t) => {
  const fixture = await fixtures(t);
  fixture.matrix.candidate_artifact_key = "d".repeat(64);
  await writeJson(fixture.paths.root, "standing-matrix.json", fixture.matrix);
  await assert.rejects(verifyFinalZero(inputMap(fixture)), /candidate artifact_key does not match/u);
});

test("final-zero rejects a candidate PR head used as the merge commit", async (t) => {
  const fixture = await fixtures(t);
  fixture.matrix.merge_commit = fixture.repository.candidateCommit;
  await writeJson(fixture.paths.root, "standing-matrix.json", fixture.matrix);
  await assert.rejects(verifyFinalZero(inputMap(fixture)), /candidate commit must differ from merge commit|matrix merge commit does not match the canonical source/u);
});

test("final-zero rejects a wrong merge tree or merge parent", async (t) => {
  const wrongTree = await fixtures(t);
  wrongTree.matrix.merge_tree = "f".repeat(40);
  await writeJson(wrongTree.paths.root, "standing-matrix.json", wrongTree.matrix);
  await assert.rejects(verifyFinalZero(inputMap(wrongTree)), /matrix merge tree does not match|final frozen source wikijump_tree does not match/u);

  const wrongParent = await fixtures(t);
  wrongParent.matrix.candidate_commit = wrongParent.repository.developCommit;
  await writeJson(wrongParent.paths.root, "standing-matrix.json", wrongParent.matrix);
  await assert.rejects(verifyFinalZero(inputMap(wrongParent)), /promotion candidate PR head does not match|final frozen source wikijump_commit does not match/u);
});

test("final-zero rejects a standing matrix without its digest-bound standing refresh receipt", async (t) => {
  const fixture = await fixtures(t);
  delete fixture.matrix.standing_refresh;
  await writeJson(fixture.paths.root, "standing-matrix.json", fixture.matrix);
  await assert.rejects(verifyFinalZero(inputMap(fixture)), /standing matrix has missing or unknown fields/u);
});

test("final-zero rejects a standing refresh with stale saved pages", async (t) => {
  const fixture = await fixtures(t);
  const refresh = JSON.parse(await fs.readFile(fixture.matrix.standing_refresh.path, "utf8"));
  refresh.saved_page_render_freshness.stale_pages_remaining = 1;
  const changed = await writeJson(fixture.paths.root, "standing-refresh.json", refresh);
  fixture.matrix.standing_refresh = changed;
  await writeJson(fixture.paths.root, "standing-matrix.json", fixture.matrix);
  await assert.rejects(
    verifyFinalZero(inputMap(fixture)),
    /standing refresh saved page render freshness is not passing/u,
  );
});

test("final-zero rejects missing, stale, or non-final consolidated validation", async (t) => {
  const stale = await fixtures(t);
  stale.consolidatedValidation.wikijump_commit = stale.repository.candidateCommit;
  await writeJson(stale.paths.root, "consolidated-validation.json", stale.consolidatedValidation);
  await assert.rejects(verifyFinalZero(inputMap(stale)), /source identity is stale/u);

  const wrongBase = await fixtures(t);
  wrongBase.consolidatedValidation.base_commit = wrongBase.repository.candidateCommit;
  await writeJson(wrongBase.paths.root, "consolidated-validation.json", wrongBase.consolidatedValidation);
  await assert.rejects(verifyFinalZero(inputMap(wrongBase)), /base is not the merge first parent/u);

  const checkpoint = await fixtures(t);
  checkpoint.consolidatedValidation.mode = "checkpoint";
  await writeJson(checkpoint.paths.root, "consolidated-validation.json", checkpoint.consolidatedValidation);
  await assert.rejects(verifyFinalZero(inputMap(checkpoint)), /must run the final barrier/u);
});

test("final-zero rejects symlinked input and artifact paths", async (t) => {
  const fixture = await fixtures(t);
  const symlink = path.join(fixture.paths.root, "standing-matrix-link.json");
  await fs.symlink(fixture.paths.standingMatrix, symlink);
  const symlinked = {...inputMap(fixture), standingMatrix: symlink};
  await assert.rejects(verifyFinalZero(symlinked), /symbolic link/u);

  const value = JSON.parse(await fs.readFile(fixture.paths.ledger, "utf8"));
  const artifactLink = path.join(fixture.paths.root, "candidate-artifact-link.json");
  await fs.symlink(value.rows[0].candidate.artifacts[0].path, artifactLink);
  value.rows[0].candidate.artifacts[0].path = artifactLink;
  await writeJson(fixture.paths.root, "ledger.json", value);
  await assert.rejects(verifyFinalZero(inputMap(fixture)), /symbolic link/u);
});

test("final-zero CLI requires every frozen input", () => {
  assert.deepEqual(parseArgs(["--ledger", "/a", "--denominator", "/b", "--deferred-denominator", "/c", "--deferred-ledger", "/d", "--standing-matrix", "/e", "--final-frozen", "/f", "--consolidated-validation", "/j", "--open-issues", "/i", "--repository", "/g", "--output", "/h"]), {ledger: "/a", denominator: "/b", "deferred-denominator": "/c", "deferred-ledger": "/d", "standing-matrix": "/e", "final-frozen": "/f", "consolidated-validation": "/j", "open-issues": "/i", repository: "/g", output: "/h"});
  assert.throws(() => parseArgs(["--ledger", "/a", "--denominator", "/b", "--deferred-denominator", "/c", "--deferred-ledger", "/d", "--standing-matrix", "/e", "--final-frozen", "/f", "--open-issues", "/i", "--repository", "/g", "--output", "/h"]), /--consolidated-validation is required/u);
});
