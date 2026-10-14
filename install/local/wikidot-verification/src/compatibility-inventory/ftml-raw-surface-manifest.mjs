import { createHash } from "node:crypto"
import path from "node:path"

import { scanRustTokens } from "./rust-source.mjs"
import { siblingCheckout } from "../sibling-checkout.mjs"

const SEMANTICS_REGISTRY = "docs/development/compatibility-surface-semantics.json"
const DEFAULT_FTML_GIT_DIR = path.join(siblingCheckout("ftml", "WIKIJUMP_FTML_CHECKOUT"), ".git")

function sha256(value) {
  return createHash("sha256").update(value).digest("hex")
}

function uniqueSortedStrings(values) {
  return [...new Set(values.filter((value) => typeof value === "string" && value !== ""))].sort()
}

function loadFtmlSnapshot(
  revision,
  { ftmlGitDir = DEFAULT_FTML_GIT_DIR, listGitTreeBlobs, readGitBlobBatch }
) {
  const tree = listGitTreeBlobs([`--git-dir=${ftmlGitDir}`], revision, "pinned FTML tree")
  const moduleRoot = "src/parsing/rule/impls/block/blocks/module/modules/"
  const rendererRoot = "src/render/html/element/"
  const fixedPaths = new Set([
    "src/parsing/lexer.pest",
    "src/preproc/parser_functions/mod.rs",
    "conf/blocks.toml",
    "src/tree/element/object.rs",
    "src/delayed.rs",
    "src/render/html/element/mod.rs"
  ])
  const selected = [...tree.entries()]
    .filter(([objectPath]) =>
      fixedPaths.has(objectPath) ||
      (objectPath.startsWith(moduleRoot) && objectPath.endsWith(".rs") && !objectPath.endsWith("/mod.rs")) ||
      (objectPath.startsWith(rendererRoot) && objectPath.endsWith(".rs")) ||
      (objectPath.startsWith("test/") && objectPath.endsWith("/wikidot.html"))
    )
    .map(([path, oid]) => ({ path, oid }))
    .sort((left, right) => left.path.localeCompare(right.path, "en"))
  for (const objectPath of fixedPaths) {
    if (!tree.has(objectPath)) throw new Error(`cannot read pinned FTML source: ${objectPath}`)
  }
  const blobs = readGitBlobBatch([`--git-dir=${ftmlGitDir}`], selected, "pinned FTML source")
  const read = (objectPath, sources) => {
    const bytes = blobs.get(objectPath)
    const blob = tree.get(objectPath)
    if (!bytes || !blob) throw new Error(`cannot read pinned FTML source: ${objectPath}`)
    sources.set(objectPath, { path: objectPath, blob, sha256: sha256(bytes) })
    return bytes.toString("utf8")
  }
  const list = (prefix) => [...tree.keys()].filter((objectPath) => objectPath.startsWith(`${prefix}/`) || objectPath === prefix)
  return { read, list }
}

function rustEnumVariants(source, enumName, sourcePath) {
  const tokens = scanRustTokens(source, sourcePath)
  const enumIndex = tokens.findIndex(
    (token, index) => token.value === "enum" && tokens[index + 1]?.value === enumName
  )
  const start = tokens.findIndex((token, index) => index > enumIndex && token.value === "{")
  if (enumIndex < 0 || start < 0) throw new Error(`${sourcePath} has no ${enumName} enum`)
  const variants = []
  const depth = { "{": 1, "(": 0, "[": 0 }
  const closing = { "}": "{", ")": "(", "]": "[" }
  let expectVariant = true
  for (let index = start + 1; index < tokens.length && depth["{"] > 0; index += 1) {
    const token = tokens[index]
    if (token.value in depth) depth[token.value] += 1
    else if (token.value in closing) depth[closing[token.value]] -= 1
    else if (depth["{"] === 1 && depth["("] === 0 && depth["["] === 0 && token.value === ",") {
      expectVariant = true
    } else if (
      depth["{"] === 1 &&
      depth["("] === 0 &&
      depth["["] === 0 &&
      expectVariant &&
      token.kind === "identifier"
    ) {
      variants.push(token.value)
      expectVariant = false
    }
  }
  if (new Set(variants).size !== variants.length) {
    throw new Error(`${sourcePath} has duplicate ${enumName} variants`)
  }
  return variants
}

