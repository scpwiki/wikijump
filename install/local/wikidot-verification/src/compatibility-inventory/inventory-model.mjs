import { createHash } from "node:crypto"
import { phase, uniqueSortedStrings } from "./catalog-surfaces.mjs"

export const SEMANTICS_REGISTRY = "docs/development/compatibility-surface-semantics.json"
const SUPPORTED_RELATIONSHIP_EDGE_TYPES = new Set([
  "alias", "equivalence", "implemented_by", "parsed_by", "rendered_by", "tested_by"
])
const AUDITED_CATALOG_FALLBACK = Object.freeze({
  count: 92,
  surface_ids_sha256: "08a6222562288778e1b0eb705a0281401424f514aeb542bedbabf37a0261a95e",
  mapping_sha256: "840583979a874ed9b9189b446f39d4932cf043c1e89235db1b443c2fd3572190"
})
const AUDITED_CURRENT_CATALOG_ISSUES = Object.freeze({
  count: 195,
  surface_ids_sha256: "3a2cf09c168b37eb4539336cca7807f06585ece006a3327c2b2093b327424377",
  mapping_sha256: "3b5f692daf4475775187ac9b5a452844b2330cbadb9ea94aa92ee9c3d9319c30",
  fallback_issue: 1387,
  fallback_count: 168,
  fallback_surface_ids_sha256: "a69ba8bc0a841c3092fb6cab2b1d2efcea8044b25ddb9dcad52669db66867820",
  fallback_mapping_sha256: "9a7489f1083ef9a114f95221997194afcb6eb1b957ad36f307e309a3a3668ff7"
})
const FTML_PUBLIC_PREVIEW_TEST_FEATURES = new Set([
  "syntax-bibliography",
  "syntax-block-formatting-elements",
  "syntax-block-quotes",
  "syntax-code-blocks",
  "syntax-date",
  "syntax-definition-lists",
  "syntax-footnotes",
  "syntax-headings",
  "syntax-horizontal-rules",
  "syntax-inline-formatting",
  "syntax-lists",
  "syntax-math",
  "syntax-notes",
  "syntax-paragraphs-and-newline",
  "syntax-table-of-contents",
  "syntax-tables",
  "syntax-text-size",
  "syntax-typography",
  "syntax-universal-escaping"
])
const FTML_PUBLIC_PREVIEW_TEST =
  "deepwell/tests/page.rs#documented_ftml_owned_syntax_has_public_preview_regressions"
