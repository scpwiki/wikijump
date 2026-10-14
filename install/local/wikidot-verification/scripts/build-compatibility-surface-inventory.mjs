#!/usr/bin/env node

import fs from "node:fs/promises"
import { constants as fsConstants } from "node:fs"
import { createHash } from "node:crypto"
import { execFileSync, spawnSync } from "node:child_process"
import path from "node:path"
import process from "node:process"
import { fileURLToPath } from "node:url"
import { discoverFramerailRouteDescriptors } from "../src/compatibility-inventory/framerail-routes.mjs"
import { discoverOpen43AuditCases as discoverOpen43AuditCasesFromSource } from "../src/compatibility-inventory/open43-audits.mjs"
import {
  applySiteChangesEvidence as applySiteChangesEvidenceFromSource,
  SITE_CHANGES_EVIDENCE_ARTIFACT
} from "../src/compatibility-inventory/site-changes-evidence.mjs"
import { discoverWwsRouteRecords } from "../src/compatibility-inventory/wws-route-parser.mjs"
import {
  CANONICAL_IMPLEMENTATION_LEDGER,
  LEDGER_STATUSES,
  discoverCatalogFeatures,
  phase,
  surface,
  uniqueSortedStrings
} from "../src/compatibility-inventory/catalog-surfaces.mjs"
import { discoverFtmlRawSurfaceManifest } from "../src/compatibility-inventory/ftml-raw-surface-manifest.mjs"
import { createPinnedSourceAccess } from "../src/compatibility-inventory/pinned-source.mjs"
import {
  CATALOG_SOURCE_ATTRIBUTION,
  applyCatalogSourceAttribution as applyCatalogSourceAttributionFromEvidence,
  applyFramerailRouteActionEvidence as applyFramerailRouteActionEvidenceFromEvidence,
  applyWwsContractEvidence as applyWwsContractEvidenceFromEvidence
} from "../src/compatibility-inventory/evidence-projection.mjs"
import {
  applyFramerailAmcTests as applyFramerailAmcTestsFromLegacy,
  discoverFramerailAmc as discoverFramerailAmcFromLegacy,
  discoverFramerailXmlRpc as discoverFramerailXmlRpcFromLegacy,
  discoverWikidotPyAmc as discoverWikidotPyAmcFromLegacy
} from "../src/compatibility-inventory/legacy-surfaces.mjs"
import { discoverDeepwellJsonRpc, discoverPageActionSurfaces } from "../src/compatibility-inventory/deepwell-surfaces.mjs"
import {
  PHASE_STATUSES,
  SEMANTICS_REGISTRY,
  applyAuditedIssueOwnership,
  applyFtmlCatalogSourceProjection,
  assertCanonicalStrings,
  buildRelationshipModel,
  normalizeSurfaceOwners,
  validateInventory
} from "../src/compatibility-inventory/inventory-model.mjs"

import { CANDIDATE_CASE_SETS } from "../src/candidate-case-command.mjs"
import { siblingCheckout } from "../src/sibling-checkout.mjs"

const SCHEMA = "wikijump.compatibility_surface_inventory.v3"
const SCRIPT_DIRECTORY = path.dirname(fileURLToPath(import.meta.url))
const DEFAULT_ROOT = path.resolve(SCRIPT_DIRECTORY, "../../../..")
const DEFAULT_OUTPUT = "docs/development/compatibility-surface-inventory.json"
const WIKIDOT_PY_GIT_DIR = path.join(
  siblingCheckout("wikidot.py", "WIKIDOT_PY_CHECKOUT", DEFAULT_ROOT),
  ".git"
)
const FTML_GIT_DIR = path.join(siblingCheckout("ftml", "WIKIJUMP_FTML_CHECKOUT", DEFAULT_ROOT), ".git")
const GIT_EXECUTABLE = "/usr/bin/git"
const GIT_ENVIRONMENT = Object.freeze({
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_NO_LAZY_FETCH: "1",
  GIT_NO_REPLACE_OBJECTS: "1",
  GIT_OPTIONAL_LOCKS: "0",
  GIT_PAGER: "cat",
  GIT_TERMINAL_PROMPT: "0",
  LANG: "C",
  LC_ALL: "C",
  PATH: "/usr/bin:/bin"
})
const WIKIDOT_PY_AMC_MODULE_EXCLUSIONS = new Set(["edit/PageEditModule"])
const SOURCE_INPUTS = new Map()
const AUDITED_OWNERSHIP_ROOT =
  "install/local/wikidot-verification/fixtures/compatibility-ownership"
