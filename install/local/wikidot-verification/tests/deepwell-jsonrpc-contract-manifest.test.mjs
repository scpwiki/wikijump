import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import fs from "node:fs/promises"
import os from "node:os"
import path from "node:path"
import test from "node:test"
import { fileURLToPath } from "node:url"

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..")
const cliPath = path.join(
  repositoryRoot,
  "install/local/wikidot-verification/scripts/build-deepwell-jsonrpc-contract-manifest.mjs"
)
const inventoryCliPath = path.join(
  repositoryRoot,
  "install/local/wikidot-verification/scripts/build-compatibility-surface-inventory.mjs"
)
const manifestPath = path.join(repositoryRoot, "docs/development/deepwell-jsonrpc-contract-manifest.json")
const currentSourceRevision = spawnSync("git", ["rev-parse", "HEAD^{commit}"], {
  cwd: repositoryRoot,
  encoding: "utf8"
}).stdout.trim()
const historicalEvidencePaths = [
  "install/local/wikidot-verification/artifacts/pr1334-deepwell-identity-jsonrpc-attribution-20260810.json",
  "install/local/wikidot-verification/artifacts/pr1334-deepwell-page-revision-jsonrpc-attribution-20260810.json"
]

function runCli(cli, argumentsList) {
  return spawnSync(process.execPath, [cli, ...argumentsList], { encoding: "utf8" })
}

async function writeSourceFixture(root) {
  await fs.cp(path.join(repositoryRoot, "deepwell"), path.join(root, "deepwell"), { recursive: true })
  for (const evidencePath of historicalEvidencePaths) {
    const target = path.join(root, evidencePath)
    await fs.mkdir(path.dirname(target), { recursive: true })
    await fs.copyFile(path.join(repositoryRoot, evidencePath), target)
  }
}