export const DEFERRED_XMLRPC_CATALOG_FEATURES = new Set([
  "catalog-feature:api-categories-select",
  "catalog-feature:api-deleted-methods",
  "catalog-feature:api-files-get-meta",
  "catalog-feature:api-files-get-one",
  "catalog-feature:api-files-save-one",
  "catalog-feature:api-files-select",
  "catalog-feature:api-overview",
  "catalog-feature:api-pages-get-meta",
  "catalog-feature:api-pages-get-one",
  "catalog-feature:api-pages-save-one",
  "catalog-feature:api-pages-select",
  "catalog-feature:api-posts-get",
  "catalog-feature:api-posts-select",
  "catalog-feature:api-tags-select",
  "catalog-feature:api-users-get-me"
])
const FRAMERAIL_ROUTE_ISSUE_EXCEPTIONS = new Map([
  ["framerail-route:/feed/forum/{feed}", 2091],
  ["framerail-route:/feed/site-changes.xml", 2087],
  ["framerail-route:/local--favicon/{filename}", 756],
  ["framerail-route:/printer--friendly/{*path}", 777],
  ["framerail-route:/forum/c-{category}/{*name}", 1034],
  ["framerail-route:/forum/start/{*extra}", 1034],
  ["framerail-route:/forum/t-{thread}/{*name}", 1034],
  ["framerail-route:/forum/{fallback}/{*extra}", 1034]
])
const PAGE_ACTION_ISSUES = new Map([
  ["page-action:backlinks", 1027],
  ["page-action:delete", 1373],
  ["page-action:discuss", 839],
  ["page-action:edit", 775],
  ["page-action:edit-append", 1041],
  ["page-action:edit-meta", 1373],
  ["page-action:edit-sections", 1041],
  ["page-action:file-delete", 1039],
  ["page-action:file-edit", 1039],
  ["page-action:file-history", 1039],
  ["page-action:file-info", 1039],
  ["page-action:file-move", 1039],
  ["page-action:file-upload", 1062],
  ["page-action:files", 1039],
  ["page-action:history", 1063],
  ["page-action:lock", 1373],
  ["page-action:more-options", 1041],
  ["page-action:parent", 1063],
  ["page-action:print", 777],
  ["page-action:rate", 1030],
  ["page-action:rename-move", 1373],
  ["page-action:site-tools", 1041],
  ["page-action:tags", 1041],
  ["page-action:view-source", 1041],
  ["page-action:watchers", 1032]
])
const CATALOG_FEATURE_ISSUE_EXCEPTIONS = new Map([
  ["catalog-feature:avatars", 1392],
  ["catalog-feature:community-site-directory", 1508],
  ["catalog-feature:data-forms-deleting-form", 1391],
  ["catalog-feature:data-forms-file-field", 1504],
  ["catalog-feature:data-forms-images", 1504],
  ["catalog-feature:forum-signatures", 1506],
  ["catalog-feature:gravatar", 1507],
  ["catalog-feature:module-createaccount", 1505],
  ["catalog-feature:module-comments", 1034],
  ["catalog-feature:module-deleteaccount", 1505],
  ["catalog-feature:module-forumcategory", 1034],
  ["catalog-feature:module-forumnewthread", 1034],
  ["catalog-feature:module-forumstart", 1034],
  ["catalog-feature:module-forumthread", 1034],
  ["catalog-feature:module-frontforum", 1034],
  ["catalog-feature:module-frontspecialmini", 1389],
  ["catalog-feature:module-listpages", 1383],
  ["catalog-feature:module-managesite", 1038],
  ["catalog-feature:module-members", 1032],
  ["catalog-feature:module-newsite", 1505],
  ["catalog-feature:outgoing-pingbacks", 1509],
  ["catalog-feature:module-petitionadmin", 1038],
  ["catalog-feature:module-recentposts", 1034],
  ["catalog-feature:module-recentthreads", 1034],
  ["catalog-feature:module-sitechanges", 1035],
  ["catalog-feature:module-sitestagcloud", 1508],
  ["catalog-feature:web-statistics", 1510],
])

export const PHASE_STATUSES = {
  evidence: new Set(["available", "partial", "missing", "blocked"]),
  source: new Set(["implemented", "in_progress", "pending", "blocked"]),
  candidate: new Set(["passed", "failed", "pending", "blocked", "not_applicable"]),
  standing: new Set(["passed", "failed", "pending", "blocked", "not_applicable"]),
  closure: new Set(["closed", "open", "blocked"])
}

export function assertCanonicalStrings(values, context) {
  if (!Array.isArray(values) || JSON.stringify(values) !== JSON.stringify(uniqueSortedStrings(values))) {
    throw new Error(`${context} must be a sorted unique string array`)
  }
}


function auditedLinesSha256(lines) {
  return createHash("sha256").update(`${[...lines].sort().join("\n")}\n`).digest("hex")
}