const AUDITED_OWNERSHIP_REPORTS = Object.freeze([
  {
    path: `${AUDITED_OWNERSHIP_ROOT}/catalog-feature-owners.json`,
    sha256: "63537ec48261f0bb956407e7fa2889a2f33548b596d441191632319907f0f855"
  },
  {
    path: `${AUDITED_OWNERSHIP_ROOT}/deepwell-jsonrpc.json`,
    sha256: "9a55d5a726dd6696639cb1686da11440bf75bc211fbb7b48bac5575e3df4074c"
  },
  {
    path: `${AUDITED_OWNERSHIP_ROOT}/framerail-actions.json`,
    sha256: "d3b461c03d931c3397fcd345b0400fb07e8fbe8621f48cfd80dfd7f6b0ec126b"
  },
  {
    path: `${AUDITED_OWNERSHIP_ROOT}/wws-wikidot-py.json`,
    sha256: "ef87c37c9bd2ebf661d003c361f386c5d979b30aeebb18a6b44c307124f0636c"
  }
])
// Audited ownership attaches to the catalog's feature identity set. Other
// catalog metadata, such as live observation bindings, does not alter owners.
const AUDITED_CATALOG_SURFACE_IDS_SHA256 = "230b734b715f15dc5b82e94f224bb8ae4a2717696d6bf23e9e57a103b83e1e28"
async function loadSupportedWikidotPySource(root) {
  const value = await readJson(root, "docs/development/wikidot-py-supported-source.json")
  if (value.schema !== "wikijump.wikidot_py_supported_source.v1" ||
      typeof value.repository !== "string" ||
      !/^[0-9a-f]{40}$/u.test(value.commit ?? "") ||
      !/^[0-9a-f]{40}$/u.test(value.root_tree ?? "") ||
      !Array.isArray(value.objects)) {
    throw new Error("wikidot.py supported source lock is invalid")
  }
  const objects = new Map()
  for (const object of value.objects) {
    if (!object || typeof object.path !== "string" || !["blob", "tree"].includes(object.type) ||
        !/^[0-9a-f]{40}$/u.test(object.oid ?? "") || objects.has(object.path)) {
      throw new Error("wikidot.py supported source lock object identity is invalid")
    }
    objects.set(object.path, [object.type, object.oid])
  }
  return { repository: value.repository, commit: value.commit, root_tree: value.root_tree, objects }
}

function pinnedWikidotPyAmcModules(wikidotPySource) {
  const result = spawnSync(
    GIT_EXECUTABLE,
    [
      "--no-replace-objects",
      `--git-dir=${WIKIDOT_PY_GIT_DIR}`,
      "grep",
      "-h",
      "-o",
      "-E",
      '"[A-Za-z0-9_/-]+Module"',
      wikidotPySource.commit,
      "--",
      "src/wikidot/module"
    ],
    {
      encoding: "utf8",
      env: GIT_ENVIRONMENT
    }
  )
  if (result.status !== 0) {
    throw new Error(`cannot read pinned wikidot.py modules: ${result.stderr.trim()}`)
  }
  const discovered = new Set(
    result.stdout
      .trim()
      .split("\n")
      .map((value) => value.slice(1, -1))
      .filter((value) => value.includes("/"))
  )
  for (const excluded of WIKIDOT_PY_AMC_MODULE_EXCLUSIONS) {
    if (!discovered.delete(excluded)) {
      throw new Error(`pinned wikidot.py no longer declares excluded AMC module: ${excluded}`)
    }
  }
  return [...discovered].sort()
}

function verifyPinnedWikidotPySource(wikidotPySource) {
  const identities = [
    [`${wikidotPySource.commit}^{tree}`, wikidotPySource.root_tree],
    ...[...wikidotPySource.objects].map(([objectPath, [, oid]]) => [
      `${wikidotPySource.commit}:${objectPath}`,
      oid
    ])
  ]
  for (const [revision, expected] of identities) {
    let actual
    try {
      actual = execFileSync(
        GIT_EXECUTABLE,
        ["--no-replace-objects", `--git-dir=${WIKIDOT_PY_GIT_DIR}`, "rev-parse", "--verify", revision],
        { encoding: "utf8", env: GIT_ENVIRONMENT, stdio: ["ignore", "pipe", "ignore"] }
      ).trim()
    } catch {
      throw new Error(`cannot resolve pinned wikidot.py source object: ${revision}`)
    }
    if (actual !== expected) {
      throw new Error(`pinned wikidot.py source object drift: ${revision}`)
    }
  }
}

