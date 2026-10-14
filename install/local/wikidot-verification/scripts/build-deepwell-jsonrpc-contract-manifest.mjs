#!/usr/bin/env node

import { createHash } from "node:crypto"
import fs from "node:fs/promises"
import path from "node:path"
import process from "node:process"
import { fileURLToPath } from "node:url"

const SCHEMA = "wikijump.deepwell_jsonrpc_contract_manifest.v1"
const SCRIPT_DIRECTORY = path.dirname(fileURLToPath(import.meta.url))
const DEFAULT_ROOT = path.resolve(SCRIPT_DIRECTORY, "../../../..")
const DEFAULT_OUTPUT = "docs/development/deepwell-jsonrpc-contract-manifest.json"
const API_PATH = "deepwell/src/api.rs"
const ENDPOINTS_DIRECTORY = "deepwell/src/endpoints"
const CONTRACT_TEST = "install/local/wikidot-verification/tests/deepwell-jsonrpc-contract-manifest.test.mjs#Deepwell JSON-RPC manifest exactly covers the current registered contract"
const HISTORICAL_EVIDENCE = [
  "install/local/wikidot-verification/artifacts/pr1334-deepwell-identity-jsonrpc-attribution-20260810.json",
  "install/local/wikidot-verification/artifacts/pr1334-deepwell-page-revision-jsonrpc-attribution-20260810.json"
]
const CURATED_RPC_WITNESSES = [
  {
    method: "category_get",
    source_path: "deepwell/tests/page/page_move.rs",
    test_name: "page_move_render_failure_rolls_back_destination_identity",
    required_fragments: [
      "page_move_rpc_request(",
      '"category_get", json!({"site": site_id, "category": category}),'
    ]
  }
]
const MUTATING_SERVICE_CALLS = new Set([
  "AuthorizationTokenService::create",
  "BlobService::add_blacklist", "BlobService::cancel_upload", "BlobService::hard_delete_all", "BlobService::remove_blacklist", "BlobService::start_upload",
  "CategoryService::update",
  "DomainService::create_custom", "DomainService::remove_custom",
  "FileRevisionService::update",
  "FileService::create", "FileService::delete", "FileService::edit", "FileService::r#move", "FileService::restore", "FileService::rollback",
  "ForumThreadService::get_or_create_page_discussion",
  "ImportService::add_discussion", "ImportService::add_page", "ImportService::add_page_revision", "ImportService::add_site", "ImportService::add_user",
  "LegacyActionService::set_tags",
  "MembershipService::accept_email_invitation", "MembershipService::join", "MembershipService::review_application", "MembershipService::submit_application", "MembershipService::submit_password",
  "MessageService::create_draft", "MessageService::delete_draft", "MessageService::send", "MessageService::update_draft",
  "MfaService::disable", "MfaService::reset_recovery_codes", "MfaService::setup",
  "PageLockService::create", "PageLockService::remove",
  "PageDraftService::remove", "PageDraftService::save",
  "PageMetaTagService::delete", "PageMetaTagService::set",
  "PageRevisionService::rerender", "PageRevisionService::update",
  "PageService::create", "PageService::delete", "PageService::edit", "PageService::r#move", "PageService::restore", "PageService::rollback", "PageService::set_layout",
  "ParentService::create", "ParentService::remove",
  "PermissionService::update_permissions_for_role",
  "RelationService::clear_page_attributions", "RelationService::create_site_ban", "RelationService::create_site_member", "RelationService::create_user_bot_owner", "RelationService::remove_site_ban", "RelationService::remove_site_member", "RelationService::remove_user_bot_owner", "RelationService::set_page_attributions",
  "RoleService::create", "RoleService::delete", "RoleService::grant_role_to_user", "RoleService::reparent_role", "RoleService::revoke_role_from_user", "RoleService::update",
  "SessionService::create", "SessionService::invalidate", "SessionService::invalidate_others", "SessionService::renew", "SessionService::renew_restricted",
  "SiteService::create", "SiteService::update",
  "TextService::create",
  "UserService::activate_from_wikidot", "UserService::add_name_change_token", "UserService::create", "UserService::delete", "UserService::import_wikidot", "UserService::update",
  "VoteService::action", "VoteService::add", "VoteService::remove"
])

function usage() {
  return `Usage: node ${path.basename(process.argv[1])} [--root REPOSITORY] [--output JSON] [--verify]\n`
}