export function normalizeSurfaceOwners(surfaces, catalogCrosswalk, semantics, auditedOwnershipActive) {
  const specificationKinds = uniqueSortedStrings(
    surfaces
      .filter(({ kind }) => kind !== "catalog_feature" && kind !== "open43_audit_case")
      .map(({ kind }) => kind)
  )
  const mappedSpecificationKinds = Object.keys(semantics.specification_owner_by_kind ?? {}).sort()
  if (JSON.stringify(mappedSpecificationKinds) !== JSON.stringify(specificationKinds)) {
    throw new Error(`${SEMANTICS_REGISTRY} has missing or unused specification owner kinds`)
  }
  const legacyOwners = uniqueSortedStrings(surfaces.map(({ public_owner: owner }) => owner))
  const mappedLegacyOwners = Object.keys(semantics.implementation_owners_by_legacy_owner ?? {}).sort()
  if (JSON.stringify(mappedLegacyOwners) !== JSON.stringify(legacyOwners)) {
    throw new Error(`${SEMANTICS_REGISTRY} has missing or unused legacy owner keys`)
  }
  const declaredImplementationOwners = new Set(semantics.implementation_owner_keys)
  for (const [legacyOwner, owners] of Object.entries(semantics.implementation_owners_by_legacy_owner)) {
    assertCanonicalStrings(owners, `${SEMANTICS_REGISTRY} legacy owner ${legacyOwner}`)
    if (owners.some((owner) => !declaredImplementationOwners.has(owner))) {
      throw new Error(`${SEMANTICS_REGISTRY} legacy owner ${legacyOwner} has an undeclared owner`)
    }
  }
  const crosswalkByFeature = new Map(
    catalogCrosswalk.map((row) => [row.feature_id, row])
  )
  const implementationOwnersFromSource = (record) => {
    const owners = new Set()
    for (const reference of record.source?.references ?? []) {
      if (reference.startsWith("deepwell/")) owners.add("wikijump.deepwell")
      else if (reference.startsWith("framerail/")) owners.add("wikijump.framerail")
      else if (reference.startsWith("wws/")) owners.add("wikijump.wws")
    }
    return uniqueSortedStrings([...owners])
  }
  const normalizedSurfaces = surfaces.map((record) => {
    const {
      public_owner: legacyOwner,
      implementation_owner_records: implementationOwnerRecords,
      ...normalized
    } = record
    let specificationOwner
    let implementationOwners
    if (record.kind === "catalog_feature") {
      const featureId = record.surface_id.slice("catalog-feature:".length)
      const crosswalk = crosswalkByFeature.get(featureId)
      specificationOwner = `catalog.feature:${featureId}`
      if (crosswalk) {
        implementationOwners = uniqueSortedStrings(["ftml", crosswalk.runtime_owner])
      } else if (DEFERRED_XMLRPC_CATALOG_FEATURES.has(record.surface_id)) {
        implementationOwners = []
      } else if (implementationOwnerRecords.length > 0) {
        implementationOwners = uniqueSortedStrings(
          implementationOwnerRecords.map(({ owner }) => owner)
        )
      } else if (auditedOwnershipActive) {
        implementationOwners = implementationOwnersFromSource(record)
        if (implementationOwners.length === 0) {
          throw new Error(`audited catalog feature has no concrete implementation owner: ${record.surface_id}`)
        }
      } else {
        implementationOwners = []
      }
    } else if (record.kind === "open43_audit_case") {
      const caseId = record.surface_id.slice("open43-audit-case:".length)
      specificationOwner = `open43.case:${caseId}`
      implementationOwners = semantics.implementation_owners_by_legacy_owner?.[legacyOwner]
    } else {
      specificationOwner = semantics.specification_owner_by_kind?.[record.kind]
      implementationOwners = semantics.implementation_owners_by_legacy_owner?.[legacyOwner]
    }
    if (!specificationOwner) {
      throw new Error(`unknown specification owner kind for ${record.surface_id}: ${record.kind}`)
    }
    if (!implementationOwners) {
      throw new Error(`unknown implementation owner for ${record.surface_id}: ${legacyOwner}`)
    }
    return {
      ...normalized,
      specification_owner: specificationOwner,
      implementation_owners: uniqueSortedStrings(implementationOwners)
    }
  })
  if (auditedOwnershipActive) {
    const fallbackRows = surfaces.filter((record) => {
      if (record.kind !== "catalog_feature") return false
      const featureId = record.surface_id.slice("catalog-feature:".length)
      return !crosswalkByFeature.has(featureId) &&
        !DEFERRED_XMLRPC_CATALOG_FEATURES.has(record.surface_id) &&
        record.implementation_owner_records.length === 0
    })
    const fallbackIds = fallbackRows.map(({ surface_id: surfaceId }) => surfaceId)
    const fallbackMapping = normalizedSurfaces
      .filter(({ surface_id: surfaceId }) => fallbackIds.includes(surfaceId))
      .map(({ surface_id: surfaceId, implementation_owners: owners }) => `${surfaceId}\t${owners.join(",")}`)
    if (
      fallbackRows.length !== AUDITED_CATALOG_FALLBACK.count ||
      auditedLinesSha256(fallbackIds) !== AUDITED_CATALOG_FALLBACK.surface_ids_sha256 ||
      auditedLinesSha256(fallbackMapping) !== AUDITED_CATALOG_FALLBACK.mapping_sha256
    ) {
      throw new Error(
        `audited catalog implementation ownership drift: count=${fallbackRows.length} ` +
          `surface_ids_sha256=${auditedLinesSha256(fallbackIds)} ` +
          `mapping_sha256=${auditedLinesSha256(fallbackMapping)}`
      )
    }
  }
  return normalizedSurfaces
}