function usage() {
  return `Usage: node ${path.basename(process.argv[1])} [--root REPOSITORY] [--output JSON] [--source-revision COMMIT]\n`
}

function parseArgs(argv) {
  let root = DEFAULT_ROOT
  let output
  let sourceRevision
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index]
    if (argument === "--root") {
      root = path.resolve(requireValue(argv, ++index, "--root"))
    } else if (argument === "--output") {
      output = path.resolve(requireValue(argv, ++index, "--output"))
    } else if (argument === "--source-revision") {
      sourceRevision = requireValue(argv, ++index, "--source-revision")
    } else if (argument === "--help" || argument === "-h") {
      process.stdout.write(usage())
      process.exit(0)
    } else {
      throw new Error(`unknown argument: ${argument}`)
    }
  }
  const resolvedOutput = output ?? path.join(root, DEFAULT_OUTPUT)
  assertRepositoryPath(root, resolvedOutput)
  return {
    root,
    output: resolvedOutput,
    sourceRevision
  }
}

function requireValue(argv, index, option) {
  const value = argv[index]
  if (!value || value.startsWith("--")) throw new Error(`${option} requires a value`)
  return value
}

function toPosix(value) {
  return value.split(path.sep).join("/")
}

function relativeReference(root, absolutePath) {
  const relative = path.relative(root, absolutePath)
  if (relative === "" || relative.startsWith("..") || path.isAbsolute(relative)) {
    throw new Error(`public reference is outside the repository: ${absolutePath}`)
  }
  return toPosix(relative)
}

function assertRepositoryPath(root, absolutePath) {
  const relative = path.relative(root, absolutePath)
  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    throw new Error(`public reference is outside the repository: ${absolutePath}`)
  }
}

async function readJson(root, relativePath) {
  const absolutePath = repositoryPath(root, relativePath)
  let source
  try {
    source = await fs.readFile(absolutePath, "utf8")
  } catch (error) {
    throw new Error(`cannot read ${relativePath}: ${error.message}`)
  }
  SOURCE_INPUTS.set(toPosix(relativePath), source)
  try {
    return JSON.parse(source)
  } catch (error) {
    throw new Error(`invalid JSON in ${relativePath}: ${error.message}`)
  }
}

async function readText(root, relativePath) {
  try {
    const source = await fs.readFile(repositoryPath(root, relativePath), "utf8")
    SOURCE_INPUTS.set(toPosix(relativePath), source)
    return source
  } catch (error) {
    throw new Error(`cannot read ${relativePath}: ${error.message}`)
  }
}

async function verifyAuditedOwnershipReports(root) {
  for (const report of AUDITED_OWNERSHIP_REPORTS) {
    const source = await readText(root, report.path)
    if (sha256(source) !== report.sha256) {
      throw new Error(`audited ownership fixture digest mismatch: ${report.path}`)
    }
  }
}

function repositoryPath(root, relativePath) {
  const absolutePath = path.resolve(root, relativePath)
  assertRepositoryPath(root, absolutePath)
  return absolutePath
}

async function writeRepositoryOutput(root, output, contents) {
  await fs.mkdir(path.dirname(output), { recursive: true })
  const rootRealPath = await fs.realpath(root)
  const parentRealPath = await fs.realpath(path.dirname(output))
  assertRepositoryPath(rootRealPath, parentRealPath)
  try {
    if ((await fs.lstat(output)).isSymbolicLink()) {
      throw new Error(`output must not be a symbolic link: ${output}`)
    }
  } catch (error) {
    if (error.code !== "ENOENT") throw error
  }
  const handle = await fs.open(
    output,
    fsConstants.O_WRONLY | fsConstants.O_CREAT | fsConstants.O_TRUNC | fsConstants.O_NOFOLLOW
  )
  try {
    await handle.writeFile(contents)
  } finally {
    await handle.close()
  }
}

async function readAbsoluteText(root, absolutePath) {
  return readText(root, relativeReference(root, absolutePath))
}