function requireValue(argv, index, option) {
  const value = argv[index]
  if (!value || value.startsWith("--")) throw new Error(`${option} requires a value`)
  return value
}

function parseArgs(argv) {
  let root = DEFAULT_ROOT
  let output
  let verify = false
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index]
    if (argument === "--root") root = path.resolve(requireValue(argv, ++index, "--root"))
    else if (argument === "--output") output = path.resolve(requireValue(argv, ++index, "--output"))
    else if (argument === "--verify") verify = true
    else if (argument === "--help" || argument === "-h") {
      process.stdout.write(usage())
      process.exit(0)
    } else throw new Error(`unknown argument: ${argument}`)
  }
  return { root, output: output ?? path.join(root, DEFAULT_OUTPUT), verify }
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex")
}

function toPosix(value) {
  return value.split(path.sep).join("/")
}

async function readText(root, relativePath) {
  try {
    return await fs.readFile(path.join(root, relativePath), "utf8")
  } catch (error) {
    throw new Error(`cannot read ${relativePath}: ${error.message}`)
  }
}

function normalizeSource(value) {
  return value.replaceAll(/\s+/gu, " ").trim()
}

function registeredMethods(apiSource) {
  const productionSource = apiSource.split("#[cfg(test)]", 1)[0]
  const isolationSelector = productionSource.match(
    /fn rpc_transaction_isolation\(method_name: &str\) -> Option<IsolationLevel> \{([\s\S]*?)\n\}\n\nasync fn build_module/u
  )
  const selectedIsolation = new Map()
  if (isolationSelector) {
    if (!/register!\(@with_isolation \$name, \$method, rpc_transaction_isolation\(\$name\)\)/u.test(productionSource)) {
      throw new Error(`${API_PATH} does not use the declared RPC isolation selector`)
    }
    const selectorBody = isolationSelector[1]
    const matchBody = selectorBody.match(/^\s*match method_name \{([\s\S]*?)\}\s*$/u)?.[1]
    if (!matchBody || !/_\s*=>\s*None\s*,?/u.test(matchBody)) {
      throw new Error(`${API_PATH} RPC isolation selector has no default`)
    }
    const explicitArm = /"([^"]+)"\s*=>\s*Some\(IsolationLevel::([A-Za-z_][A-Za-z0-9_]*)\)\s*,?/gu
    for (const match of matchBody.matchAll(explicitArm)) {
      if (selectedIsolation.has(match[1])) throw new Error(`duplicate RPC isolation selector: ${match[1]}`)
      selectedIsolation.set(match[1], match[2])
    }
    if (matchBody.replaceAll(explicitArm, "").replace(/_\s*=>\s*None\s*,?/u, "").trim()) {
      throw new Error(`${API_PATH} contains an unsupported RPC isolation selector arm`)
    }
  }
  const registrations = [...productionSource.matchAll(
    /register!\s*\(\s*"([^"]+)"\s*,\s*([A-Za-z_][A-Za-z0-9_]*)(?:\s*,\s*isolation\s*=\s*IsolationLevel::([A-Za-z_][A-Za-z0-9_]*))?\s*,?\s*\)/gu
  )].map((match) => ({
    method: match[1],
    handler: match[2],
    isolation: match[3] ?? selectedIsolation.get(match[1]) ?? "default"
  }))
  const declaredNames = [...productionSource.matchAll(/register!\s*\(\s*"([^"]+)"/gu)].map((match) => match[1])
  if (registrations.length === 0) throw new Error(`${API_PATH} declares no JSON-RPC methods`)
  if (registrations.length !== declaredNames.length) {
    throw new Error(`${API_PATH} contains an unsupported register! declaration`)
  }
  const names = new Set()
  for (const registration of registrations) {
    if (names.has(registration.method)) throw new Error(`duplicate JSON-RPC registration: ${registration.method}`)
    names.add(registration.method)
  }
  for (const method of selectedIsolation.keys()) {
    if (!names.has(method)) throw new Error(`RPC isolation selector names unregistered method: ${method}`)
  }
  return registrations.sort((left, right) => left.method.localeCompare(right.method, "en"))
}

