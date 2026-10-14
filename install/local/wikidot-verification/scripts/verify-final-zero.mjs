#!/usr/bin/env node

import {execFile as execFileCallback} from "node:child_process";
import {constants as fsConstants} from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";
import {promisify} from "node:util";

import compatibilityContract from "../../../../docs/development/compatibility-denominator-contract.json" with {type: "json"};

import {validateStandingPromotionPrecondition} from "../../../standing/scripts/verify-promotion-precondition.mjs";
import {validateStandingRefreshReceipt} from "../../../standing/scripts/verify-standing-refresh.mjs";
import {sha256Hex} from "../src/canonical-json.mjs";
import {runCliIfMain} from "../src/cli-entry.mjs";
import {validateConsolidatedValidation, verifyFinalFrozenReceipt} from "../src/final-frozen-receipt-contract.mjs";
import {sealJsonNoReplace} from "../src/standing-browser-parity-util.mjs";

const FINAL_ZERO_CLASSES = Object.freeze(compatibilityContract.vocabularies.final_zero_nonzero_classes);
const execFile = promisify(execFileCallback);
const LEDGER_SCHEMA = "wikijump.compatibility_ledger.v1";
const CURRENT_DENOMINATOR_SCHEMA = "wikijump.compatibility_final_zero_denominator.v1";
const DEFERRED_DENOMINATOR_SCHEMA = "wikijump.compatibility_deferred_denominator.v1";
const DEFERRED_LEDGER_SCHEMA = "wikijump.compatibility_deferred_ledger.v1";
const STANDING_MATRIX_SCHEMA = "wikijump.compatibility_standing_matrix.v2";
const CURRENT_OPEN_ISSUES_SCHEMA = "wikijump.current_open_issues.v1";
const TRACKING_ISSUE = 1089;
const CONSOLIDATED_VALIDATION_SOURCE_FILES = Object.freeze([
  "scripts/preflight.sh",
  "scripts/run-test-no-external-network.sh",
  "install/local/wikidot-verification/scripts/run-deepwell-integration-validation.mjs",
]);
const LEDGER_FIELDS = ["schema", "counts", "inputs", "source_manifests", "raw_source_records", "source_local_identities", "surface_assignments", "relationships", "deferred_exclusions", "rows"];
const LEDGER_COUNT_FIELDS = ["raw_records", "public_inventory_records", "canonical_surfaces", "input_alias_edges", "deduplication_relationships"];
const LEDGER_INPUT_FIELDS = ["inventory", "wikijump", "ftml"];
const ROW_FIELDS = ["surface_id", "actor", "input", "observable_interval", "result", "source", "evidence", "tests", "owners", "issues", "blockers", "candidate", "standing", "closure"];
const MATRIX_FIELDS = ["schema", "status", "run_id", "merge_commit", "merge_tree", "ftml_sha", "ftml_tree", "candidate_commit", "candidate_artifact_key", "promotion_precondition", "standing_refresh", "rows"];
const MATRIX_ROW_FIELDS = ["surface_id", "source_local_id", "kind", "status", "artifacts"];
const DENOMINATOR_FIELDS = ["schema", "status", "rows"];
const CURRENT_DENOMINATOR_ROW_FIELDS = ["surface_id", "source_local_id", "kind", "actor", "input", "observable_interval", "result"];
const DEFERRED_DENOMINATOR_ROW_FIELDS = ["surface_id", "source_local_id", "kind"];
const DEFERRED_LEDGER_FIELDS = ["schema", "status", "rows"];
const DEFERRED_LEDGER_ROW_FIELDS = ["surface_id", "source_local_id", "kind", "deferred_owner"];
const HEX40 = /^[0-9a-f]{40}$/u;
const HEX64 = /^[0-9a-f]{64}$/u;
const CANONICAL_SURFACE_ID = /^surface:[0-9]{8}$/u;
const DEFERRED_EXPECTED = Object.freeze([
  ["catalog-feature:api-categories-select", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-deleted-methods", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-files-get-meta", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-files-get-one", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-files-save-one", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-files-select", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-overview", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-pages-get-meta", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-pages-get-one", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-pages-save-one", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-pages-select", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-posts-get", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-posts-select", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-tags-select", "catalog_feature", "wikijump.xmlrpc-api"],
  ["catalog-feature:api-users-get-me", "catalog_feature", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:categories.select", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:files.get_meta", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:files.get_one", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:files.save_one", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:files.select", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:pages.get_meta", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:pages.get_one", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:pages.save_one", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:pages.select", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:posts.get", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:posts.select", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:system.listMethods", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:system.methodHelp", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:system.methodSignature", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:system.multicall", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:tags.select", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["framerail-xmlrpc:users.get_me", "framerail_xmlrpc_method", "wikijump.xmlrpc-api"],
  ["wikidot-py-amc-module:changes/SiteChangesListModule:parameters=options,page,perpage", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:dashboard/messages/DMInboxModule:parameters=page?", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:dashboard/messages/DMSentModule:parameters=page?", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:dashboard/messages/DMViewMessageModule:parameters=item", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:edit/EditMetaModule:parameters=pageId", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:files/PageFilesModule:parameters=page_id", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:forum/ForumCommentsListModule:parameters=pageId", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:forum/ForumStartModule:parameters=hidden", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:forum/ForumViewCategoryModule:parameters=c,p", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:forum/ForumViewThreadModule:parameters=t", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:forum/ForumViewThreadPostsModule:parameters=pageNo,t", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:forum/sub/ForumEditPostFormModule:parameters=postId,threadId", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:forum/sub/ForumPostRevisionModule:parameters=revisionId", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:forum/sub/ForumPostRevisionsModule:parameters=postId", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:history/PageRevisionListModule:parameters=options,page_id,perpage", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:history/PageSourceModule:parameters=revision_id", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:history/PageVersionModule:parameters=revision_id", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:list/ListPagesModule:parameters=module_body,p,pagetype,page_type,page-type,category,tags,tag,parent,created_at,createdat,updated_at,updatedat,created_by,createdby,rating,score,name,fullname,full_slug,fullslug,range,order,offset,limit,perpage,per_page,separate,wrapper,rss,rsstitle,rssdescription,rsshome,rsslimit,rssonly", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:managesite/ManageSiteMembersApplicationsModule:parameters=(none)", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:membership/MembersListModule:parameters=group,page", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:pagerate/WhoRatedPageModule:parameters=pageId", "wikidot_py_amc_module_shape", "external.wikidot-py"],
  ["wikidot-py-amc-module:viewsource/ViewSourceModule:parameters=page_id", "wikidot_py_amc_module_shape", "external.wikidot-py"],
].map(([sourceLocalId, kind, deferredOwner]) => ({sourceLocalId, kind, deferredOwner})));
const DEFERRED_EXPECTED_BY_ID = new Map(DEFERRED_EXPECTED.map((row) => [row.sourceLocalId, row]));

function fail(message) {
  throw new Error(message);
}

function exactKeys(value, expected, name) {
  if (value === null || typeof value !== "object" || Array.isArray(value) || JSON.stringify(Object.keys(value).sort()) !== JSON.stringify([...expected].sort())) fail(`${name} has missing or unknown fields`);
}

function requireNonEmptyString(value, name) {
  if (typeof value !== "string" || value.trim() === "") fail(`${name} must be a non-empty string`);
  return value;
}

function statFingerprint(stat) {
  return JSON.stringify({dev: String(stat.dev), ino: String(stat.ino), nlink: String(stat.nlink), mode: String(stat.mode), size: String(stat.size), mtimeNs: String(stat.mtimeNs), ctimeNs: String(stat.ctimeNs)});
}

function requireRegularFile(stat, name) {
  if (!stat?.isFile() || stat.isSymbolicLink() || (stat.nlink !== 1 && stat.nlink !== 1n)) fail(`${name} must be a regular non-symlink file`);
}

async function readStableRegularFile(filePath, name) {
  const absolute = path.resolve(filePath);
  const real = await fs.realpath(absolute).catch(() => null);
  if (real !== absolute) fail(`${name} path contains a symbolic link`);
  const beforeStat = await fs.lstat(absolute, {bigint: true}).catch(() => null);
  requireRegularFile(beforeStat, name);
  const before = statFingerprint(beforeStat);
  if (!fsConstants.O_NOFOLLOW) fail("stable input verification requires O_NOFOLLOW support");
  const handle = await fs.open(absolute, fsConstants.O_RDONLY | fsConstants.O_NOFOLLOW);
  let bytes;
  try {
    const openedStat = await handle.stat({bigint: true});
    requireRegularFile(openedStat, name);
    if (statFingerprint(openedStat) !== before) fail(`${name} changed while it was being read`);
    bytes = await handle.readFile();
  } finally {
    await handle.close();
  }
  const afterStat = await fs.lstat(absolute, {bigint: true}).catch(() => null);
  requireRegularFile(afterStat, name);
  if (statFingerprint(afterStat) !== before) fail(`${name} changed while it was being read`);
  return {bytes, sha256: sha256Hex(bytes)};
}

async function readJsonInput(filePath, name) {
  const absolute = path.resolve(filePath);
  const file = await readStableRegularFile(absolute, name);
  try {
    return {value: JSON.parse(file.bytes.toString("utf8")), reference: {path: absolute, sha256: file.sha256}};
  } catch (error) {
    fail(`${name} is not valid JSON: ${error.message}`);
  }
}

function digestReference(value, name) {
  exactKeys(value, ["path", "sha256"], name);
  if (!path.isAbsolute(value.path) || !HEX64.test(value.sha256 ?? "")) fail(`${name} is not immutable path+digest evidence`);
}

async function verifyArtifactReference(value, name, parseJson = false) {
  digestReference(value, name);
  const file = await readStableRegularFile(value.path, name);
  if (file.sha256 !== value.sha256) fail(`${name} identity moved`);
  if (!parseJson) return file;
  try {
    return {value: JSON.parse(file.bytes.toString("utf8")), reference: {path: value.path, sha256: value.sha256}};
  } catch (error) {
    fail(`${name} is not valid JSON: ${error.message}`);
  }
}

function revisionIdentity(value, name) {
  if (value === null || typeof value !== "object" || Array.isArray(value) || !HEX40.test(value.commit ?? "") || !HEX40.test(value.tree ?? "")) fail(`${name} is not a sealed source identity`);
}

function identityKey(row) {
  return `${row.surface_id}\0${row.source_local_id}\0${row.kind}`;
}

function validateIdentityRows(rows, fields, name) {
  if (!Array.isArray(rows) || rows.length === 0) fail(`${name} has no rows`);
  const identities = new Set();
  const surfaces = new Set();
  const sourceLocalIds = new Set();
  for (const row of rows) {
    exactKeys(row, fields, `${name} row`);
    for (const field of fields.slice(0, 3)) requireNonEmptyString(row[field], `${name} row ${field}`);
    const key = identityKey(row);
    if (identities.has(key) || surfaces.has(row.surface_id) || sourceLocalIds.has(row.source_local_id)) fail(`${name} has duplicate or ambiguous row ${row.surface_id}`);
    identities.add(key);
    surfaces.add(row.surface_id);
    sourceLocalIds.add(row.source_local_id);
  }
  return new Map(rows.map((row) => [row.surface_id, row]));
}

function requireExactIdentitySet(expected, actual, name) {
  const expectedKeys = [...expected.keys()].sort();
  const actualKeys = [...actual.keys()].sort();
  if (JSON.stringify(expectedKeys) !== JSON.stringify(actualKeys)) fail(`${name} has missing or extra rows`);
  for (const key of expectedKeys) if (JSON.stringify(expected.get(key)) !== JSON.stringify(actual.get(key))) fail(`${name} row identity changed for ${key}`);
}

function expectedDeferredRows() {
  return DEFERRED_EXPECTED.map(({sourceLocalId, kind, deferredOwner}) => ({surface_id: sourceLocalId, source_local_id: sourceLocalId, kind, deferred_owner: deferredOwner}));
}

function deferredTripleMap(rows, name) {
  const mapped = new Map();
  for (const row of rows) {
    const expected = DEFERRED_EXPECTED_BY_ID.get(row.source_local_id);
    if (!expected || row.surface_id !== row.source_local_id || row.kind !== expected.kind) fail(`${name} contains an unknown or wrong deferred identity`);
    if (mapped.has(identityKey(row))) fail(`${name} contains duplicate deferred identity`);
    mapped.set(identityKey(row), {surface_id: row.surface_id, source_local_id: row.source_local_id, kind: row.kind});
  }
  return mapped;
}

function validateCurrentDenominator(value) {
  exactKeys(value, DENOMINATOR_FIELDS, "current denominator");
  if (value.schema !== CURRENT_DENOMINATOR_SCHEMA || value.status !== "sealed") fail("current denominator is not sealed");
  const rows = validateIdentityRows(value.rows, CURRENT_DENOMINATOR_ROW_FIELDS, "current denominator");
  for (const row of value.rows) {
    if (!CANONICAL_SURFACE_ID.test(row.surface_id) || DEFERRED_EXPECTED_BY_ID.has(row.source_local_id) || row.kind === "framerail_xmlrpc_method" || row.kind === "wikidot_py_amc_module_shape") fail("current denominator contains deferred work");
    for (const field of ["actor", "input", "observable_interval", "result"]) {
      requireNonEmptyString(row[field], `current denominator row ${field}`);
    }
  }
  return rows;
}

function validateDeferredDenominator(value) {
  exactKeys(value, DENOMINATOR_FIELDS, "deferred denominator");
  if (value.schema !== DEFERRED_DENOMINATOR_SCHEMA || value.status !== "sealed") fail("deferred denominator is not sealed");
  const rows = validateIdentityRows(value.rows, DEFERRED_DENOMINATOR_ROW_FIELDS, "deferred denominator");
  if (value.rows.length !== DEFERRED_EXPECTED.length) fail("deferred denominator does not contain exactly 54 rows");
  const actual = deferredTripleMap(value.rows, "deferred denominator");
  const expected = new Map(expectedDeferredRows().map((row) => [identityKey(row), {surface_id: row.surface_id, source_local_id: row.source_local_id, kind: row.kind}]));
  requireExactIdentitySet(expected, actual, "deferred denominator");
  return rows;
}

function validateDeferredLedger(value) {
  exactKeys(value, DEFERRED_LEDGER_FIELDS, "deferred ledger");
  if (value.schema !== DEFERRED_LEDGER_SCHEMA || value.status !== "sealed") fail("deferred ledger is not sealed");
  const actual = new Map();
  if (!Array.isArray(value.rows) || value.rows.length !== DEFERRED_EXPECTED.length) fail("deferred ledger does not contain exactly 54 rows");
  for (const row of value.rows) {
    exactKeys(row, DEFERRED_LEDGER_ROW_FIELDS, "deferred ledger row");
    const expected = DEFERRED_EXPECTED_BY_ID.get(row.source_local_id);
    if (!expected || row.surface_id !== row.source_local_id || row.kind !== expected.kind || row.deferred_owner !== expected.deferredOwner || actual.has(identityKey(row))) fail("deferred ledger has an unknown, wrong, or duplicate row");
    actual.set(identityKey(row), {surface_id: row.surface_id, source_local_id: row.source_local_id, kind: row.kind, deferred_owner: row.deferred_owner});
  }
  requireExactIdentitySet(new Map(expectedDeferredRows().map((row) => [identityKey(row), row])), actual, "deferred ledger");
  return actual;
}

function completeLedger(ledger) {
  exactKeys(ledger, LEDGER_FIELDS, "canonical compatibility ledger");
  if (ledger.schema !== LEDGER_SCHEMA) fail("canonical compatibility ledger has unsupported schema");
  exactKeys(ledger.counts, ["raw_records", "public_inventory_records", "canonical_surfaces", "input_alias_edges", "deduplication_relationships"], "ledger counts");
  if (Object.values(ledger.counts).some((value) => !Number.isSafeInteger(value) || value < 0) || ledger.counts.raw_records !== ledger.raw_source_records.length || ledger.counts.canonical_surfaces !== ledger.surface_assignments.length || ledger.counts.canonical_surfaces !== ledger.rows.length || ledger.counts.input_alias_edges < ledger.counts.deduplication_relationships) fail("canonical compatibility ledger counts are incomplete");
  exactKeys(ledger.inputs, LEDGER_INPUT_FIELDS, "ledger inputs");
  digestReference(ledger.inputs.inventory, "ledger inventory");
  revisionIdentity(ledger.inputs.wikijump, "ledger Wikijump input");
  revisionIdentity(ledger.inputs.ftml, "ledger FTML input");
  for (const name of ["source_manifests", "raw_source_records", "source_local_identities", "surface_assignments", "rows"]) if (!Array.isArray(ledger[name]) || ledger[name].length === 0) fail(`canonical compatibility ledger has no ${name}`);
  if (!Array.isArray(ledger.relationships)) fail("canonical compatibility ledger has no relationships");
  for (const [rows, fields, name] of [
    [ledger.source_local_identities, ["source_local_id", "raw_record_id"], "source local identities"],
    [ledger.surface_assignments, ["assignment_id", "surface_id", "raw_record_id"], "surface assignments"],
  ]) for (const field of fields) {
    const values = rows.map((row) => row[field]);
    if (values.some((value) => typeof value !== "string" || value === "") || new Set(values).size !== values.length) fail(`${name} has missing or duplicate ${field}`);
  }
  const rowIds = new Set();
  for (const row of ledger.rows) {
    exactKeys(row, ROW_FIELDS, "compatibility ledger row");
    if (!CANONICAL_SURFACE_ID.test(row.surface_id) || rowIds.has(row.surface_id)) fail("compatibility ledger has missing or duplicate canonical row identity");
    rowIds.add(row.surface_id);
    for (const field of ["actor", "input", "observable_interval", "result", "source", "evidence", "tests", "owners", "issues", "blockers", "candidate", "standing", "closure"]) if (row[field] === null || typeof row[field] !== "object" || Array.isArray(row[field])) fail(`compatibility ledger row has no ${field}`);
    for (const field of ["candidate", "standing"]) {
      exactKeys(row[field], ["state", "artifacts"], `compatibility ledger row ${field}`);
      if (!Array.isArray(row[field].artifacts) || row[field].artifacts.length === 0 || row[field].state !== "pass") fail(`compatibility ledger row has incomplete ${field} proof`);
      for (const [index, artifact] of row[field].artifacts.entries()) digestReference(artifact, `ledger row ${row.surface_id} ${field} artifact ${index}`);
    }
  }
  exactKeys(ledger.deferred_exclusions, ["count", "by_kind", "by_owner", "records"], "ledger deferred exclusions");
  if (ledger.deferred_exclusions.count !== DEFERRED_EXPECTED.length || !Array.isArray(ledger.deferred_exclusions.records) || ledger.deferred_exclusions.records.length !== DEFERRED_EXPECTED.length) fail("ledger deferred exclusions are not exactly scoped");
  if (JSON.stringify(ledger.deferred_exclusions.by_kind) !== JSON.stringify({catalog_feature: 15, framerail_xmlrpc_method: 17, wikidot_py_amc_module_shape: 22}) || JSON.stringify(ledger.deferred_exclusions.by_owner) !== JSON.stringify({"external.wikidot-py": 22, "wikijump.xmlrpc-api": 32})) fail("ledger deferred exclusion counts are not exact");
  const actual = new Map();
  for (const record of ledger.deferred_exclusions.records) {
    exactKeys(record, ["source_local_id", "kind", "deferred_owner"], "ledger deferred exclusion record");
    const expected = DEFERRED_EXPECTED_BY_ID.get(record.source_local_id);
    if (!expected || record.kind !== expected.kind || record.deferred_owner !== expected.deferredOwner || actual.has(record.source_local_id)) fail("ledger deferred exclusions have an unknown, wrong, or duplicate identity");
    actual.set(record.source_local_id, record);
  }
  const currentIds = new Set(ledger.source_local_identities.map((row) => row.source_local_id));
  for (const sourceLocalId of DEFERRED_EXPECTED_BY_ID.keys()) if (currentIds.has(sourceLocalId)) fail(`canonical compatibility ledger contains deferred work: ${sourceLocalId}`);
  if (actual.size !== DEFERRED_EXPECTED.length) fail("ledger deferred exclusions are incomplete");
  return ledger;
}

function validateCurrentOpenIssues(value) {
  exactKeys(value, ["schema", "repository", "captured_at", "tracking_issue", "issues"], "current open issues");
  if (value.schema !== CURRENT_OPEN_ISSUES_SCHEMA) fail("current open issues has unsupported schema");
  if (value.repository !== "Rokurolize/wikijump") fail("current open issues is for the wrong repository");
  if (value.tracking_issue !== TRACKING_ISSUE) fail("current open issues has the wrong tracking issue");
  requireNonEmptyString(value.captured_at, "current open issues captured_at");
  if (!Array.isArray(value.issues)) fail("current open issues issues must be an array");
  const numbers = new Set();
  for (const issue of value.issues) {
    exactKeys(issue, ["number", "title", "url"], "current open issue");
    if (!Number.isSafeInteger(issue.number) || issue.number <= 0 || numbers.has(issue.number)) {
      fail("current open issues contains a missing or duplicate issue number");
    }
    numbers.add(issue.number);
    requireNonEmptyString(issue.title, `current open issue ${issue.number} title`);
    requireNonEmptyString(issue.url, `current open issue ${issue.number} url`);
  }
  if (!numbers.has(TRACKING_ISSUE)) fail("current open issues does not contain the tracking issue");
  return value.issues;
}

function validateStandingMatrix(value) {
  exactKeys(value, MATRIX_FIELDS, "standing matrix");
  if (value.schema !== STANDING_MATRIX_SCHEMA || value.status !== "pass" || typeof value.run_id !== "string" || value.run_id === "" || !HEX40.test(value.merge_commit ?? "") || !HEX40.test(value.merge_tree ?? "") || !HEX40.test(value.ftml_sha ?? "") || !HEX40.test(value.ftml_tree ?? "") || !HEX40.test(value.candidate_commit ?? "") || !HEX64.test(value.candidate_artifact_key ?? "")) fail("standing matrix is not a sealed passing matrix");
  digestReference(value.promotion_precondition, "standing promotion precondition");
  digestReference(value.standing_refresh, "standing refresh receipt");
  const rows = validateIdentityRows(value.rows, ["surface_id", "source_local_id", "kind", "status", "artifacts"], "standing matrix");
  for (const row of value.rows) {
    if (!CANONICAL_SURFACE_ID.test(row.surface_id) || row.status !== "pass" || !Array.isArray(row.artifacts) || row.artifacts.length === 0) fail(`standing matrix row ${row.surface_id} is incomplete`);
    for (const [index, artifact] of row.artifacts.entries()) digestReference(artifact, `standing matrix row ${row.surface_id} artifact ${index}`);
  }
  return rows;
}

async function requireRepository(repositoryPath) {
  const absolute = path.resolve(repositoryPath);
  const real = await fs.realpath(absolute).catch(() => null);
  const stat = await fs.lstat(absolute).catch(() => null);
  if (!stat?.isDirectory() || stat.isSymbolicLink() || real !== absolute) fail("repository must be a non-symlink directory");
  return absolute;
}

async function git(repositoryPath, ...arguments_) {
  try {
    const {stdout} = await execFile("git", ["-C", repositoryPath, ...arguments_], {encoding: "utf8", maxBuffer: 1024 * 1024});
    return stdout.trim();
  } catch (error) {
    fail(`repository Git identity could not be verified: ${error?.message ?? String(error)}`);
  }
}

async function verifyCandidateRuntimeDelta(repositoryPath, candidateCommit, mergeCommit) {
  if (candidateCommit === mergeCommit) fail("candidate commit must differ from merge commit");
  const changed = (await git(repositoryPath, "diff", "--name-only", `${candidateCommit}..${mergeCommit}`)).split("\n").filter(Boolean);
  const allowed = [".github/", "docs/development/candidate-case-set-manifest.json", "install/local/wikidot-verification/", "install/standing/"];
  if (changed.some((file) => !allowed.some((prefix) => file.startsWith(prefix)))) fail("merged source changed runtime inputs after candidate proof");
}

async function verifyRepositoryMerge(repositoryPath, mergeCommit, mergeTree, candidateCommit) {
  const [head, tree, parentsLine] = await Promise.all([
    git(repositoryPath, "rev-parse", "HEAD^{commit}"),
    git(repositoryPath, "rev-parse", "HEAD^{tree}"),
    git(repositoryPath, "rev-list", "--parents", "-n", "1", "HEAD"),
  ]);
  if (head !== mergeCommit) fail("repository HEAD is not the standing merge commit");
  if (tree !== mergeTree) fail("repository HEAD tree does not match the standing merge tree");
  const parents = parentsLine.split(/\s+/u);
  if (parents.length !== 3 || parents[0] !== mergeCommit) fail("repository HEAD is not a normal two-parent merge commit");
  if (candidateCommit === mergeCommit) fail("candidate commit must differ from merge commit");
  const candidateMergeBase = await git(repositoryPath, "merge-base", candidateCommit, mergeCommit);
  if (candidateMergeBase !== candidateCommit) fail("repository merge does not include the candidate PR head ancestor");
  await verifyCandidateRuntimeDelta(repositoryPath, candidateCommit, mergeCommit);
}

async function verifyLedgerArtifacts(ledger) {
  await verifyArtifactReference(ledger.inputs.inventory, "ledger inventory");
  await Promise.all(ledger.rows.flatMap((row) => [
    ...row.candidate.artifacts.map((artifact, index) => verifyArtifactReference(artifact, `ledger row ${row.surface_id} candidate artifact ${index}`)),
    ...row.standing.artifacts.map((artifact, index) => verifyArtifactReference(artifact, `ledger row ${row.surface_id} standing artifact ${index}`)),
  ]));
}

async function verifyMatrixArtifacts(matrix) {
  await Promise.all(matrix.rows.flatMap((row) => row.artifacts.map((artifact, index) => verifyArtifactReference(artifact, `standing matrix row ${row.surface_id} artifact ${index}`))));
}

function reconcileRows(ledger, denominatorRows, matrixRows) {
  const denominatorIdentities = new Map([...denominatorRows].map(([surfaceId, row]) => [surfaceId, {surface_id: row.surface_id, source_local_id: row.source_local_id, kind: row.kind}]));
  const ledgerRows = new Map(ledger.rows.map((row) => [row.surface_id, {surface_id: row.surface_id, source_local_id: denominatorRows.get(row.surface_id)?.source_local_id, kind: denominatorRows.get(row.surface_id)?.kind}]));
  const matrixIdentities = new Map([...matrixRows].map(([surfaceId, row]) => [surfaceId, {surface_id: row.surface_id, source_local_id: row.source_local_id, kind: row.kind}]));
  requireExactIdentitySet(denominatorIdentities, ledgerRows, "ledger and current denominator");
  requireExactIdentitySet(denominatorIdentities, matrixIdentities, "standing matrix and current denominator");
  const sourceByRaw = new Map(ledger.source_local_identities.map((row) => [row.raw_record_id, row.source_local_id]));
  const assignmentBySurface = new Map(ledger.surface_assignments.map((row) => [row.surface_id, row]));
  const ledgerBySurface = new Map(ledger.rows.map((row) => [row.surface_id, row]));
  for (const [surfaceId, denominatorRow] of denominatorRows) {
    const assignment = assignmentBySurface.get(surfaceId);
    const sourceLocalId = sourceByRaw.get(assignment?.raw_record_id);
    if (sourceLocalId !== denominatorRow.source_local_id) fail(`current denominator source identity is not bound for ${surfaceId}`);
    const matrixRow = matrixRows.get(surfaceId);
    if (matrixRow.source_local_id !== denominatorRow.source_local_id || matrixRow.kind !== denominatorRow.kind) fail(`standing matrix row identity is not bound for ${surfaceId}`);
    const ledgerRow = ledgerBySurface.get(surfaceId);
    for (const field of ["actor", "input", "observable_interval", "result"]) {
      if (ledgerRow?.[field]?.state !== "known" || ledgerRow[field].value !== denominatorRow[field]) {
        fail(`compatibility ledger semantic value is missing or drifted for ${surfaceId} ${field}`);
      }
    }
  }
}

async function verifyPromotion(matrix, ledger) {
  const input = await verifyArtifactReference(matrix.promotion_precondition, "standing promotion precondition", true);
  const promotion = validateStandingPromotionPrecondition(input.value);
  for (const [actual, expected, name] of [
    [promotion.run_id, matrix.run_id, "promotion run id"],
    [matrix.merge_commit, ledger.inputs.wikijump.commit, "matrix merge commit"],
    [matrix.merge_tree, ledger.inputs.wikijump.tree, "matrix merge tree"],
    [matrix.ftml_sha, ledger.inputs.ftml.commit, "matrix FTML commit"],
    [matrix.ftml_tree, ledger.inputs.ftml.tree, "matrix FTML tree"],
    [promotion.candidate.wikijump_commit, matrix.candidate_commit, "promotion candidate PR head"],
    [promotion.candidate.ftml_sha, matrix.ftml_sha, "promotion candidate FTML commit"],
    [promotion.build.wikijump_commit, matrix.candidate_commit, "promotion build candidate PR head"],
    [promotion.build.ftml_sha, ledger.inputs.ftml.commit, "promotion build FTML commit"],
    [promotion.candidate.artifact_key, matrix.candidate_artifact_key, "candidate artifact_key"],
  ]) if (actual !== expected) fail(`${name} does not match the canonical source`);
  return promotion;
}

async function verifyStandingRefresh(matrix, promotion) {
  const input = await verifyArtifactReference(matrix.standing_refresh, "standing refresh receipt", true);
  const refresh = validateStandingRefreshReceipt(input.value);
  for (const [actual, expected, name] of [
    [refresh.run_id, matrix.run_id, "standing refresh run id"],
    [refresh.wikijump_sha, matrix.merge_commit, "standing refresh merge commit"],
    [refresh.wikijump_tree, matrix.merge_tree, "standing refresh merge tree"],
    [refresh.ftml_sha, matrix.ftml_sha, "standing refresh FTML commit"],
    [refresh.promotion_precondition.path, matrix.promotion_precondition.path, "standing refresh promotion precondition path"],
    [refresh.promotion_precondition.sha256, matrix.promotion_precondition.sha256, "standing refresh promotion precondition digest"],
    [refresh.runtime_differential_identity.identity.wikijump_sha, matrix.merge_commit, "standing refresh runtime merge commit"],
    [refresh.runtime_differential_identity.identity.ftml_sha, matrix.ftml_sha, "standing refresh runtime FTML commit"],
    [refresh.runtime_differential_identity.identity.dependency_lock_sha256, refresh.dependency_lock_sha256, "standing refresh runtime dependency lock"],
  ]) if (actual !== expected) fail(`${name} does not match the canonical source`);
  if (promotion.candidate.wikijump_commit === refresh.wikijump_sha) fail("standing refresh merge source is the candidate PR head");
  const [preparedInput, runtimeInput] = await Promise.all([
    verifyArtifactReference(refresh.prepared_receipt, "standing refresh prepared receipt", true),
    verifyArtifactReference({path: refresh.runtime_differential_identity.path, sha256: refresh.runtime_differential_identity.sha256}, "standing refresh runtime identity", true),
  ]);
  if (JSON.stringify(runtimeInput.value) !== JSON.stringify(refresh.runtime_differential_identity.identity)) fail("standing refresh runtime identity digest does not match its embedded identity");
  const prepared = preparedInput.value;
  if (prepared.schema_version !== 1 || prepared.kind !== "standing-image-preparation" || prepared.status !== "pass" || prepared.run_id !== refresh.run_id || prepared.wikijump_sha !== refresh.wikijump_sha || prepared.wikijump_tree !== refresh.wikijump_tree || prepared.ftml_sha !== refresh.ftml_sha || prepared.dependency_lock_sha256 !== refresh.dependency_lock_sha256) fail("standing refresh prepared receipt is stale");
  if (JSON.stringify(prepared.promotion_precondition) !== JSON.stringify(refresh.promotion_precondition)) fail("standing refresh prepared receipt is not bound to its promotion precondition");
  for (const service of ["deepwell", "framerail", "wws"]) {
    if (prepared.images?.[service]?.reference !== refresh.images[service].id || prepared.images?.[service]?.id !== refresh.images[service].id || promotion.build.images[service] !== refresh.images[service].id) fail(`standing refresh ${service} image is not bound to the candidate build`);
  }
  return refresh;
}

async function verifyConsolidatedValidation(input, matrix, repositoryPath) {
  const value = input.value;
  try {
    validateConsolidatedValidation(value, {wikijump_commit: matrix.merge_commit, wikijump_tree: matrix.merge_tree});
  } catch (error) {
    fail(`consolidated validation receipt is not passing for the merge source: ${error.message}`);
  }
  if (Number.isNaN(Date.parse(value.completed_at))) fail("consolidated validation receipt has invalid completion time");
  exactKeys(value.preflight.source_sha256, CONSOLIDATED_VALIDATION_SOURCE_FILES, "consolidated validation source hashes");
  const {stdout: parentStdout} = await execFile("git", ["-C", repositoryPath, "rev-parse", `${matrix.merge_commit}^1`], {encoding: "utf8"});
  if (parentStdout.trim() !== value.base_commit) fail("consolidated validation base is not the merge first parent");
  for (const relative of CONSOLIDATED_VALIDATION_SOURCE_FILES) {
    const {stdout} = await execFile("git", ["-C", repositoryPath, "show", `${matrix.merge_commit}:${relative}`], {encoding: null, maxBuffer: 16 * 1024 * 1024});
    if (sha256Hex(stdout) !== value.preflight.source_sha256[relative]) fail(`consolidated validation source hash drifted for ${relative}`);
  }
}

function finalZeroCounts(ledger, finalFrozen, openIssues) {
  const rows = ledger.rows;
  const count = (predicate) => rows.reduce((total, row) => total + (predicate(row) ? 1 : 0), 0);
  const ownedIssueNumbers = new Set(rows.flatMap((row) => row.issues.numbers ?? []));
  const openProductIssues = openIssues.filter(({number}) => number !== TRACKING_ISSUE);
  const unownedOpenProductIssues = openProductIssues.filter(({number}) => !ownedIssueNumbers.has(number));
  const counts = {
    complete_product_rows_open_or_unreconciled:
      count((row) => row.closure.state !== "closed" || row.issues.state !== "present" || row.blockers.state !== "none") +
      openProductIssues.length,
    duplicate_or_ambiguous_canonical_identities: 0,
    missing_or_failing_final_validation: finalFrozen?.receipt?.validation ? 0 : rows.length,
    missing_or_failing_candidate_proofs: count((row) => row.candidate.state !== "pass"),
    missing_or_failing_standing_proofs: count((row) => row.standing.state !== "pass"),
    missing_or_stale_source_provenance: count((row) => row.source.state !== "present"),
    missing_public_surfaces: count((row) => typeof row.surface_id !== "string" || row.surface_id === ""),
    unimplemented_source_required_rows: count((row) => row.source.state !== "present"),
    unknown_owners_or_untyped_edges: count((row) => row.owners.state !== "present"),
    unrepresented_charter_requirements:
      count((row) => row.issues.state !== "present" || row.tests.state !== "present") +
      unownedOpenProductIssues.length,
    unresolved_wikidot_evidence_requirements: count((row) => row.evidence.state !== "present"),
  };
  if (JSON.stringify(Object.keys(counts).sort()) !== JSON.stringify([...FINAL_ZERO_CLASSES].sort())) fail("final-zero count classes do not match the campaign contract");
  return counts;
}

export function parseArgs(argv) {
  const names = new Set(["ledger", "denominator", "deferred-denominator", "deferred-ledger", "standing-matrix", "final-frozen", "consolidated-validation", "open-issues", "repository", "output"]);
  const args = {};
  for (let index = 0; index < argv.length; index += 2) {
    const flag = argv[index];
    if (flag === "--help" || flag === "-h") return {help: true};
    const name = flag?.startsWith("--") ? flag.slice(2) : "";
    const value = argv[index + 1];
    if (!names.has(name) || Object.hasOwn(args, name) || !value || value.startsWith("--")) fail(`unknown or duplicate option: ${flag}`);
    args[name] = path.resolve(value);
  }
  for (const name of names) if (!args[name]) fail(`--${name} is required`);
  return args;
}

export function usage() {
  return "Usage: verify-final-zero.mjs --ledger FILE --denominator FILE --deferred-denominator FILE --deferred-ledger FILE --standing-matrix FILE --final-frozen FILE --consolidated-validation FILE --open-issues FILE --repository DIRECTORY --output FILE";
}

export async function verifyFinalZero({ledger, denominator, deferredDenominator, deferredLedger, standingMatrix, finalFrozen, consolidatedValidation, openIssues, repository}) {
  const repositoryPath = await requireRepository(repository);
  const [ledgerInput, denominatorInput, deferredDenominatorInput, deferredLedgerInput, standingInput, consolidatedValidationInput, openIssuesInput] = await Promise.all([
    readJsonInput(ledger, "canonical compatibility ledger"),
    readJsonInput(denominator, "current denominator"),
    readJsonInput(deferredDenominator, "deferred denominator"),
    readJsonInput(deferredLedger, "deferred ledger"),
    readJsonInput(standingMatrix, "standing compatibility matrix"),
    readJsonInput(consolidatedValidation, "consolidated validation receipt"),
    readJsonInput(openIssues, "current open issues"),
  ]);
  const ledgerValue = completeLedger(ledgerInput.value);
  const denominatorRows = validateCurrentDenominator(denominatorInput.value);
  validateDeferredDenominator(deferredDenominatorInput.value);
  validateDeferredLedger(deferredLedgerInput.value);
  const matrixRows = validateStandingMatrix(standingInput.value);
  const currentOpenIssues = validateCurrentOpenIssues(openIssuesInput.value);
  const promotion = await verifyPromotion(standingInput.value, ledgerValue);
  const frozen = await verifyFinalFrozenReceipt({
    receiptPath: path.resolve(finalFrozen),
    source: {
      wikijump_commit: standingInput.value.candidate_commit,
      wikijump_tree: promotion.candidate.wikijump_tree,
      ftml_sha: standingInput.value.ftml_sha,
    },
  });
  await verifyLedgerArtifacts(ledgerValue);
  await verifyMatrixArtifacts(standingInput.value);
  reconcileRows(ledgerValue, denominatorRows, matrixRows);
  await verifyStandingRefresh(standingInput.value, promotion);
  await verifyRepositoryMerge(repositoryPath, standingInput.value.merge_commit, standingInput.value.merge_tree, standingInput.value.candidate_commit);
  await verifyConsolidatedValidation(consolidatedValidationInput, standingInput.value, repositoryPath);
  const counts = finalZeroCounts(ledgerValue, frozen, currentOpenIssues);
  const nonzero = Object.entries(counts).filter(([, value]) => value !== 0);
  if (nonzero.length > 0) fail(`final-zero check failed: ${nonzero.map(([name, value]) => `${name}=${value}`).join(", ")}`);
  return {
    schema: "wikijump.compatibility_final_zero_receipt.v1",
    status: "pass",
    merge_commit: standingInput.value.merge_commit,
    counts,
    inputs: {
      ledger: ledgerInput.reference,
      denominator: denominatorInput.reference,
      deferred_denominator: deferredDenominatorInput.reference,
      deferred_ledger: deferredLedgerInput.reference,
      standing_matrix: standingInput.reference,
      standing_refresh: standingInput.value.standing_refresh,
      final_frozen: {path: frozen.path, sha256: frozen.sha256},
      consolidated_validation: consolidatedValidationInput.reference,
      open_issues: openIssuesInput.reference,
      repository: repositoryPath,
    },
  };
}

export async function main(argv, {stdout = console.log} = {}) {
  const args = parseArgs(argv);
  if (args.help) {
    stdout(usage());
    return 0;
  }
  const receipt = await verifyFinalZero({ledger: args.ledger, denominator: args.denominator, deferredDenominator: args["deferred-denominator"], deferredLedger: args["deferred-ledger"], standingMatrix: args["standing-matrix"], finalFrozen: args["final-frozen"], consolidatedValidation: args["consolidated-validation"], openIssues: args["open-issues"], repository: args.repository});
  const sealed = await sealJsonNoReplace(args.output, receipt);
  stdout(JSON.stringify({schema: receipt.schema, status: receipt.status, output: sealed.path, sha256: sealed.sha256}));
  return 0;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error?.stack ?? error?.message ?? String(error));
    return 1;
  },
});
