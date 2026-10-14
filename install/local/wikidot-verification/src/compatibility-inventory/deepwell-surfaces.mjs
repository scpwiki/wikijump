import path from "node:path"
import { LEDGER_STATUSES, phase, surface, uniqueSortedStrings } from "./catalog-surfaces.mjs"
import { PHASE_STATUSES } from "./inventory-model.mjs"
import { extractBalanced, maskTypeScriptCommentsAndLiterals, objectPropertyNames } from "./typescript-source.mjs"

const MISSING_PAGE_CONTROL_CONTRACTS = new Map([
  [
    "create",
    {
      operations: ["edit"],
      states: [
        "missing-page-settled",
        "editor-loading",
        "editor-settled",
        "save-loading",
        "save-success",
        "save-denial",
        "save-failure",
        "created-page-settled"
      ]
    }
  ],
  [
    "restore",
    {
      operations: ["deletedGet", "restore"],
      states: [
        "missing-page-settled",
        "deleted-selection-loading",
        "deleted-selection-settled",
        "deleted-selection-denial",
        "deleted-selection-failure",
        "restore-loading",
        "restore-success",
        "restore-denial",
        "restore-failure",
        "restored-page-settled"
      ]
    }
  ]
])

export async function discoverDeepwellJsonRpc(root, helpers) {
  const { readJson, readText, sha256, isCanonicalRepositoryReference } = helpers
  const registryPath = "deepwell/src/api.rs"
  const manifestPath = "docs/development/deepwell-jsonrpc-contract-manifest.json"
  const [sourceText, manifest] = await Promise.all([
    readText(root, registryPath),
    readJson(root, manifestPath)
  ])
  const methods = [...sourceText.matchAll(/register!\s*\(\s*"([^"]+)"\s*,/gu)].map(
    (match) => match[1]
  )
  if (methods.length === 0) throw new Error(`${registryPath} declares no JSON-RPC methods`)
  if (new Set(methods).size !== methods.length) {
    const seen = new Set()
    const duplicate = methods.find((method) => {
      if (seen.has(method)) return true
      seen.add(method)
      return false
    })
    throw new Error(`duplicate surface_id: deepwell-jsonrpc:${duplicate}`)
  }
  if (
    manifest.schema !== "wikijump.deepwell_jsonrpc_contract_manifest.v1" ||
    manifest.method_count !== methods.length ||
    !Array.isArray(manifest.methods) ||
    manifest.methods.length !== methods.length
  ) {
    throw new Error(`${manifestPath} does not match the Deepwell method denominator`)
  }
  if (
    manifest.source_identities?.jsonrpc_registry?.path !== registryPath ||
    manifest.source_identities.jsonrpc_registry.sha256 !== sha256(sourceText)
  ) {
    throw new Error(`${manifestPath} JSON-RPC registry identity drift`)
  }
  const contractByMethod = new Map()
  for (const contract of manifest.methods) {
    if (!contract || typeof contract.method !== "string" || contract.method === "") {
      throw new Error(`${manifestPath} contains a method without an identity`)
    }
    if (contractByMethod.has(contract.method)) {
      throw new Error(`${manifestPath} contains duplicate method ${contract.method}`)
    }
    contractByMethod.set(contract.method, contract)
  }
  if (
    contractByMethod.size !== methods.length ||
    methods.some((method) => !contractByMethod.has(method))
  ) {
    throw new Error(`${manifestPath} method identities do not match ${registryPath}`)
  }
  const sourceCache = new Map()
  const readOwnerSource = async (sourceReference) => {
    const sourcePath = sourceReference.split("#", 1)[0]
    if (!sourceCache.has(sourcePath)) sourceCache.set(sourcePath, await readText(root, sourcePath))
    return sourceCache.get(sourcePath)
  }
  const records = []
  for (const method of methods) {
    const contract = contractByMethod.get(method)
    const owner = contract.endpoint_owner
    if (
      owner?.component !== "deepwell" ||
      typeof owner.source !== "string" ||
      !isCanonicalRepositoryReference(owner.source) ||
      !/^[0-9a-f]{64}$/u.test(owner.source_sha256 ?? "")
    ) {
      throw new Error(`${manifestPath} has invalid endpoint ownership for ${method}`)
    }
    if (sha256(await readOwnerSource(owner.source)) !== owner.source_sha256) {
      throw new Error(`${manifestPath} endpoint source identity drift for ${method}`)
    }
    const witness = contract.test_witness
    if (
      !witness ||
      !["source_contract_only", "endpoint_behavioral", "rpc_behavioral"].includes(witness.kind) ||
      typeof witness.reference !== "string" ||
      witness.reference === ""
    ) {
      throw new Error(`${manifestPath} has invalid test witness for ${method}`)
    }
    const behavioralTests =
      witness.kind === "source_contract_only"
        ? [
            "install/local/wikidot-verification/tests/deepwell-jsonrpc-contract-manifest.test.mjs#Deepwell JSON-RPC manifest exactly covers the current registered contract"
          ]
        : [witness.reference]
    records.push(surface({
      surfaceId: `deepwell-jsonrpc:${method}`,
      kind: "deepwell_jsonrpc_method",
      publicOwner: "deepwell",
      publicReference: [`${registryPath}#register:${method}`, owner.source],
      tests: behavioralTests,
      evidence: phase("available", [manifestPath]),
      source: phase("implemented", [owner.source.split("#", 1)[0]])
    }))
  }
  return records
}