export function applyFtmlCatalogSourceProjection(surfaces, ftmlRawSurfaceManifest) {
  const rawById = new Map(
    ftmlRawSurfaceManifest.records.map((record) => [record.surface_id, record])
  )
  const pureFtmlByFeature = new Map(
    ftmlRawSurfaceManifest.catalog_crosswalk
      .filter(({ runtime_owner: runtimeOwner }) => runtimeOwner === null)
      .map((row) => [row.feature_id, row])
  )
  const revision = ftmlRawSurfaceManifest.source?.commit
  if (!/^[0-9a-f]{40}$/u.test(revision ?? "")) {
    throw new Error("pinned FTML catalog source projection has no exact revision")
  }
  return surfaces.map((record) => {
    if (record.kind !== "catalog_feature") return record
    const featureId = record.surface_id.slice("catalog-feature:".length)
    const crosswalk = pureFtmlByFeature.get(featureId)
    if (!crosswalk) return record
    const ownerIds = uniqueSortedStrings([
      ...crosswalk.parsed_by,
      ...crosswalk.rendered_by
    ])
    if (ownerIds.length === 0) {
      throw new Error(`pure FTML catalog feature has no implementation source: ${featureId}`)
    }
    const references = ownerIds.map((surfaceId) => {
      const owner = rawById.get(surfaceId)
      if (!owner || typeof owner.source_reference !== "string" || owner.source_reference === "") {
        throw new Error(`pure FTML catalog feature has an unknown source owner: ${featureId} -> ${surfaceId}`)
      }
      return `Rokurolize/ftml@${revision}:${owner.source_reference}`
    })
    const tests = FTML_PUBLIC_PREVIEW_TEST_FEATURES.has(featureId)
      ? uniqueSortedStrings([
          ...(record.existing_refs?.tests ?? []),
          FTML_PUBLIC_PREVIEW_TEST
        ])
      : record.existing_refs?.tests ?? []
    return {
      ...record,
      existing_refs: {
        ...record.existing_refs,
        tests
      },
      source: phase("implemented", references)
    }
  })
}

function framerailAmcModuleIssue(surfaceId) {
  const moduleName = surfaceId.slice("framerail-amc-module:".length).split(":")[0]
  if (moduleName === "pagerate/WhoRatedPageModule") return 1030
  if (moduleName === "membership/MembersListModule") return 1032
  if (moduleName.startsWith("forum/")) return 1034
  if (moduleName === "changes/SiteChangesListModule") return 1035
  if (moduleName === "files/PageFilesModule") return 1039
  if (moduleName === "viewsource/ViewSourceModule") return 1041
  if (moduleName.startsWith("history/")) return 1063
  if (moduleName === "list/ListPagesModule") return 1374
  throw new Error(`missing audited Framerail AMC module issue: ${surfaceId}`)
}

function auditedIssueForSurface(record) {
  switch (record.kind) {
    case "catalog_feature":
      if (DEFERRED_XMLRPC_CATALOG_FEATURES.has(record.surface_id)) return null
      return CATALOG_FEATURE_ISSUE_EXCEPTIONS.get(record.surface_id) ??
        AUDITED_CURRENT_CATALOG_ISSUES.fallback_issue
    case "deepwell_jsonrpc_method":
      return 1368
    case "wws_route":
      return record.surface_id.includes("/local--html/{page_slug}/{id}/{domain}") ? 1370 : 1369
    case "wikidot_py_amc_module_shape":
      return 1376
    case "framerail_xmlrpc_method":
      return 1375
    case "framerail_route":
      return FRAMERAIL_ROUTE_ISSUE_EXCEPTIONS.get(record.surface_id) ?? 1372
    case "framerail_server_action":
      return record.surface_id === "framerail-server-action:/-/settings?/display" ? 1063 : 1372
    case "framerail_amc_action_shape":
      if (record.surface_id === "framerail-amc-action:ForumAction:createPageDiscussionThread") return 839
      if (record.surface_id === "framerail-amc-action:misc/NewPageHelperAction:createNewPage") return 1371
      throw new Error(`missing audited Framerail AMC action issue: ${record.surface_id}`)
    case "framerail_amc_module_shape":
      return framerailAmcModuleIssue(record.surface_id)
    case "page_action": {
      const issue = PAGE_ACTION_ISSUES.get(record.surface_id)
      if (!issue) throw new Error(`missing audited page action issue: ${record.surface_id}`)
      return issue
    }
    default:
      return null
  }
}