async function endpointSources(root) {
  const absoluteDirectory = path.join(root, ENDPOINTS_DIRECTORY)
  const files = (await fs.readdir(absoluteDirectory, { withFileTypes: true }))
    .filter((entry) => entry.isFile() && entry.name.endsWith(".rs"))
    .map((entry) => entry.name)
    .filter((name) => !["macros.rs", "mod.rs"].includes(name))
    .sort()
  const byHandler = new Map()
  for (const file of files) {
    const relativePath = `${ENDPOINTS_DIRECTORY}/${file}`
    const moduleFiles = [relativePath]
    const moduleDirectory = path.join(
      absoluteDirectory,
      file.replace(/\.rs$/u, "")
    )
    try {
      const pending = [moduleDirectory]
      while (pending.length > 0) {
        const directory = pending.pop()
        for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
          const target = path.join(directory, entry.name)
          if (entry.isDirectory()) pending.push(target)
          else if (entry.isFile() && entry.name.endsWith(".rs")) {
            moduleFiles.push(toPosix(path.relative(root, target)))
          }
        }
      }
    } catch (error) {
      if (error?.code !== "ENOENT") throw error
    }

    const moduleFunctions = new Map()
    const topLevelFunctions = []
    let topLevelSource = ""
    for (const modulePath of moduleFiles.sort()) {
      const source = await readText(root, modulePath)
      if (modulePath === relativePath) topLevelSource = source
      for (const match of source.matchAll(/^[ \t]*(?:(pub(?:\s*\([^)]*\))?)\s+)?(async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s*<[^>{}]*>)?\s*\(/gmu)) {
        const entry = {
          handler: match[3],
          source,
          sourcePath: modulePath,
          offset: match.index,
          body: rustFunctionBody(source, match.index, `${modulePath}#${match[3]}`)
        }
        const matches = moduleFunctions.get(entry.handler) ?? []
        matches.push(entry)
        moduleFunctions.set(entry.handler, matches)
        if (modulePath === relativePath) topLevelFunctions.push(entry)
      }
    }
    for (const entries of moduleFunctions.values()) {
      for (const entry of entries) entry.fileFunctions = moduleFunctions
    }
    const handlers = [
      ...topLevelFunctions,
      ...reexportedFunctions(relativePath, topLevelSource, moduleFunctions)
    ]
    for (const entry of handlers) {
      if (!entry.source.slice(entry.offset).trimStart().startsWith("pub async fn")) continue
      const matches = byHandler.get(entry.handler) ?? []
      matches.push(entry)
      byHandler.set(entry.handler, matches)
    }
  }
  return byHandler
}

// An endpoint file may define handlers in a private child module and
// re-export them (`pub use child::{a, b};`). Only names the top-level file
// re-exports are handlers; other child functions stay helpers.
function reexportedFunctions(relativePath, topLevel, moduleFunctions) {
  const moduleDirectory = relativePath.replace(/\.rs$/u, "")
  const entries = []
  for (const match of topLevel.matchAll(/^[ \t]*pub\s+use\s+([A-Za-z_][A-Za-z0-9_]*)::(?:\{([^}]*)\}|([A-Za-z_][A-Za-z0-9_]*))\s*;/gmu)) {
    const childPath = `${moduleDirectory}/${match[1]}.rs`
    const names = (match[2] ?? match[3]).split(",").map((name) => name.trim()).filter(Boolean)
    for (const name of names) {
      const candidates = (moduleFunctions.get(name) ?? []).filter((entry) => entry.sourcePath === childPath)
      if (candidates.length !== 1) {
        throw new Error(`${relativePath} re-exports ${match[1]}::${name}, which resolves ${candidates.length} functions in ${childPath}`)
      }
      entries.push(candidates[0])
    }
  }
  return entries
}

function rustFunctionBody(source, offset, reference) {
  const open = source.indexOf("{", offset)
  if (open < 0) throw new Error(`${reference} has no function body`)
  let depth = 0
  for (let index = open; index < source.length; index += 1) {
    if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2)
      index = newline < 0 ? source.length : newline
      continue
    }
    if (source.startsWith("/*", index)) {
      const close = source.indexOf("*/", index + 2)
      if (close < 0) throw new Error(`${reference} has an unterminated block comment`)
      index = close + 1
      continue
    }
    const rawString = /^(?:b|c)?r(#+)?"/u.exec(source.slice(index))
    if (rawString) {
      const terminator = `"${rawString[1] ?? ""}`
      const close = source.indexOf(terminator, index + rawString[0].length)
      if (close < 0) throw new Error(`${reference} has an unterminated raw string`)
      index = close + terminator.length - 1
      continue
    }
    if (source[index] === '"') {
      index += 1
      while (index < source.length && source[index] !== '"') {
        if (source[index] === "\\") index += 1
        index += 1
      }
      if (index >= source.length) throw new Error(`${reference} has an unterminated string`)
      continue
    }
    if (source[index] === "{") depth += 1
    else if (source[index] === "}") {
      depth -= 1
      if (depth === 0) return source.slice(offset, index + 1)
    }
  }
  throw new Error(`${reference} has an unterminated function body`)
}