async function declaredPageActions(root, sourcePath, readText) {
  const sourceText = await readText(root, sourcePath)
  const lexicalSource = maskTypeScriptCommentsAndLiterals(sourceText, sourcePath)
  const candidates = [...lexicalSource.matchAll(/\bexport\s+const\s+pageActions\s*=\s*/gu)]
  const declarations = candidates.filter((candidate) => {
    const lineStart = lexicalSource.lastIndexOf("\n", candidate.index) + 1
    return /^[ \t]*$/u.test(lexicalSource.slice(lineStart, candidate.index))
  })
  if (candidates.length > 1) throw new Error(`${sourcePath} has duplicate exported pageActions declarations`)
  if (declarations.length !== candidates.length) throw new Error(`${sourcePath} has declaration-shaped text outside a supported top-level declaration`)
  const [declaration] = declarations
  if (!declaration) throw new Error(`${sourcePath} has no exported pageActions declaration`)
  const expressionStart = declaration.index + declaration[0].length
  if (lexicalSource[expressionStart] !== "{") {
    throw new Error(`${sourcePath}#pageActions is not an object literal`)
  }
  const lexicalExpression = extractBalanced(lexicalSource, expressionStart, "{", "}")
  const expression = sourceText.slice(expressionStart, expressionStart + lexicalExpression.length)
  const names = objectPropertyNames(expression, `${sourcePath}#pageActions`)
  if (new Set(names).size !== names.length) {
    throw new Error(`${sourcePath}#pageActions contains duplicate declarations`)
  }
  return new Set(names)
}