const pinnedSource = createPinnedSourceAccess({
  sourceInputs: SOURCE_INPUTS,
  readText,
  repositoryPath,
  gitExecutable: GIT_EXECUTABLE,
  gitEnvironment: GIT_ENVIRONMENT,
  ftmlGitDir: FTML_GIT_DIR
})
const {
  resolveGitObject,
  sourceProvenance,
  readGitSpecBatch,
  verifyRegistryBlobs,
  preloadPinnedRevisionTexts,
  gitRevisionContains
} = pinnedSource

function assertExactKeys(value, expected, context) {
  const actual = value && typeof value === "object" && !Array.isArray(value)
    ? Object.keys(value).sort()
    : []
  if (JSON.stringify(actual) !== JSON.stringify([...expected].sort())) {
    throw new Error(`${context} has missing or extra keys`)
  }
}

function validateSemanticsRegistry(semantics) {
  assertExactKeys(semantics, [
    "schema",
    "ftml",
    "relationship_edge_types",
    "specification_owner_by_kind",
    "implementation_owners_by_legacy_owner",
    "specification_owner_keys",
    "implementation_owner_keys"
  ], SEMANTICS_REGISTRY)
  if (semantics.schema !== "wikijump.compatibility_surface_semantics.v1") {
    throw new Error(`${SEMANTICS_REGISTRY} has unknown schema`)
  }
  assertExactKeys(semantics.ftml, [
    "counts", "raw_surface_identities", "catalog_crosswalk"
  ], `${SEMANTICS_REGISTRY} ftml`)
  assertExactKeys(semantics.ftml.counts, [
    "lexer_rules", "parser_functions", "canonical_blocks", "block_aliases",
    "typed_modules", "ast_variants", "delayed_forms", "generated_runtime_kinds",
    "renderer_modules", "wikidot_fixtures", "total"
  ], `${SEMANTICS_REGISTRY} FTML counts`)
  if (!Array.isArray(semantics.ftml.raw_surface_identities)) {
    throw new Error(`${SEMANTICS_REGISTRY} has no FTML identities`)
  }
  for (const identity of semantics.ftml.raw_surface_identities) {
    assertExactKeys(identity, ["surface_id", "kind"], `${SEMANTICS_REGISTRY} FTML identity`)
  }
  const sortedIdentities = [...semantics.ftml.raw_surface_identities].sort((left, right) =>
    left.surface_id.localeCompare(right.surface_id, "en")
  )
  if (
    new Set(semantics.ftml.raw_surface_identities.map(({ surface_id: id }) => id)).size !==
      semantics.ftml.raw_surface_identities.length ||
    JSON.stringify(sortedIdentities) !== JSON.stringify(semantics.ftml.raw_surface_identities)
  ) {
    throw new Error(`${SEMANTICS_REGISTRY} FTML identities must be sorted and unique`)
  }
  if (!Array.isArray(semantics.ftml.catalog_crosswalk)) {
    throw new Error(`${SEMANTICS_REGISTRY} has no FTML crosswalk`)
  }
  for (const row of semantics.ftml.catalog_crosswalk) {
    assertExactKeys(row, [
      "feature_id", "parsed_by", "rendered_by", "tested_by", "ftml_surfaces", "runtime_owner"
    ], `${SEMANTICS_REGISTRY} FTML crosswalk row`)
    for (const field of ["parsed_by", "rendered_by", "tested_by", "ftml_surfaces"]) {
      assertCanonicalStrings(row[field], `${SEMANTICS_REGISTRY} ${row.feature_id} ${field}`)
    }
  }
  const crosswalkIds = semantics.ftml.catalog_crosswalk.map(({ feature_id: id }) => id)
  if (JSON.stringify(crosswalkIds) !== JSON.stringify(uniqueSortedStrings(crosswalkIds))) {
    throw new Error(`${SEMANTICS_REGISTRY} FTML crosswalk rows must be sorted and unique`)
  }
  assertCanonicalStrings(semantics.relationship_edge_types, `${SEMANTICS_REGISTRY} edge types`)
  assertCanonicalStrings(semantics.specification_owner_keys, `${SEMANTICS_REGISTRY} specification owners`)
  assertCanonicalStrings(semantics.implementation_owner_keys, `${SEMANTICS_REGISTRY} implementation owners`)
  if (
    !semantics.specification_owner_by_kind ||
    typeof semantics.specification_owner_by_kind !== "object" ||
    Array.isArray(semantics.specification_owner_by_kind) ||
    !semantics.implementation_owners_by_legacy_owner ||
    typeof semantics.implementation_owners_by_legacy_owner !== "object" ||
    Array.isArray(semantics.implementation_owners_by_legacy_owner)
  ) {
    throw new Error(`${SEMANTICS_REGISTRY} owner maps must be objects`)
  }
}