test("Deepwell JSON-RPC manifest exactly covers the current registered contract", async (t) => {
  const result = runCli(cliPath, ["--root", repositoryRoot, "--verify"])

  assert.equal(result.status, 0, result.stderr)
  const manifest = JSON.parse(await fs.readFile(manifestPath, "utf8"))
  assert.equal(manifest.schema, "wikijump.deepwell_jsonrpc_contract_manifest.v1")
  assert.ok(manifest.method_count > 0)
  assert.equal(manifest.methods.length, manifest.method_count)
  assert.equal(new Set(manifest.methods.map(({ method }) => method)).size, manifest.method_count)
  for (const method of manifest.methods) {
    assert.equal(method.endpoint_owner.component, "deepwell")
    assert.match(method.endpoint_owner.source, /^deepwell\/src\/endpoints\/(?:[^/]+\/)?[^/]+\.rs#[A-Za-z_][A-Za-z0-9_]*$/u)
    assert.match(method.endpoint_owner.source_sha256, /^[0-9a-f]{64}$/u)
    assert.equal(method.params_schema.transport, "jsonrpsee::types::params::Params<'static>")
    assert.equal(typeof method.params_schema.decoder, "string")
    assert.ok(Array.isArray(method.actor_context.requirements))
    assert.ok(["mutating", "read_only"].includes(method.mutation_class.classification))
    assert.ok(["default", "RepeatableRead"].includes(method.transaction_isolation))
    assert.ok(["rpc_behavioral", "endpoint_behavioral", "source_contract_only"].includes(method.test_witness.kind))
  }

  const byMethod = new Map(manifest.methods.map((method) => [method.method, method]))
  assert.equal(byMethod.get("parent_get_direct_metadata").transaction_isolation, "RepeatableRead")
  assert.equal(byMethod.get("parent_get").transaction_isolation, "default")
  for (const method of ["page_draft_exists", "page_draft_remove", "page_draft_save"]) {
    assert.equal(byMethod.get(method).test_witness.kind, "endpoint_behavioral")
  }
  assert.equal(byMethod.get("site_tools_list_drafts").test_witness.kind, "source_contract_only")
  assert.equal(byMethod.get("page_draft_exists").mutation_class.classification, "read_only")
  assert.equal(byMethod.get("site_tools_list_drafts").mutation_class.classification, "read_only")
  for (const method of ["page_draft_remove", "page_draft_save"]) {
    assert.equal(byMethod.get(method).mutation_class.classification, "mutating")
  }
  assert.deepEqual(byMethod.get("member_set").actor_context.requirements, [
    "authenticated_user",
    "permission_check"
  ])
  assert.deepEqual(byMethod.get("member_remove").actor_context.requirements, [
    "authenticated_user",
    "permission_check"
  ])
  assert.deepEqual(byMethod.get("membership_join").actor_context.requirements, [])
  assert.equal(byMethod.get("membership_join").params_schema.decoder, "parse!(params, SiteMembership)")
  assert.equal(
    byMethod.get("wikidot_categories_page_list_module").params_schema.decoder,
    "parse!(params, Page)"
  )
  assert.deepEqual(byMethod.get("wikidot_categories_page_list_module").actor_context.requirements, [
    "optional_user"
  ])
  assert.equal(
    byMethod.get("wikidot_categories_page_list_module").mutation_class.classification,
    "read_only"
  )
  assert.equal(byMethod.get("membership_application_list").mutation_class.classification, "read_only")
  for (const method of [
    "membership_password_submit",
    "membership_application_submit",
    "membership_application_review",
    "membership_email_invitation_accept"
  ]) {
    assert.equal(byMethod.get(method).mutation_class.classification, "mutating")
  }
  assert.equal(byMethod.get("blob_blacklist_add").params_schema.decoder, "params.parse()")
  for (const method of [
    "blob_blacklist_add",
    "blob_blacklist_remove",
    "mfa_verify",
    "mfa_setup",
    "mfa_reset_recovery",
    "session_renew"
  ]) {
    assert.equal(byMethod.get(method).mutation_class.classification, "mutating")
  }
  for (const method of ["page_move", "page_rerender", "page_attribution_delete", "vote_action"]) {
    assert.equal(byMethod.get(method).mutation_class.classification, "mutating")
  }
  for (const method of ["blob_blacklist_check", "blob_hard_delete_preview"]) {
    assert.equal(byMethod.get(method).mutation_class.classification, "read_only")
  }
  assert.deepEqual(byMethod.get("ping").actor_context.transport_authentication, {
    header_symbol: "AUTHORIZATION",
    header: "Authorization",
    scheme: "Bearer",
    token_format: "64 lowercase hexadecimal characters",
    duplicate_values: "rejected"
  })
  assert.deepEqual(byMethod.get("ping").actor_context.request_context_headers, [
    { header: "X-Deepwell-Session-Token", target: "session_token" },
    { header: "X-Deepwell-Site-Id", target: "site_id" },
    { header: "X-Deepwell-Page", target: "page_ref" }
  ])
  assert.equal(byMethod.get("blob_blacklist_add").test_witness.kind, "endpoint_behavioral")
  assert.match(byMethod.get("blob_blacklist_add").test_witness.reference, /^deepwell\/tests\/blob\.rs#/u)
  assert.equal(byMethod.get("echo").test_witness.kind, "rpc_behavioral")
  assert.match(byMethod.get("echo").test_witness.reference, /^deepwell\/tests\/rpc_boundary\.rs#/u)
  const sourceOnly = manifest.methods.filter(({ test_witness }) => test_witness.kind === "source_contract_only")
  for (const { test_witness } of sourceOnly) {
    assert.equal(test_witness.coverage_gap.searched_root, "deepwell/tests")
    assert.equal(typeof test_witness.coverage_gap.reason, "string")
  }

  const outputDirectory = await fs.mkdtemp(path.join(repositoryRoot, ".deepwell-contract-inventory-"))
  t.after(() => fs.rm(outputDirectory, { recursive: true, force: true }))
  const inventoryPath = path.join(outputDirectory, "inventory.json")
  const inventoryResult = runCli(inventoryCliPath, [
    "--root", repositoryRoot,
    "--output", inventoryPath,
    "--source-revision", currentSourceRevision
  ])
  assert.equal(inventoryResult.status, 0, inventoryResult.stderr)
  const inventory = JSON.parse(await fs.readFile(inventoryPath, "utf8"))
  const inventoryMethods = inventory.surfaces
    .filter(({ kind }) => kind === "deepwell_jsonrpc_method")
    .map(({ surface_id }) => surface_id.slice("deepwell-jsonrpc:".length))
    .sort()
  assert.deepEqual(inventoryMethods, manifest.methods.map(({ method }) => method).sort())
})

test("Deepwell JSON-RPC verifier rejects omitted or duplicate contract records", async () => {
  const outputDirectory = await fs.mkdtemp(path.join(os.tmpdir(), "deepwell-contract-manifest-"))
  const outputPath = path.join(outputDirectory, "manifest.json")
  await fs.copyFile(manifestPath, outputPath)
  const original = JSON.parse(await fs.readFile(outputPath, "utf8"))

  const omitted = structuredClone(original)
  omitted.methods.pop()
  omitted.method_count -= 1
  await fs.writeFile(outputPath, `${JSON.stringify(omitted, null, 2)}\n`)
  const omittedResult = runCli(cliPath, ["--root", repositoryRoot, "--output", outputPath, "--verify"])
  assert.equal(omittedResult.status, 1)
  assert.match(omittedResult.stderr, /contract manifest is stale/u)

  const duplicate = structuredClone(original)
  duplicate.methods.push(structuredClone(duplicate.methods[0]))
  duplicate.method_count += 1
  await fs.writeFile(outputPath, `${JSON.stringify(duplicate, null, 2)}\n`)
  const duplicateResult = runCli(cliPath, ["--root", repositoryRoot, "--output", outputPath, "--verify"])
  assert.equal(duplicateResult.status, 1)
  assert.match(duplicateResult.stderr, /contract manifest is stale/u)
})

test("Deepwell JSON-RPC generator rejects a duplicate source registration", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "deepwell-contract-source-"))
  await writeSourceFixture(root)
  const apiPath = path.join(root, "deepwell/src/api.rs")
  const apiSource = await fs.readFile(apiPath, "utf8")
  await fs.writeFile(apiPath, apiSource.replace("#[cfg(test)]", 'register!("ping", ping);\n\n#[cfg(test)]'))

  const result = runCli(cliPath, ["--root", root, "--output", path.join(root, "manifest.json")])

  assert.equal(result.status, 1)
  assert.match(result.stderr, /duplicate JSON-RPC registration: ping/u)
})