export async function discoverPageActionSurfaces(root, helpers) {
  const { readJson, readText, sha256, isCanonicalRepositoryReference, toPosix, validatedBrowserIntervalProof } = helpers
  const registryPath = "docs/development/wikidot-page-action-surfaces.json"
  const registry = await readJson(root, registryPath)
  if (registry.schema !== "wikijump.wikidot_page_action_surface_registry.v2") {
    throw new Error(`${registryPath} has an unsupported schema`)
  }
  if (!Array.isArray(registry.evidence_references) || registry.evidence_references.length === 0) {
    throw new Error(`${registryPath} must declare evidence_references`)
  }
  if (!Array.isArray(registry.surfaces) || registry.surfaces.length === 0) {
    throw new Error(`${registryPath} must declare surfaces`)
  }
  if (!Array.isArray(registry.missing_page_controls)) {
    throw new Error(`${registryPath} must declare missing_page_controls`)
  }
  const pageActions = registry.surfaces.map((entry) => {
    if (!entry || !/^[a-z][a-z0-9-]+$/u.test(entry.action_id ?? "")) {
      throw new Error(`${registryPath} contains an invalid action_id`)
    }
    if (!LEDGER_STATUSES.has(entry.source_status)) {
      throw new Error(`${registryPath} has an unknown source status for ${entry.action_id}`)
    }
    const standingStatus = entry.standing_status ?? "pending"
    if (!PHASE_STATUSES.standing.has(standingStatus)) {
      throw new Error(`${registryPath} has an unknown standing status for ${entry.action_id}`)
    }
    return surface({
      surfaceId: `page-action:${entry.action_id}`,
      kind: "page_action",
      publicOwner: "framerail",
      publicReference: [registryPath, ...(entry.public_references ?? [])],
      issues: entry.issues ?? [],
      tests: entry.test_references ?? [],
      evidence: phase("partial", registry.evidence_references),
      source: phase(entry.source_status, entry.public_references ?? []),
      standing: phase(standingStatus)
    })
  })
  const controlIds = registry.missing_page_controls.map((entry) => entry?.control_id)
  if (
    new Set(controlIds).size !== controlIds.length ||
    JSON.stringify([...controlIds].sort()) !==
      JSON.stringify([...MISSING_PAGE_CONTROL_CONTRACTS.keys()].sort())
  ) {
    throw new Error(`${registryPath} must declare exactly one create and one restore control`)
  }
  const missingPageControls = []
  const pageActionDeclarations = new Map()
  for (const entry of registry.missing_page_controls) {
    const contract = MISSING_PAGE_CONTROL_CONTRACTS.get(entry.control_id)
    if (!LEDGER_STATUSES.has(entry.source_status)) {
      throw new Error(
        `${registryPath} has an unknown source status for missing-page ${entry.control_id}`
      )
    }
    if (!Array.isArray(entry.operation_bindings)) {
      throw new Error(`${registryPath} ${entry.control_id} operation_bindings must be an array`)
    }
    const operationIds = entry.operation_bindings.map((binding) => binding?.operation_id)
    if (
      new Set(operationIds).size !== operationIds.length ||
      JSON.stringify(operationIds) !== JSON.stringify(contract.operations)
    ) {
      throw new Error(
        `${registryPath} ${entry.control_id} operations must be ${contract.operations.join(",")}`
      )
    }
    const operationBindings = entry.operation_bindings.map((binding) => {
      if (
        !Array.isArray(binding.public_references) ||
        binding.public_references.length === 0 ||
        binding.public_references.some(
          (reference) => {
            if (typeof reference !== "string") return true
            const parts = reference.split("#")
            return (
              parts.length !== 2 ||
              !isCanonicalRepositoryReference(reference) ||
              parts[1] !== `action:${binding.operation_id}`
            )
          }
        )
      ) {
        throw new Error(
          `${registryPath} ${entry.control_id} ${binding.operation_id} has invalid public references`
        )
      }
      return {
        operation_id: binding.operation_id,
        public_references: uniqueSortedStrings(binding.public_references)
      }
    })
    if (JSON.stringify(entry.observable_states) !== JSON.stringify(contract.states)) {
      throw new Error(
        `${registryPath} ${entry.control_id} observable_states do not match the closed contract`
      )
    }
    const proof = validatedBrowserIntervalProof(
      entry.browser_interval_proof,
      registryPath,
      entry.control_id
    )
    if (!Array.isArray(entry.source_identities) || entry.source_identities.length === 0) {
      throw new Error(`${registryPath} ${entry.control_id} must declare source_identities`)
    }
    const identityPaths = new Set()
    const sourceIdentities = []
    for (const identity of entry.source_identities) {
      const normalizedIdentityPath =
        typeof identity?.path === "string" ? toPosix(path.normalize(identity.path)) : ""
      if (
        !identity ||
        typeof identity.path !== "string" ||
        identity.path === "" ||
        path.isAbsolute(identity.path) ||
        normalizedIdentityPath !== identity.path ||
        normalizedIdentityPath === ".." ||
        normalizedIdentityPath.startsWith("../") ||
        Object.keys(identity).some((key) => key !== "path")
      ) {
        throw new Error(`${registryPath} ${entry.control_id} has an invalid source identity`)
      }
      if (identityPaths.has(identity.path)) {
        throw new Error(
          `${registryPath} ${entry.control_id} has duplicate source identity ${identity.path}`
        )
      }
      identityPaths.add(identity.path)
      // The registry names the owning sources; their bytes are bound here, at
      // the inventory's own source revision, so an edit needs no hand re-pin.
      const sourceText = await readText(root, identity.path)
      sourceIdentities.push({ path: identity.path, sha256: sha256(sourceText) })
    }
    for (const binding of operationBindings) {
      for (const reference of binding.public_references) {
        const sourcePath = reference.split("#", 1)[0]
        if (!identityPaths.has(sourcePath)) {
          throw new Error(
            `${registryPath} ${entry.control_id} operation reference lacks a source identity: ${sourcePath}`
          )
        }
        let declarations = pageActionDeclarations.get(sourcePath)
        if (!declarations) {
          declarations = await declaredPageActions(root, sourcePath, readText)
          pageActionDeclarations.set(sourcePath, declarations)
        }
        if (!declarations.has(binding.operation_id)) {
          throw new Error(
            `${registryPath} ${entry.control_id} ${binding.operation_id} is not declared by ${sourcePath}#pageActions`
          )
        }
      }
    }
    const base = surface({
      surfaceId: `missing-page-control:${entry.control_id}`,
      kind: "missing_page_control",
      publicOwner: "framerail",
      publicReference: [
        registryPath,
        ...sourceIdentities.map(({ path: sourcePath }) => sourcePath),
        ...operationBindings.flatMap(({ public_references: references }) => references)
      ],
      issues: entry.issues ?? [],
      tests: entry.test_references ?? [],
      evidence: phase(
        proof.status,
        proof.status === "available" ? proof.references : []
      ),
      source: phase(entry.source_status, sourceIdentities.map(({ path: sourcePath }) => sourcePath))
    })
    missingPageControls.push({
      ...base,
      operation_bindings: operationBindings,
      observable_states: [...entry.observable_states],
      browser_interval_proof: proof,
      source_identities: sourceIdentities
    })
  }
  return [...pageActions, ...missingPageControls]
}

