import fs from "node:fs/promises"
import path from "node:path"
import { phase, uniqueSortedStrings } from "./catalog-surfaces.mjs"
import { DEFERRED_XMLRPC_CATALOG_FEATURES } from "./inventory-model.mjs"

export const CATALOG_SOURCE_ATTRIBUTION = "docs/development/compatibility-catalog-source-attribution.json"

export async function applyFramerailRouteActionEvidence(root, records, sourceRevision, helpers) {
  const { readJson, preloadPinnedRevisionTexts, isCanonicalRepositoryReference, gitRevisionContains } = helpers
  const registryPath = "docs/development/framerail-route-action-evidence.json"
  const registry = await readJson(root, registryPath)
  if (
    registry.schema !== "wikijump.framerail_route_action_evidence.v1" ||
    !/^[0-9a-f]{40}$/u.test(registry.source_revision ?? "") ||
    !Array.isArray(registry.records)
  ) {
    throw new Error(`${registryPath} has an invalid route/action evidence contract`)
  }
  const recordIds = records.map(({ surface_id: surfaceId }) => surfaceId).sort()
  const evidenceIds = registry.records.map(({ surface_id: surfaceId }) => surfaceId).sort()
  if (
    new Set(evidenceIds).size !== evidenceIds.length ||
    JSON.stringify(evidenceIds) !== JSON.stringify(recordIds)
  ) {
    const registered = new Set(evidenceIds)
    const discovered = new Set(recordIds)
    throw new Error(
      `${registryPath} does not exactly match the current route/action denominator ` +
        `(unregistered: ${recordIds.filter((id) => !registered.has(id)).join(", ") || "none"}; ` +
        `no longer discovered: ${evidenceIds.filter((id) => !discovered.has(id)).join(", ") || "none"})`
    )
  }
  const evidenceById = new Map(
    registry.records.map((record) => [record.surface_id, record])
  )
  preloadPinnedRevisionTexts(
    root,
    sourceRevision,
    registry.records.flatMap((record) =>
      Array.isArray(record.tests)
        ? record.tests.flatMap((reference) => {
            if (typeof reference !== "string") return []
            const separator = reference.indexOf("::")
            return separator > 0 ? [reference.slice(0, separator)] : []
          })
        : []
    )
  )
  let linked = 0
  let gaps = 0
  const projected = []
  for (const record of records) {
    const evidenceRecord = evidenceById.get(record.surface_id)
    if (
      !Array.isArray(evidenceRecord.tests) ||
      !Array.isArray(evidenceRecord.tracking) ||
      evidenceRecord.tracking.length === 0 ||
      !Array.isArray(evidenceRecord.temporal)
    ) {
      throw new Error(`${registryPath} has an invalid row for ${record.surface_id}`)
    }
    if (
      evidenceRecord.tracking.some(
        ({ issue }) => !Number.isSafeInteger(issue) || issue <= 0
      )
    ) {
      throw new Error(`${registryPath} has invalid tracking for ${record.surface_id}`)
    }
    const tests = []
    for (const reference of evidenceRecord.tests) {
      if (typeof reference !== "string") {
        throw new Error(`${registryPath} has an invalid test reference for ${record.surface_id}`)
      }
      const separator = reference.indexOf("::")
      if (separator <= 0 || reference.endsWith("::")) {
        throw new Error(`${registryPath} has an invalid test reference for ${record.surface_id}`)
      }
      const testPath = reference.slice(0, separator)
      const testName = reference.slice(separator + 2)
      if (
        !isCanonicalRepositoryReference(testPath) ||
        !gitRevisionContains(root, sourceRevision, testPath, testName)
      ) {
        throw new Error(`${registryPath} has a stale current test reference: ${reference}`)
      }
      tests.push(`${testPath}#${testName}`)
    }
    if (tests.length > 0) linked += 1
    else gaps += 1
    projected.push({
      ...record,
      existing_refs: {
        ...record.existing_refs,
        tests: uniqueSortedStrings(tests)
      }
    })
  }
  if (linked !== records.length || gaps !== 0) {
    throw new Error(`${registryPath} current test-link counts drifted`)
  }
  return projected
}