function isCanonicalRepositoryReference(reference) {
  if (typeof reference !== "string" || reference === "" || reference.trim() !== reference) {
    return false
  }
  const fragmentIndex = reference.indexOf("#")
  const referencePath = fragmentIndex < 0 ? reference : reference.slice(0, fragmentIndex)
  const fragment = fragmentIndex < 0 ? null : reference.slice(fragmentIndex + 1)
  const normalizedPath = toPosix(path.normalize(referencePath))
  return (
    referencePath !== "" &&
    !path.isAbsolute(referencePath) &&
    normalizedPath === referencePath &&
    normalizedPath !== "." &&
    normalizedPath !== ".." &&
    !normalizedPath.startsWith("../") &&
    fragment !== ""
  )
}

function validatedBrowserIntervalProof(proof, registryPath, controlId) {
  if (!proof || Array.isArray(proof) || typeof proof !== "object") {
    throw new Error(`${registryPath} ${controlId} has invalid browser_interval_proof`)
  }
  const keys = Object.keys(proof).sort()
  if (proof.status === "missing") {
    if (
      JSON.stringify(keys) !== JSON.stringify(["issue", "status"]) ||
      !Number.isInteger(proof.issue) ||
      proof.issue <= 0
    ) {
      throw new Error(`${registryPath} ${controlId} has invalid browser_interval_proof`)
    }
    return { status: "missing", issue: proof.issue }
  }
  if (proof.status === "available") {
    const references = proof.references
    if (
      JSON.stringify(keys) !== JSON.stringify(["references", "status"]) ||
      !Array.isArray(references) ||
      references.length === 0 ||
      references.some((reference) => !isCanonicalRepositoryReference(reference)) ||
      JSON.stringify(references) !== JSON.stringify(uniqueSortedStrings(references))
    ) {
      throw new Error(`${registryPath} ${controlId} has invalid browser_interval_proof`)
    }
    return { status: "available", references: [...references] }
  }
  throw new Error(`${registryPath} ${controlId} has invalid browser_interval_proof`)
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex")
}

function gitBlobOid(value) {
  const bytes = Buffer.isBuffer(value) ? value : Buffer.from(value)
  return createHash("sha1")
    .update(`blob ${bytes.length}\0`)
    .update(bytes)
    .digest("hex")
}

function sourceInputSetSha256(registries) {
  return sha256(
    JSON.stringify(
      registries.map(({ path: registryPath, sha256: digest }) => [registryPath, digest])
    )
  )
}

async function discoverFramerailRoutes(root) {
  return (
    await discoverFramerailRouteDescriptors(root, {
      readAbsoluteText: (absolutePath) => readAbsoluteText(root, absolutePath)
    })
  ).map((descriptor) => surface(descriptor))
}

async function applyFramerailRouteActionEvidence(root, records, sourceRevision) {
  return applyFramerailRouteActionEvidenceFromEvidence(root, records, sourceRevision, {
    readJson,
    preloadPinnedRevisionTexts,
    isCanonicalRepositoryReference,
    gitRevisionContains
  })
}

async function applyWwsContractEvidence(root, records, sourceRevision) {
  return applyWwsContractEvidenceFromEvidence(root, records, sourceRevision, {
    readJson,
    isCanonicalRepositoryReference,
    sha256,
    gitBlobOid,
    resolveGitObject,
    readText
  })
}

async function applyCatalogSourceAttribution(root, surfaces, sourceRevision) {
  return applyCatalogSourceAttributionFromEvidence(root, surfaces, sourceRevision, {
    readJson,
    preloadPinnedRevisionTexts,
    gitRevisionContains
  })
}

async function applySiteChangesEvidence(root, records) {
  return applySiteChangesEvidenceFromSource(root, records, {
    readText: (selectedRoot, relativePath) => readText(selectedRoot, relativePath)
  })
}