function ftmlRecord(surfaceId, kind, name, sourcePath, extra = {}) {
  return {
    surface_id: surfaceId,
    kind,
    name,
    source_reference: sourcePath,
    ...extra
  }
}

function buildFtmlCrosswalk(catalog, recordIds, semantics) {
  const nominated = catalog.features
    .filter((feature) =>
      (feature.suggested_tdd_seams ?? []).some((seam) => seam.includes("FTML public parse/render"))
    )
    .map(({ id }) => id)
    .sort()
  const rows = semantics.ftml?.catalog_crosswalk
  if (!Array.isArray(rows)) throw new Error(`${SEMANTICS_REGISTRY} has no FTML catalog crosswalk`)
  const featureIds = rows.map(({ feature_id: featureId }) => featureId)
  if (
    new Set(featureIds).size !== featureIds.length ||
    JSON.stringify([...featureIds].sort()) !== JSON.stringify(nominated)
  ) {
    throw new Error(`${SEMANTICS_REGISTRY} FTML crosswalk does not exactly match Catalog nominations`)
  }
  for (const row of rows) {
    const fields = ["parsed_by", "rendered_by", "tested_by"]
    if (fields.some((field) => !Array.isArray(row[field]))) {
      throw new Error(`${SEMANTICS_REGISTRY} has malformed FTML crosswalk row: ${row.feature_id}`)
    }
    const ftmlSurfaces = uniqueSortedStrings(fields.flatMap((field) => row[field]))
    if (JSON.stringify(ftmlSurfaces) !== JSON.stringify(row.ftml_surfaces)) {
      throw new Error(`${SEMANTICS_REGISTRY} has inconsistent FTML crosswalk row: ${row.feature_id}`)
    }
    for (const surfaceId of ftmlSurfaces) {
      if (!recordIds.has(surfaceId)) throw new Error(`${row.feature_id} links unknown FTML surface: ${surfaceId}`)
    }
    if (
      row.runtime_owner !== null &&
      row.runtime_owner !== `wikijump.runtime:${row.feature_id}`
    ) {
      throw new Error(`${SEMANTICS_REGISTRY} has invalid runtime owner: ${row.feature_id}`)
    }
  }
  return rows.map((row) => ({ ...row }))
}