function localFunctionClosure(entry) {
  const closure = []
  const visited = new Set()
  function visit(current) {
    const identity = `${current.sourcePath}#${current.handler}`
    if (visited.has(identity)) return
    visited.add(identity)
    closure.push(current)
    for (const match of current.body.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu)) {
      const matches = current.fileFunctions.get(match[1]) ?? []
      if (matches.length === 1) visit(matches[0])
    }
  }
  visit(entry)
  return closure
}

function parameterDecoder(source) {
  const macro = /\b(parse(?:_one)?)!\s*\(/u.exec(source)
  if (macro !== null) {
    const open = source.indexOf("(", macro.index)
    const close = matchingParenthesis(source, open, "parameter decoder")
    return normalizeSource(source.slice(macro.index, close + 1))
  }
  const method = /\b([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*parse\s*\(\s*\)\s*\.or_raise\s*\(/u.exec(source)
  return method === null ? "not_decoded" : `${method[1]}.parse()`
}

function matchingParenthesis(source, open, reference) {
  let depth = 0
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === '"') {
      index += 1
      while (index < source.length && source[index] !== '"') {
        if (source[index] === "\\") index += 1
        index += 1
      }
      continue
    }
    if (source[index] === "(") depth += 1
    else if (source[index] === ")") {
      depth -= 1
      if (depth === 0) return index
    }
  }
  throw new Error(`${reference} has an unterminated parenthesized expression`)
}

function sourceRequirements(sources) {
  const requirements = new Set()
  for (const source of sources) {
    if (/ctx\.request\(\)\.user_id\(\)/u.test(source)) requirements.add("authenticated_user")
    else if (/ctx\.request\(\)\.user_id\b/u.test(source)) requirements.add("optional_user")
    if (/ctx\.request\(\)\.site_id\(\)/u.test(source)) requirements.add("site_context")
    if (/ctx\.request\(\)\.page_reference\(\)/u.test(source)) requirements.add("page_context")
    if (/MutationAuthorization::/u.test(source)) requirements.add("mutation_authorization")
    if (/PermissionService::/u.test(source)) requirements.add("permission_check")
  }
  return [...requirements].sort()
}

function mutationSignals(sources) {
  const signals = []
  const serviceCall = /\b([A-Za-z_][A-Za-z0-9_]*Service)::(?:(r#)?([A-Za-z_][A-Za-z0-9_]*))\b/gu
  for (const source of sources) {
    for (const match of source.matchAll(serviceCall)) {
      const call = `${match[1]}::${match[2] ?? ""}${match[3]}`
      if (MUTATING_SERVICE_CALLS.has(call)) signals.push(call)
    }
  }
  return [...new Set(signals)].sort()
}

function endpointContract(entry, transport) {
  const source = entry.body
  const params = /\b(_?[A-Za-z_][A-Za-z0-9_]*)\s*:\s*Params\s*<\s*'static\s*>/u.exec(source)
  if (!params) throw new Error(`${entry.sourcePath}#${entry.handler} has no Params argument`)
  const closure = localFunctionClosure(entry)
  const sources = closure.map(({ body }) => body)
  const signals = mutationSignals(sources)
  return {
    endpoint_owner: {
      component: "deepwell",
      handler: entry.handler,
      source: `${entry.sourcePath}#${entry.handler}`,
      source_sha256: sha256(entry.source),
      function_sha256: sha256(source)
    },
    params_schema: {
      transport: "jsonrpsee::types::params::Params<'static>",
      parameter_name: params[1],
      decoder: parameterDecoder(source)
    },
    actor_context: {
      transport_authentication: transport.authentication,
      request_context_headers: transport.requestContextHeaders,
      requirements: sourceRequirements(sources),
      requirement_sources: closure.map(({ handler, sourcePath }) => `${sourcePath}#${handler}`).sort()
    },
    mutation_class: {
      classification: signals.length === 0 ? "read_only" : "mutating",
      source_signals: signals
    }
  }
}

function deriveTransportContract(rpcAuthSource, middlewareSource) {
  const tokenLength = /const\s+TOKEN_HEX_LENGTH\s*:\s*usize\s*=\s*([0-9]+)\s*;/u.exec(rpcAuthSource)
  const authorizationImport = /use\s+http::header::\{([^}]+)\};/u.exec(rpcAuthSource)
  if (
    tokenLength === null ||
    authorizationImport === null ||
    !authorizationImport[1].split(",").map((entry) => entry.trim()).includes("AUTHORIZATION") ||
    !rpcAuthSource.includes("headers.get_all(AUTHORIZATION).iter()") ||
    !/if\s+values\.next\(\)\.is_some\(\)\s*\{\s*return false;\s*\}/u.test(rpcAuthSource) ||
    !rpcAuthSource.includes('value.strip_prefix("Bearer ")') ||
    !rpcAuthSource.includes("byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)")
  ) {
    throw new Error("deepwell/src/middleware/rpc_auth.rs has an unsupported RPC authentication declaration")
  }
  const contextFunction = rustFunctionBody(
    middlewareSource,
    middlewareSource.indexOf("fn request_context_headers"),
    "deepwell/src/middleware.rs#request_context_headers"
  )
  const headers = [...contextFunction.matchAll(
    /let\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*request[\s\S]*?\.get\("([^"]+)"\)/gu
  )].map((match) => ({ target: match[1], header: match[2] }))
  const expected = [
    ["session_token", "X-Deepwell-Session-Token"],
    ["site_id", "X-Deepwell-Site-Id"],
    ["page_ref", "X-Deepwell-Page"]
  ]
  if (
    headers.length !== expected.length ||
    !expected.every(([target, header], index) => headers[index]?.target === target && headers[index]?.header === header)
  ) {
    throw new Error("deepwell/src/middleware.rs has an unsupported request context header declaration")
  }
  return {
    authentication: {
      header_symbol: "AUTHORIZATION",
      header: "Authorization",
      scheme: "Bearer",
      token_format: `${tokenLength[1]} lowercase hexadecimal characters`,
      duplicate_values: "rejected"
    },
    requestContextHeaders: headers.map(({ header, target }) => ({ header, target }))
  }
}

async function behavioralWitnesses(root, registeredMethodNames) {
  const directory = path.join(root, "deepwell/tests")
  const files = []
  async function visit(current) {
    for (const entry of await fs.readdir(current, { withFileTypes: true })) {
      const target = path.join(current, entry.name)
      if (entry.isDirectory()) await visit(target)
      else if (entry.isFile() && entry.name.endsWith(".rs")) files.push(target)
    }
  }
  await visit(directory)
  const endpointWitnesses = new Map()
  const rpcWitnesses = new Map()
  for (const file of files.sort()) {
    const source = await fs.readFile(file, "utf8")
    const tests = [...source.matchAll(/#\[(?:tokio::test|test)\]\s*(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/gu)]
    for (const match of source.matchAll(/run_endpoint(?:_err)?!\(\s*runner\s*,\s*([A-Za-z_][A-Za-z0-9_]*)/gu)) {
      const testName = [...tests].reverse().find(({ index }) => index < match.index)?.[1]
      if (testName === undefined) continue
      const handler = match[1]
      const references = endpointWitnesses.get(handler) ?? []
      references.push(`${toPosix(path.relative(root, file))}#${testName}`)
      endpointWitnesses.set(handler, references)
    }
    for (const match of source.matchAll(/rpc_request\(\s*"([^"]+)"\s*,/gu)) {
      const testName = [...tests].reverse().find(({ index }) => index < match.index)?.[1]
      if (testName === undefined) continue
      const method = match[1]
      const references = rpcWitnesses.get(method) ?? []
      references.push(`${toPosix(path.relative(root, file))}#${testName}`)
      rpcWitnesses.set(method, references)
    }
  }
  for (const witness of CURATED_RPC_WITNESSES) {
    if (!registeredMethodNames.has(witness.method)) {
      throw new Error(`curated RPC witness references an unregistered method: ${witness.method}`)
    }
    const source = await readText(root, witness.source_path)
    const declarations = [...source.matchAll(
      /#\[(?:tokio::test|test)\]\s*(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/gu
    )].filter((match) => match[1] === witness.test_name)
    if (declarations.length !== 1) {
      throw new Error(
        `${witness.source_path} must declare exactly one curated RPC witness test named ${witness.test_name}`
      )
    }
    const reference = `${witness.source_path}#${witness.test_name}`
    const body = normalizeSource(rustFunctionBody(source, declarations[0].index, reference))
    for (const fragment of witness.required_fragments) {
      if (!body.includes(normalizeSource(fragment))) {
        throw new Error(`${reference} no longer contains the curated ${witness.method} RPC witness`)
      }
    }
    const references = rpcWitnesses.get(witness.method) ?? []
    references.push(reference)
    rpcWitnesses.set(witness.method, references)
  }
  return {
    endpoint: new Map([...endpointWitnesses].map(([handler, references]) => [handler, [...new Set(references)].sort()])),
    rpc: new Map([...rpcWitnesses].map(([method, references]) => [method, [...new Set(references)].sort()]))
  }
}

async function buildManifest(root) {
  const apiSource = await readText(root, API_PATH)
  const registrations = registeredMethods(apiSource)
  const [endpointByHandler, witnesses, rpcAuthSource, middlewareSource] = await Promise.all([
    endpointSources(root),
    behavioralWitnesses(root, new Set(registrations.map(({ method }) => method))),
    readText(root, "deepwell/src/middleware/rpc_auth.rs"),
    readText(root, "deepwell/src/middleware.rs")
  ])
  const transport = deriveTransportContract(rpcAuthSource, middlewareSource)
  const methods = []
  for (const registration of registrations) {
    const matches = endpointByHandler.get(registration.handler) ?? []
    if (matches.length !== 1) {
      throw new Error(`${API_PATH}#register:${registration.method} resolves ${matches.length} endpoint handlers named ${registration.handler}`)
    }
    methods.push({
      method: registration.method,
      ...endpointContract(matches[0], transport),
      transaction_isolation: registration.isolation,
      test_witness: witnessFor(
        witnesses.rpc.get(registration.method) ?? [],
        witnesses.endpoint.get(registration.handler) ?? []
      )
    })
  }
  const historicalEvidence = []
  for (const evidencePath of HISTORICAL_EVIDENCE) {
    const source = await readText(root, evidencePath)
    historicalEvidence.push({ path: evidencePath, sha256: sha256(source), classification: "historical" })
  }
  return {
    schema: SCHEMA,
    source_identities: {
      jsonrpc_registry: { path: API_PATH, sha256: sha256(apiSource) },
      request_context_middleware: {
        path: "deepwell/src/middleware.rs",
        sha256: sha256(middlewareSource)
      },
      rpc_authentication_middleware: {
        path: "deepwell/src/middleware/rpc_auth.rs",
        sha256: sha256(rpcAuthSource)
      }
    },
    historical_evidence: historicalEvidence,
    method_count: methods.length,
    methods
  }
}

function witnessFor(rpcReferences, endpointReferences) {
  if (rpcReferences.length > 0) {
    return {
      kind: "rpc_behavioral",
      reference: rpcReferences[0],
      alternatives: rpcReferences.slice(1),
      source_contract_reference: CONTRACT_TEST
    }
  }
  if (endpointReferences.length > 0) {
    return {
      kind: "endpoint_behavioral",
      reference: endpointReferences[0],
      alternatives: endpointReferences.slice(1),
      source_contract_reference: CONTRACT_TEST
    }
  }
  return {
    kind: "source_contract_only",
    reference: CONTRACT_TEST,
    coverage_gap: {
      searched_root: "deepwell/tests",
      reason: "no direct run_endpoint handler invocation or rpc_request method invocation was found",
      searched_invocations: ["run_endpoint!(runner, handler, ...)", "rpc_request(\"method\", ...)"]
    }
  }
}

async function main() {
  const options = parseArgs(process.argv.slice(2))
  const manifest = `${JSON.stringify(await buildManifest(options.root), null, 2)}\n`
  if (options.verify) {
    let actual
    try {
      actual = await fs.readFile(options.output, "utf8")
    } catch (error) {
      throw new Error(`cannot read contract manifest ${options.output}: ${error.message}`)
    }
    if (actual !== manifest) throw new Error(`contract manifest is stale: ${toPosix(path.relative(options.root, options.output))}`)
    process.stdout.write(`verified ${JSON.parse(manifest).method_count} Deepwell JSON-RPC contracts\n`)
    return
  }
  await fs.mkdir(path.dirname(options.output), { recursive: true })
  await fs.writeFile(options.output, manifest)
  process.stdout.write(`wrote ${JSON.parse(manifest).method_count} Deepwell JSON-RPC contracts to ${toPosix(path.relative(options.root, options.output))}\n`)
}

main().catch((error) => {
  process.stderr.write(`Deepwell JSON-RPC contract manifest failed: ${error.message}\n`)
  process.exitCode = 1
})