async function discoverFramerailAmc(root) {
  return discoverFramerailAmcFromLegacy(root, { readText, readJson })
}

function applyFramerailAmcTests(root, records, sourceRevision) {
  return applyFramerailAmcTestsFromLegacy(root, records, sourceRevision, {
    preloadPinnedRevisionTexts,
    gitRevisionContains
  })
}

async function discoverWikidotPyAmc(root, wikidotPySource) {
  return discoverWikidotPyAmcFromLegacy(root, wikidotPySource, {
    readJson,
    verifyPinnedWikidotPySource,
    pinnedWikidotPyAmcModules
  })
}

async function discoverFramerailXmlRpc(root) {
  return discoverFramerailXmlRpcFromLegacy(root, { readText })
}

async function discoverWwsRoutes(root) {
  const registryPath = "wws/src/route.rs"
  const sourceText = await readText(root, registryPath)
  return discoverWwsRouteRecords(sourceText, registryPath).map(({ method, routePath, reference }) =>
    surface({
      surfaceId: `wws-route:${method}:${routePath}`,
      kind: "wws_route",
      publicOwner: "wws",
      publicReference: [reference]
    })
  )
}

async function discoverOpen43AuditCases(root) {
  return discoverOpen43AuditCasesFromSource(root, {
    readJson: (selectedRoot, relativePath) => readJson(selectedRoot, relativePath),
    readText: (selectedRoot, relativePath) => readText(selectedRoot, relativePath),
    readGitSpecBatch
  })
}