export function applyAuditedIssueOwnership(surfaces, auditedOwnershipActive) {
  if (!auditedOwnershipActive) return surfaces
  const assigned = surfaces.map((record) => {
    const issue = auditedIssueForSurface(record)
    if (issue === null) return record
    const existing = record.existing_refs.issues
    if (existing.length > 0 && (existing.length !== 1 || existing[0] !== issue)) {
      throw new Error(`audited issue conflicts with existing issue for ${record.surface_id}`)
    }
    return {
      ...record,
      existing_refs: {
        ...record.existing_refs,
        issues: [issue]
      }
    }
  })
  // Every surface of an audited kind is assigned (or rejected) by
  // auditedIssueForSurface above, and the generated inventory is verified
  // byte-for-byte, so pinned per-kind counts and ID hashes would only force a
  // hand re-pin whenever a surface is added.
  const currentCatalogRows = assigned.filter((record) =>
    record.kind === "catalog_feature" && !DEFERRED_XMLRPC_CATALOG_FEATURES.has(record.surface_id)
  )
  const currentCatalogIds = currentCatalogRows.map(({ surface_id: surfaceId }) => surfaceId)
  const currentCatalogMapping = currentCatalogRows.map(
    ({ surface_id: surfaceId, existing_refs: existingRefs }) =>
      `${surfaceId}\t${existingRefs.issues.join(",")}`
  )
  const fallbackCatalogRows = currentCatalogRows.filter(
    ({ surface_id: surfaceId }) => !CATALOG_FEATURE_ISSUE_EXCEPTIONS.has(surfaceId)
  )
  const fallbackCatalogIds = fallbackCatalogRows.map(({ surface_id: surfaceId }) => surfaceId)
  const fallbackCatalogMapping = fallbackCatalogRows.map(
    ({ surface_id: surfaceId, existing_refs: existingRefs }) =>
      `${surfaceId}\t${existingRefs.issues.join(",")}`
  )
  if (
    currentCatalogRows.length !== AUDITED_CURRENT_CATALOG_ISSUES.count ||
    auditedLinesSha256(currentCatalogIds) !== AUDITED_CURRENT_CATALOG_ISSUES.surface_ids_sha256 ||
    auditedLinesSha256(currentCatalogMapping) !== AUDITED_CURRENT_CATALOG_ISSUES.mapping_sha256 ||
    fallbackCatalogRows.length !== AUDITED_CURRENT_CATALOG_ISSUES.fallback_count ||
    auditedLinesSha256(fallbackCatalogIds) !==
      AUDITED_CURRENT_CATALOG_ISSUES.fallback_surface_ids_sha256 ||
    auditedLinesSha256(fallbackCatalogMapping) !==
      AUDITED_CURRENT_CATALOG_ISSUES.fallback_mapping_sha256
  ) {
    throw new Error("audited current catalog issue ownership drift")
  }
  return assigned
}