test("Deepwell JSON-RPC generator rejects an unrecognized isolation selector arm", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "deepwell-contract-isolation-"))
  try {
    await writeSourceFixture(root)
    const apiPath = path.join(root, "deepwell/src/api.rs")
    const apiSource = await fs.readFile(apiPath, "utf8")
    await fs.writeFile(apiPath, apiSource.replace(
      '"parent_get_direct_metadata" => Some(IsolationLevel::RepeatableRead),',
      '"parent_get_direct_metadata" => Some(IsolationLevel::Serializable),\n        "parent_get" => None,'
    ))

    const result = runCli(cliPath, ["--root", root, "--output", path.join(root, "manifest.json")])
    assert.equal(result.status, 1)
    assert.match(result.stderr, /unsupported RPC isolation selector arm/u)
  } finally {
    await fs.rm(root, { recursive: true, force: true })
  }
})

test("Deepwell JSON-RPC generator follows sync and qualified or generic local helpers", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "deepwell-contract-helpers-"))
  t.after(() => fs.rm(root, { recursive: true, force: true }))
  const outputPath = path.join(root, "manifest.json")
  await writeSourceFixture(root)

  const result = runCli(cliPath, ["--root", root, "--output", outputPath])

  assert.equal(result.status, 0, result.stderr)
  const manifest = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const byMethod = new Map(manifest.methods.map((method) => [method.method, method]))
  for (const method of ["page_create", "page_edit", "file_create"]) {
    assert.deepEqual(byMethod.get(method).actor_context.requirements, [
      "authenticated_user",
      "permission_check"
    ])
  }
  assert.ok(byMethod.get("page_edit").actor_context.requirement_sources.includes(
    "deepwell/src/endpoints/page/permissions.rs#ensure_page_edit_permission"
  ))
})

test("Deepwell JSON-RPC generator fails closed when middleware contract declarations drift", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "deepwell-contract-middleware-"))
  await writeSourceFixture(root)
  const authPath = path.join(root, "deepwell/src/middleware/rpc_auth.rs")
  const authSource = await fs.readFile(authPath, "utf8")
  await fs.writeFile(authPath, authSource.replace('value.strip_prefix("Bearer ")', 'value.strip_prefix("Token ")'))

  const authResult = runCli(cliPath, ["--root", root, "--output", path.join(root, "manifest.json")])
  assert.equal(authResult.status, 1)
  assert.match(authResult.stderr, /unsupported RPC authentication declaration/u)

  await writeSourceFixture(root)
  const middlewarePath = path.join(root, "deepwell/src/middleware.rs")
  const middlewareSource = await fs.readFile(middlewarePath, "utf8")
  await fs.writeFile(middlewarePath, middlewareSource.replace("X-Deepwell-Site-Id", "X-Alternate-Site-Id"))

  const contextResult = runCli(cliPath, ["--root", root, "--output", path.join(root, "manifest.json")])
  assert.equal(contextResult.status, 1)
  assert.match(contextResult.stderr, /unsupported request context header declaration/u)
})

test("Deepwell JSON-RPC generator rejects changed authorization aliases or duplicate-header behavior", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "deepwell-contract-auth-control-flow-"))
  await writeSourceFixture(root)
  const authPath = path.join(root, "deepwell/src/middleware/rpc_auth.rs")
  const authSource = await fs.readFile(authPath, "utf8")
  await fs.writeFile(authPath, authSource.replace("use http::header::{AUTHORIZATION, HeaderMap, WWW_AUTHENTICATE};", "use http::header::{HeaderMap, WWW_AUTHENTICATE};\nconst AUTHORIZATION: &str = \"X-Alternate-Authorization\";"))

  const aliasResult = runCli(cliPath, ["--root", root, "--output", path.join(root, "manifest.json")])
  assert.equal(aliasResult.status, 1)
  assert.match(aliasResult.stderr, /unsupported RPC authentication declaration/u)

  await writeSourceFixture(root)
  const duplicateSource = await fs.readFile(authPath, "utf8")
  await fs.writeFile(authPath, duplicateSource.replace("if values.next().is_some() {\n        return false;\n    }", "if values.next().is_some() {\n        return true;\n    }"))

  const duplicateResult = runCli(cliPath, ["--root", root, "--output", path.join(root, "manifest.json")])
  assert.equal(duplicateResult.status, 1)
  assert.match(duplicateResult.stderr, /unsupported RPC authentication declaration/u)
})