export async function applyWwsContractEvidence(root, records, sourceRevision, helpers) {
  const { readJson, isCanonicalRepositoryReference, sha256, gitBlobOid, resolveGitObject, readText } = helpers
  const denominatorPath = "docs/development/wws-route-registration-denominator.json"
  const denominator = await readJson(root, denominatorPath)
  if (
    denominator.schema !== "wikijump.wws_route_registration_denominator.v2" ||
    !Array.isArray(denominator.registrations) ||
    denominator.counts?.registrations !== denominator.registrations.length ||
    !Array.isArray(denominator.behavior_records)
  ) {
    throw new Error(`${denominatorPath} has an invalid denominator contract`)
  }
  if (!Array.isArray(denominator.source?.inputs) || denominator.source.inputs.length === 0) {
    throw new Error(`${denominatorPath} has no source identities`)
  }
  for (const input of denominator.source.inputs) {
    if (
      typeof input?.path !== "string" ||
      !isCanonicalRepositoryReference(input.path) ||
      !/^[0-9a-f]{40}$/u.test(input.git_blob ?? "") ||
      !/^[0-9a-f]{64}$/u.test(input.sha256 ?? "")
    ) {
      throw new Error(`${denominatorPath} has an invalid source identity`)
    }
    const sourceBytes = await fs.readFile(path.join(root, input.path))
    if (sha256(sourceBytes) !== input.sha256) {
      throw new Error(`${denominatorPath} source identity drift: ${input.path}`)
    }
    const sourceBlob = sourceRevision === null
      ? gitBlobOid(sourceBytes)
      : resolveGitObject(["-C", root], `${sourceRevision}:${input.path}`, "WWS source blob")
    if (sourceBlob !== input.git_blob) {
      throw new Error(`${denominatorPath} pinned source identity drift: ${input.path}`)
    }
  }
  if (
    typeof denominator.generator?.path !== "string" ||
    !/^[0-9a-f]{64}$/u.test(denominator.generator.sha256 ?? "") ||
    sha256(await readText(root, denominator.generator.path)) !== denominator.generator.sha256
  ) {
    throw new Error(`${denominatorPath} generator identity drift`)
  }
  if (
    typeof denominator.behavior_evidence?.path !== "string" ||
    !/^[0-9a-f]{64}$/u.test(denominator.behavior_evidence.sha256 ?? "") ||
    sha256(await readText(root, denominator.behavior_evidence.path)) !==
      denominator.behavior_evidence.sha256
  ) {
    throw new Error(`${denominatorPath} behavior evidence identity drift`)
  }

  const registrations = new Map()
  for (const registration of denominator.registrations) {
    const expectedId =
      `wws-route-registration:${registration.declared_method_class}:${registration.path}`
    if (
      registration.registration_id !== expectedId ||
      !["ANY", "GET"].includes(registration.declared_method_class) ||
      registrations.has(registration.registration_id)
    ) {
      throw new Error(`${denominatorPath} has an invalid or duplicate registration`)
    }
    registrations.set(registration.registration_id, registration)
  }
  const behaviorsByRegistration = new Map()
  for (const behavior of denominator.behavior_records) {
    if (
      typeof behavior?.id !== "string" ||
      !["implemented", "not_faithfully_mapped"].includes(behavior.status) ||
      !Array.isArray(behavior.registration_ids) ||
      typeof behavior.public_test !== "string" ||
      behavior.public_test === ""
    ) {
      throw new Error(`${denominatorPath} has an invalid behavior record`)
    }
    for (const registrationId of behavior.registration_ids) {
      if (!registrations.has(registrationId)) {
        throw new Error(`${denominatorPath} behavior references an unknown registration`)
      }
      const behaviors = behaviorsByRegistration.get(registrationId) ?? []
      behaviors.push(behavior)
      behaviorsByRegistration.set(registrationId, behaviors)
    }
    if (behavior.historical_evidence_receipt) {
      const receipt = await readText(root, behavior.historical_evidence_receipt)
      if (
        !/^[0-9a-f]{64}$/u.test(behavior.historical_evidence_sha256 ?? "") ||
        sha256(receipt) !== behavior.historical_evidence_sha256
      ) {
        throw new Error(`${denominatorPath} historical evidence identity drift for ${behavior.id}`)
      }
    }
  }

  const registrationForSurface = (surfaceId) => {
    const match = /^wws-route:(ANY|GET|HEAD|FALLBACK):(.*)$/u.exec(surfaceId)
    if (!match) throw new Error(`invalid WWS dispatch surface: ${surfaceId}`)
    const [, method, routePath] = match
    if (method === "ANY") return `wws-route-registration:ANY:${routePath}`
    if (method === "GET" || method === "HEAD") {
      return `wws-route-registration:GET:${routePath}`
    }
    const anyId = `wws-route-registration:ANY:${routePath}`
    return registrations.has(anyId)
      ? anyId
      : `wws-route-registration:GET:${routePath}`
  }

  const usedRegistrations = new Set()
  const projected = records.map((record) => {
    const registrationId = registrationForSurface(record.surface_id)
    if (!registrations.has(registrationId)) {
      throw new Error(`${denominatorPath} has no registration for ${record.surface_id}`)
    }
    usedRegistrations.add(registrationId)
    const behaviors = behaviorsByRegistration.get(registrationId) ?? []
    const partial = behaviors.some(({ status }) => status === "not_faithfully_mapped")
    const evidenceReferences = [
      denominatorPath,
      ...behaviors.flatMap((behavior) =>
        behavior.status === "implemented"
          ? [denominator.behavior_evidence.path]
          : [behavior.historical_evidence_receipt]
      )
    ]
    return {
      ...record,
      existing_refs: {
        ...record.existing_refs,
        tests: uniqueSortedStrings(
          behaviors.length > 0
            ? behaviors.map(({ public_test: publicTest }) => publicTest)
            : [
                "install/local/wikidot-verification/tests/wws-route-registration-denominator-cli.test.mjs#CLI writes the exact current 34-registration WWS denominator with source ownership"
              ]
        )
      },
      evidence: phase(partial ? "partial" : "available", evidenceReferences)
    }
  })
  if ([...registrations.keys()].some((registrationId) => !usedRegistrations.has(registrationId))) {
    throw new Error(`${denominatorPath} has a registration without a dispatch surface`)
  }
  return projected
}