export function discoverFtmlRawSurfaceManifest(
  ftmlSource,
  catalog,
  semantics,
  dependencies
) {
  const revision = ftmlSource.commit
  const snapshot = loadFtmlSnapshot(revision, dependencies)
  const sources = new Map()
  const records = []
  const add = (record) => records.push(record)

  const lexerPath = "src/parsing/lexer.pest"
  const lexer = snapshot.read(lexerPath, sources)
  for (const name of [...lexer.matchAll(/^([a-z_][a-z0-9_]*)\s*=/gmu)].map((match) => match[1])) {
    add(ftmlRecord(`ftml.tokenizer:${name}`, "lexer_rule", name, lexerPath))
  }

  const parserFunctionsPath = "src/preproc/parser_functions/mod.rs"
  const parserFunctions = snapshot.read(parserFunctionsPath, sources)
  for (const name of [...parserFunctions.matchAll(/^\s*"(if|ifexpr|expr)"\s*=>\s*ParserFunctionKind::/gmu)].map((match) => `#${match[1]}`)) {
    add(ftmlRecord(`ftml.preprocessor:${name}`, "parser_function", name, parserFunctionsPath))
  }

  const blocksPath = "conf/blocks.toml"
  const blocks = snapshot.read(blocksPath, sources)
  const sections = [...blocks.matchAll(/^\[([a-z0-9-]+)\]$/gmu)]
  for (const [index, section] of sections.entries()) {
    const name = section[1]
    add(ftmlRecord(`ftml.block:${name}`, "canonical_block", name, blocksPath))
    const end = sections[index + 1]?.index ?? blocks.length
    const aliases = /^aliases\s*=\s*(\[[^\n]*\])/mu.exec(blocks.slice(section.index, end))
    for (const alias of aliases ? JSON.parse(aliases[1]) : []) {
      add(ftmlRecord(`ftml.block-alias:${alias}->${name}`, "block_alias", alias, blocksPath, {
        canonical_surface: `ftml.block:${name}`
      }))
    }
  }

  const moduleRoot = "src/parsing/rule/impls/block/blocks/module/modules"
  for (const modulePath of snapshot.list(moduleRoot).filter((value) => value.endsWith(".rs") && !value.endsWith("/mod.rs"))) {
    const moduleSource = snapshot.read(modulePath, sources)
    const name = /accepts_names:\s*&\["([A-Za-z]+)"\]/u.exec(moduleSource)?.[1]
    if (!name) throw new Error(`${modulePath} has no typed module name`)
    add(ftmlRecord(`ftml.module:${name}`, "typed_module", name, modulePath))
  }

  const astPath = "src/tree/element/object.rs"
  const ast = snapshot.read(astPath, sources)
  for (const name of rustEnumVariants(ast, "Element", astPath)) {
    add(ftmlRecord(`ftml.ast:${name}`, "ast_variant", name, astPath))
  }

  const delayedPath = "src/delayed.rs"
  const delayed = snapshot.read(delayedPath, sources)
  for (const name of rustEnumVariants(delayed, "DelayedNode", delayedPath)) {
    add(ftmlRecord(`ftml.delayed:${name}`, "delayed_form", name, delayedPath))
  }
  for (const name of rustEnumVariants(delayed, "GeneratedKind", delayedPath)) {
    add(ftmlRecord(`ftml.generated:${name}`, "generated_runtime_kind", name, delayedPath))
  }

  const rendererRoot = "src/render/html/element"
  const rendererIndexPath = `${rendererRoot}/mod.rs`
  const rendererIndex = snapshot.read(rendererIndexPath, sources)
  add(ftmlRecord("ftml.renderer:dispatcher", "renderer_module", "dispatcher", rendererIndexPath))
  for (const name of [...rendererIndex.matchAll(/^mod\s+([a-z_]+);$/gmu)].map((match) => match[1])) {
    const rendererPath = `${rendererRoot}/${name}.rs`
    snapshot.read(rendererPath, sources)
    add(ftmlRecord(`ftml.renderer:${name}`, "renderer_module", name, rendererPath))
  }

  for (const fixturePath of snapshot.list("test").filter((value) => value.endsWith("/wikidot.html"))) {
    snapshot.read(fixturePath, sources)
    const name = fixturePath.slice(0, -"/wikidot.html".length)
    add(ftmlRecord(`ftml.fixture:${name}`, "wikidot_fixture", name, fixturePath))
  }

  const identifiers = records.map(({ surface_id: surfaceId }) => surfaceId)
  if (new Set(identifiers).size !== identifiers.length) throw new Error("duplicate FTML raw surface")
  const counts = {
    lexer_rules: records.filter(({ kind }) => kind === "lexer_rule").length,
    parser_functions: records.filter(({ kind }) => kind === "parser_function").length,
    canonical_blocks: records.filter(({ kind }) => kind === "canonical_block").length,
    block_aliases: records.filter(({ kind }) => kind === "block_alias").length,
    typed_modules: records.filter(({ kind }) => kind === "typed_module").length,
    ast_variants: records.filter(({ kind }) => kind === "ast_variant").length,
    delayed_forms: records.filter(({ kind }) => kind === "delayed_form").length,
    generated_runtime_kinds: records.filter(({ kind }) => kind === "generated_runtime_kind").length,
    renderer_modules: records.filter(({ kind }) => kind === "renderer_module").length,
    wikidot_fixtures: records.filter(({ kind }) => kind === "wikidot_fixture").length,
    total: records.length
  }
  if (JSON.stringify(counts) !== JSON.stringify(semantics.ftml?.counts)) {
    throw new Error(`pinned FTML raw surface denominator drift: ${JSON.stringify(counts)}`)
  }
  const identities = records
    .map(({ surface_id: surfaceId, kind }) => ({ surface_id: surfaceId, kind }))
    .sort((left, right) => left.surface_id.localeCompare(right.surface_id, "en"))
  const expectedIdentities = semantics.ftml?.raw_surface_identities
  if (
    !Array.isArray(expectedIdentities) ||
    JSON.stringify(identities) !== JSON.stringify(expectedIdentities)
  ) {
    throw new Error("pinned FTML raw surface identities drift")
  }
  const recordIds = new Set(identifiers)
  return {
    schema: "wikijump.ftml_raw_surface_manifest.v1",
    source: { ...ftmlSource },
    registries: [...sources.values()].sort((left, right) => left.path.localeCompare(right.path, "en")),
    counts,
    records: records.sort((left, right) => left.surface_id.localeCompare(right.surface_id, "en")),
    catalog_crosswalk: buildFtmlCrosswalk(catalog, recordIds, semantics)
  }
}