export function buildRelationshipModel(surfaces, ftmlRawSurfaceManifest, semantics) {
  const publicIds = new Set(surfaces.map(({ surface_id: surfaceId }) => surfaceId))
  if (publicIds.size !== surfaces.length) {
    const seen = new Set()
    const duplicate = surfaces.find(({ surface_id: surfaceId }) => {
      if (seen.has(surfaceId)) return true
      seen.add(surfaceId)
      return false
    })
    throw new Error(`duplicate surface_id: ${duplicate.surface_id}`)
  }
  const rawIds = new Set(
    ftmlRawSurfaceManifest.records.map(({ surface_id: surfaceId }) => surfaceId)
  )
  for (const surfaceId of rawIds) {
    if (publicIds.has(surfaceId)) throw new Error(`FTML raw surface is double-counted: ${surfaceId}`)
  }

  const specificationOwners = semantics.specification_owner_keys
  const implementationOwners = semantics.implementation_owner_keys
  for (const [name, owners] of [
    ["specification", specificationOwners],
    ["implementation", implementationOwners]
  ]) {
    if (!Array.isArray(owners) || new Set(owners).size !== owners.length) {
      throw new Error(`${SEMANTICS_REGISTRY} has missing or duplicate ${name} owner keys`)
    }
  }
  const usedSpecificationOwners = uniqueSortedStrings(
    surfaces.map(({ specification_owner: owner }) => owner)
  )
  const usedImplementationOwners = uniqueSortedStrings(
    surfaces.flatMap(({ implementation_owners: owners }) => owners)
  )
  if (JSON.stringify(specificationOwners) !== JSON.stringify(usedSpecificationOwners)) {
    throw new Error(`${SEMANTICS_REGISTRY} has missing or unused specification owner keys`)
  }
  if (JSON.stringify(implementationOwners) !== JSON.stringify(usedImplementationOwners)) {
    throw new Error(
      `${SEMANTICS_REGISTRY} has missing or unused implementation owner keys: expected ${JSON.stringify(implementationOwners)}, discovered ${JSON.stringify(usedImplementationOwners)}`
    )
  }
  const implementationOwnerSet = new Set(implementationOwners)
  const edges = []
  for (const record of surfaces) {
    for (const owner of record.implementation_owners) {
      edges.push({ source: record.surface_id, type: "implemented_by", target: owner })
    }
  }
  for (const row of ftmlRawSurfaceManifest.catalog_crosswalk) {
    const source = `catalog-feature:${row.feature_id}`
    if (!publicIds.has(source)) throw new Error(`FTML crosswalk has unknown catalog feature: ${source}`)
    for (const [field, type] of [
      ["parsed_by", "parsed_by"],
      ["rendered_by", "rendered_by"],
      ["tested_by", "tested_by"]
    ]) {
      for (const target of row[field]) edges.push({ source, type, target })
    }
  }
  for (const record of ftmlRawSurfaceManifest.records) {
    if (record.kind === "block_alias") {
      edges.push({ source: record.surface_id, type: "alias", target: record.canonical_surface })
    }
  }

  const edgeKeys = new Set()
  const edgeTypes = semantics.relationship_edge_types
  if (
    !Array.isArray(edgeTypes) ||
    new Set(edgeTypes).size !== edgeTypes.length ||
    edgeTypes.some((type) => !SUPPORTED_RELATIONSHIP_EDGE_TYPES.has(type)) ||
    [...SUPPORTED_RELATIONSHIP_EDGE_TYPES].some((type) => !edgeTypes.includes(type))
  ) {
    throw new Error(`${SEMANTICS_REGISTRY} has missing, duplicate, or unknown relationship edge types`)
  }
  const edgeTypeSet = new Set(edgeTypes)
  for (const edge of edges) {
    const key = `${edge.source}\u0000${edge.type}\u0000${edge.target}`
    if (edgeKeys.has(key)) throw new Error(`duplicate relationship edge: ${key}`)
    edgeKeys.add(key)
    if (!edgeTypeSet.has(edge.type)) throw new Error(`unknown relationship edge type: ${edge.type}`)
    if (!publicIds.has(edge.source) && !rawIds.has(edge.source)) {
      throw new Error(`relationship edge has unknown source: ${edge.source}`)
    }
    if (edge.type === "implemented_by") {
      if (!implementationOwnerSet.has(edge.target)) {
        throw new Error(`relationship edge has unknown implementation owner: ${edge.target}`)
      }
    } else if (!rawIds.has(edge.target)) {
      throw new Error(`relationship edge has unknown FTML target: ${edge.target}`)
    }
  }
  return {
    owner_keys: {
      specification: specificationOwners,
      implementation: implementationOwners
    },
    relationship_edges: edges.sort((left, right) =>
      `${left.source}\u0000${left.type}\u0000${left.target}`.localeCompare(
        `${right.source}\u0000${right.type}\u0000${right.target}`,
        "en"
      )
    )
  }
}

export function validateInventory(surfaces, ownerKeys) {
  const identifiers = new Set()
  const specificationOwners = new Set(ownerKeys.specification)
  const implementationOwners = new Set(ownerKeys.implementation)
  for (const record of surfaces) {
    if (typeof record.surface_id !== "string" || record.surface_id === "") {
      throw new Error("compatibility surface is missing surface_id")
    }
    if (identifiers.has(record.surface_id)) {
      throw new Error(`duplicate surface_id: ${record.surface_id}`)
    }
    identifiers.add(record.surface_id)
    if (!specificationOwners.has(record.specification_owner)) {
      throw new Error(`unknown specification owner for ${record.surface_id}`)
    }
    if (
      !Array.isArray(record.implementation_owners) ||
      record.implementation_owners.some((owner) => !implementationOwners.has(owner))
    ) {
      throw new Error(`unknown implementation owner for ${record.surface_id}`)
    }
    if (!Array.isArray(record.public_reference) || record.public_reference.length === 0) {
      throw new Error(`missing public reference for ${record.surface_id}`)
    }
    for (const field of Object.keys(PHASE_STATUSES)) {
      const status = record[field]?.status
      if (!PHASE_STATUSES[field].has(status)) {
        throw new Error(`unknown ${field} status for ${record.surface_id}: ${status}`)
      }
    }
  }
}