async function buildInventory(root, sourceRevision) {
  SOURCE_INPUTS.clear()
  await verifyAuditedOwnershipReports(root)
  const wikidotPySource = await loadSupportedWikidotPySource(root)
  const [
    provenance,
    catalog,
    deepwell,
    framerailRoutes,
    amc,
    wikidotPyAmc,
    xmlRpc,
    pageActions,
    wws,
    open43,
    semantics
  ] =
    await Promise.all([
      sourceProvenance(root, sourceRevision),
      discoverCatalogFeatures(root, { readJson, sourceInputs: SOURCE_INPUTS }),
      discoverDeepwellJsonRpc(root, { readJson, readText, sha256, isCanonicalRepositoryReference }),
      discoverFramerailRoutes(root),
      discoverFramerailAmc(root),
      discoverWikidotPyAmc(root, wikidotPySource),
      discoverFramerailXmlRpc(root),
      discoverPageActionSurfaces(root, { readJson, readText, sha256, isCanonicalRepositoryReference, toPosix, validatedBrowserIntervalProof }),
      discoverWwsRoutes(root),
      discoverOpen43AuditCases(root),
      readJson(root, SEMANTICS_REGISTRY)
    ])
  validateSemanticsRegistry(semantics)
  const ftmlRawSurfaceManifest = discoverFtmlRawSurfaceManifest(
    provenance.ftml,
    JSON.parse(SOURCE_INPUTS.get("docs/wikidot-specifications/catalog.json")),
    semantics,
    {
      ftmlGitDir: FTML_GIT_DIR,
      listGitTreeBlobs: pinnedSource.listGitTreeBlobs,
      readGitBlobBatch: pinnedSource.readGitBlobBatch
    }
  )
  const auditedOwnershipActive = sha256(
    `${catalog.map(({ surface_id: surfaceId }) => surfaceId).sort().join("\n")}\n`
  ) === AUDITED_CATALOG_SURFACE_IDS_SHA256
  const projectedFramerailRoutes = auditedOwnershipActive
    ? await applyFramerailRouteActionEvidence(
        root,
        framerailRoutes,
        sourceRevision
      )
    : framerailRoutes
  const projectedAmcEvidence = await applySiteChangesEvidence(root, amc)
  const projectedAmc = auditedOwnershipActive
    ? applyFramerailAmcTests(root, projectedAmcEvidence, sourceRevision)
    : projectedAmcEvidence
  const projectedWws = auditedOwnershipActive
    ? await applyWwsContractEvidence(root, wws, sourceRevision)
    : wws
  verifyRegistryBlobs(root, sourceRevision)
  const ftmlProjectedSources = applyFtmlCatalogSourceProjection([
    ...catalog,
    ...deepwell,
    ...projectedFramerailRoutes,
    ...projectedAmc,
    ...wikidotPyAmc,
    ...xmlRpc,
    ...pageActions,
    ...projectedWws,
    ...open43.records
  ].sort((left, right) => left.surface_id.localeCompare(right.surface_id, "en")), ftmlRawSurfaceManifest)
  const projectedSources = await applyCatalogSourceAttribution(
    root,
    ftmlProjectedSources,
    sourceRevision
  )
  const surfaces = normalizeSurfaceOwners(
    applyAuditedIssueOwnership(projectedSources, auditedOwnershipActive),
    ftmlRawSurfaceManifest.catalog_crosswalk,
    semantics,
    auditedOwnershipActive
  )
  const relationshipModel = buildRelationshipModel(surfaces, ftmlRawSurfaceManifest, semantics)
  validateInventory(surfaces, relationshipModel.owner_keys)
  const byKind = {}
  for (const kind of uniqueSortedStrings(surfaces.map(({ kind }) => kind))) {
    byKind[kind] = surfaces.filter((surfaceRecord) => surfaceRecord.kind === kind).length
  }
  const registries = [...SOURCE_INPUTS]
    .map(([registryPath, source]) => ({ path: registryPath, sha256: sha256(source) }))
    .sort((left, right) => left.path.localeCompare(right.path, "en"))
  return {
    schema: SCHEMA,
    relationship_edge_types: [...semantics.relationship_edge_types],
    ...relationshipModel,
    ftml_raw_surface_manifest: ftmlRawSurfaceManifest,
    provenance: {
      wikijump: {
        source_input_set_sha256: sourceInputSetSha256(registries)
      },
      ftml: provenance.ftml,
      registries
    },
    sources: {
      catalog: "docs/wikidot-specifications/catalog.json",
      live_observations: "docs/wikidot-specifications/live-observations.json",
      implementation_ledger: CANONICAL_IMPLEMENTATION_LEDGER,
      implementation_ledger_mirror: "docs/wikidot-specifications/implementation-ledger.json",
      source_coverage: "docs/wikidot-specifications/source-coverage.json",
      compatibility_surface_semantics: SEMANTICS_REGISTRY,
      compatibility_catalog_source_attribution: CATALOG_SOURCE_ATTRIBUTION,
      audited_ownership_reports: AUDITED_OWNERSHIP_REPORTS,
      deepwell_jsonrpc_registry: "deepwell/src/api.rs",
      framerail_routes_root: "framerail/src/routes",
      framerail_amc_registry: "framerail/src/lib/server/ajax-module-connector.js",
      framerail_amc_wire_contracts: "docs/development/framerail-amc-wire-contracts.json",
      framerail_amc_live_evidence: SITE_CHANGES_EVIDENCE_ARTIFACT,
      wikidot_py_amc_contract: "docs/development/wikidot-py-amc-client-parity.json",
      wikidot_py_source: {
        repository: wikidotPySource.repository,
        commit: wikidotPySource.commit,
        root_tree: wikidotPySource.root_tree,
        objects: [...wikidotPySource.objects].map(([objectPath, [type, oid]]) => ({
          path: objectPath,
          type,
          oid
        }))
      },
      framerail_xmlrpc_registry: "framerail/src/lib/server/xmlrpc/methods.ts",
      page_action_registry: "docs/development/wikidot-page-action-surfaces.json",
      wws_route_registry: "wws/src/route.rs",
      open43_audits: open43.auditPaths
    },
    counts: { total: surfaces.length, by_kind: byKind },
    surfaces
  }
}

async function pinnedSourceRevision(root, requestedRevision) {
  if (requestedRevision) return requestedRevision
  return null
}

async function main() {
  const options = parseArgs(process.argv.slice(2))
  const sourceRevision = await pinnedSourceRevision(options.root, options.sourceRevision)
  const inventory = await buildInventory(options.root, sourceRevision)
  await writeRepositoryOutput(options.root, options.output, `${JSON.stringify(inventory, null, 2)}\n`)
  const outputReference = path.relative(options.root, options.output)
  process.stdout.write(
    `wrote ${inventory.counts.total} compatibility surfaces to ${
      outputReference.startsWith("..") ? options.output : toPosix(outputReference)
    }\n`
  )
}

main().catch((error) => {
  process.stderr.write(`compatibility inventory failed: ${error.message}\n`)
  process.exitCode = 1
})