export async function applyCatalogSourceAttribution(root, surfaces, sourceRevision, helpers) {
  const { readJson, preloadPinnedRevisionTexts, gitRevisionContains } = helpers
  const registry = await readJson(root, CATALOG_SOURCE_ATTRIBUTION)
  if (
    registry.schema !== "wikijump.compatibility_catalog_source_attribution.v1" ||
    !Array.isArray(registry.records)
  ) {
    throw new Error(`${CATALOG_SOURCE_ATTRIBUTION} has an invalid contract`)
  }
  const surfaceIds = new Set(
    surfaces
      .filter(({ kind }) => kind === "catalog_feature")
      .map(({ surface_id: surfaceId }) => surfaceId)
  )
  const recordIds = registry.records.map(({ surface_id: surfaceId }) => surfaceId)
  if (new Set(recordIds).size !== recordIds.length) {
    throw new Error(`${CATALOG_SOURCE_ATTRIBUTION} has duplicate surface ids`)
  }
  preloadPinnedRevisionTexts(
    root,
    sourceRevision,
    registry.records.flatMap((record) =>
      [...(Array.isArray(record.sources) ? record.sources : []), ...(Array.isArray(record.tests) ? record.tests : [])]
        .flatMap((witness) =>
          typeof witness?.path === "string" && witness.path !== "" && !path.isAbsolute(witness.path)
            ? [witness.path]
            : []
        )
    )
  )
  const verified = new Map()
  for (const record of registry.records) {
    if (
      typeof record.surface_id !== "string" ||
      !surfaceIds.has(record.surface_id) ||
      DEFERRED_XMLRPC_CATALOG_FEATURES.has(record.surface_id) ||
      !Array.isArray(record.sources) ||
      record.sources.length === 0 ||
      !Array.isArray(record.tests) ||
      record.tests.length === 0
    ) {
      throw new Error(`${CATALOG_SOURCE_ATTRIBUTION} has an invalid row for ${record.surface_id}`)
    }
    const verifyWitnesses = (witnesses, kind) => witnesses.map((witness) => {
      if (
        typeof witness?.path !== "string" ||
        witness.path === "" ||
        path.isAbsolute(witness.path) ||
        witness.path.split("/").includes("..") ||
        typeof witness.anchor !== "string" ||
        witness.anchor === ""
      ) {
        throw new Error(`${CATALOG_SOURCE_ATTRIBUTION} has invalid ${kind} witness for ${record.surface_id}`)
      }
      if (!gitRevisionContains(root, sourceRevision, witness.path, witness.anchor)) {
        throw new Error(
          `${CATALOG_SOURCE_ATTRIBUTION} ${kind} witness drifted for ${record.surface_id}: ${witness.path}#${witness.anchor}`
        )
      }
      return `${witness.path}#${witness.anchor}`
    })
    verified.set(record.surface_id, {
      sources: verifyWitnesses(record.sources, "source"),
      tests: verifyWitnesses(record.tests, "test")
    })
  }
  return surfaces.map((record) => {
    const attribution = verified.get(record.surface_id)
    if (!attribution) return record
    return {
      ...record,
      source: phase("implemented", [...record.source.references, ...attribution.sources]),
      existing_refs: {
        ...record.existing_refs,
        tests: uniqueSortedStrings([...record.existing_refs.tests, ...attribution.tests])
      }
    }
  })
}
