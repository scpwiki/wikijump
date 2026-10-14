#!/usr/bin/env node

import fs from "node:fs/promises"
import path from "node:path"
import process from "node:process"

const HEX40 = /^[0-9a-f]{40}$/u
const HEX64 = /^[0-9a-f]{64}$/u
const ID_PATTERNS = {
  surface_id: /^surface:[0-9]{8}$/u,
  raw_record_id: /^raw:[0-9]{8}$/u,
  relationship_id: /^relationship:[0-9]{8}$/u,
  assignment_id: /^assignment:[0-9]{8}$/u,
  source_manifest_id: /^manifest:[0-9]{8}$/u
}
const SCHEMAS = {
  source_manifest: ["source_manifest_id", "source_class", "schema_id", "repository", "commit", "tree", "path", "sha256"],
  raw_source_record: ["source_manifest_id", "raw_record_id", "record_sha256"],
  surface_assignment: ["assignment_id", "surface_id", "source_manifest_id", "raw_record_id"],
  relationship: ["relationship_id", "relationship_type", "source_manifest_id", "raw_record_id", "target_surface_id", "evidence"],
  compatibility_ledger_row: ["surface_id", "actor", "input", "observable_interval", "result", "source", "evidence", "tests", "owners", "issues", "blockers", "candidate", "standing", "closure"]
}
const FINAL_ZERO_CLASSES = [
  "complete_product_rows_open_or_unreconciled",
  "duplicate_or_ambiguous_canonical_identities",
  "missing_or_failing_candidate_proofs",
  "missing_or_failing_final_validation",
  "missing_or_failing_standing_proofs",
  "missing_or_stale_source_provenance",
  "missing_public_surfaces",
  "unimplemented_source_required_rows",
  "unknown_owners_or_untyped_edges",
  "unrepresented_charter_requirements",
  "unresolved_wikidot_evidence_requirements"
]

function fail(message) {
  throw new Error(message)
}

function exactKeys(value, keys, context) {
  if (!value || typeof value !== "object" || Array.isArray(value)) fail(`${context} must be an object`)
  const actual = Object.keys(value).sort()
  const expected = [...keys].sort()
  if (JSON.stringify(actual) !== JSON.stringify(expected)) fail(`${context} has missing or unknown fields`)
}

function exactStrings(actual, expected, context) {
  if (!Array.isArray(actual) || JSON.stringify(actual) !== JSON.stringify(expected)) {
    fail(`${context} does not match the closed vocabulary`)
  }
}

function sortedUniqueStrings(values, context) {
  if (
    !Array.isArray(values) ||
    values.some((value) => typeof value !== "string" || value === "") ||
    JSON.stringify(values) !== JSON.stringify([...new Set(values)].sort())
  ) fail(`${context} must be sorted unique strings`)
}

function noNull(value, context = "contract") {
  if (value === null) fail(`${context} contains ambiguous null state`)
  if (Array.isArray(value)) value.forEach((item, index) => noNull(item, `${context}[${index}]`))
  else if (value && typeof value === "object") {
    for (const [key, item] of Object.entries(value)) noNull(item, `${context}.${key}`)
  }
}

function opaqueId(value, kind, context) {
  if (!ID_PATTERNS[kind].test(value ?? "")) fail(`${context} is not a stable opaque ${kind}`)
}

function uniqueBy(records, key, context) {
  const values = records.map((record) => record[key])
  if (new Set(values).size !== values.length) fail(`duplicate ${context}`)
}

function artifact(reference, context, absolutePath = false) {
  exactKeys(reference, ["path", "sha256"], context)
  if (
    typeof reference.path !== "string" ||
    reference.path === "" ||
    (absolutePath && !path.isAbsolute(reference.path)) ||
    !HEX64.test(reference.sha256 ?? "")
  ) {
    fail(`${context} is not immutable path+digest evidence`)
  }
}

function valueState(value, context, contract) {
  if (value?.state === "known") {
    exactKeys(value, ["state", "value"], context)
    if (typeof value.value !== "string" || value.value === "") fail(`${context} has no known value`)
  } else if (value?.state === contract.missing_state_model.missing_tag) {
    exactKeys(value, ["state", "reason"], context)
    if (!contract.vocabularies.missing_reasons.includes(value.reason)) fail(`${context} has unknown missing reason`)
  } else fail(`${context} has unknown value state`)
}

function presentOrMissing(value, presentField, states, context, contract, itemCheck) {
  if (!states.includes(value?.state)) fail(`${context} has unknown state`)
  if (value.state === "missing") {
    exactKeys(value, ["state", "reason"], context)
    if (!contract.vocabularies.missing_reasons.includes(value.reason)) fail(`${context} has unknown missing reason`)
    return
  }
  exactKeys(value, ["state", presentField], context)
  if (!Array.isArray(value[presentField]) || value[presentField].length === 0) fail(`${context} has no ${presentField}`)
  value[presentField].forEach((item, index) => itemCheck(item, `${context}.${presentField}[${index}]`))
}

function validate(contract) {
  noNull(contract)
  exactKeys(contract, ["schema", "phase", "identity_policy", "missing_state_model", "record_schemas", "vocabularies", "owner_keys", "structural_examples"], "contract")
  if (contract.schema !== "wikijump.compatibility_denominator_contract.v1" || contract.phase !== "phase_1a_contract_only") fail("unknown contract schema or phase")

  exactKeys(contract.identity_policy, ["canonical_surface_id_pattern", "source_manifest_id_pattern", "raw_record_id_pattern", "relationship_id_pattern", "assignment_id_pattern", "canonical_assignment", "identity_from_mutable_coordinates", "reassignment"], "identity policy")
  if (
    contract.identity_policy.canonical_surface_id_pattern !== "^surface:[0-9]{8}$" ||
    contract.identity_policy.source_manifest_id_pattern !== "^manifest:[0-9]{8}$" ||
    contract.identity_policy.raw_record_id_pattern !== "^raw:[0-9]{8}$" ||
    contract.identity_policy.relationship_id_pattern !== "^relationship:[0-9]{8}$" ||
    contract.identity_policy.assignment_id_pattern !== "^assignment:[0-9]{8}$" ||
    contract.identity_policy.canonical_assignment !== "maintained_opaque" ||
    contract.identity_policy.identity_from_mutable_coordinates !== "forbidden" ||
    contract.identity_policy.reassignment !== "forbidden"
  ) fail("canonical identity policy permits mutable-coordinate identity")

  exactKeys(contract.missing_state_model, ["missing_tag", "null_is_valid_state", "required_missing_reason", "value_states"], "missing state model")
  if (
    contract.missing_state_model.missing_tag !== "missing" ||
    contract.missing_state_model.null_is_valid_state !== false ||
    contract.missing_state_model.required_missing_reason !== true
  ) fail("missing state model is ambiguous")
  exactStrings(contract.missing_state_model.value_states, ["known", "missing"], "value states")

  exactKeys(contract.record_schemas, Object.keys(SCHEMAS), "record schemas")
  for (const [name, fields] of Object.entries(SCHEMAS)) exactStrings(contract.record_schemas[name], fields, `${name} fields`)
  exactKeys(contract.vocabularies, ["relationship_types", "source_states", "evidence_states", "test_states", "owner_states", "issue_states", "blocker_states", "proof_states", "closure_states", "missing_reasons", "final_zero_nonzero_classes"], "vocabularies")
  exactStrings(contract.vocabularies.relationship_types, ["alias", "equivalence"], "relationship types")
  exactStrings(contract.vocabularies.source_states, ["missing", "present"], "source states")
  exactStrings(contract.vocabularies.evidence_states, ["missing", "present"], "evidence states")
  exactStrings(contract.vocabularies.test_states, ["missing", "present"], "test states")
  exactStrings(contract.vocabularies.owner_states, ["missing", "present"], "owner states")
  exactStrings(contract.vocabularies.issue_states, ["missing", "present"], "issue states")
  exactStrings(contract.vocabularies.blocker_states, ["none", "present"], "blocker states")
  exactStrings(contract.vocabularies.proof_states, ["blocked", "fail", "pass", "pending"], "proof states")
  exactStrings(contract.vocabularies.closure_states, ["closed", "open"], "closure states")
  exactStrings(contract.vocabularies.missing_reasons, ["blocked", "not_observed", "not_recorded", "not_written"], "missing reasons")
  exactStrings(contract.vocabularies.final_zero_nonzero_classes, FINAL_ZERO_CLASSES, "final-zero classes")

  exactKeys(contract.owner_keys, ["specification", "implementation"], "owner keys")
  sortedUniqueStrings(contract.owner_keys.specification, "specification owner keys")
  sortedUniqueStrings(contract.owner_keys.implementation, "implementation owner keys")
  const owners = new Set([...contract.owner_keys.specification, ...contract.owner_keys.implementation])

  const examples = contract.structural_examples
  exactKeys(examples, ["source_manifests", "raw_source_records", "surface_assignments", "relationships", "rows"], "structural examples")
  for (const value of Object.values(examples)) if (!Array.isArray(value) || value.length === 0) fail("missing structural example records")

  for (const manifest of examples.source_manifests) {
    exactKeys(manifest, SCHEMAS.source_manifest, "source manifest")
    opaqueId(manifest.source_manifest_id, "source_manifest_id", "source manifest id")
    if (![manifest.source_class, manifest.schema_id, manifest.repository, manifest.path].every((value) => typeof value === "string" && value !== "")) fail("source manifest has missing identity field")
    if (!HEX40.test(manifest.commit ?? "") || !HEX40.test(manifest.tree ?? "") || !HEX64.test(manifest.sha256 ?? "")) fail("source manifest has mutable or invalid revision binding")
  }
  uniqueBy(examples.source_manifests, "source_manifest_id", "source manifest id")
  const manifests = new Set(examples.source_manifests.map(({ source_manifest_id: id }) => id))

  for (const record of examples.raw_source_records) {
    exactKeys(record, SCHEMAS.raw_source_record, "raw source record")
    opaqueId(record.raw_record_id, "raw_record_id", "raw record id")
    if (!manifests.has(record.source_manifest_id) || !HEX64.test(record.record_sha256 ?? "")) fail("raw source record has unknown or mutable identity")
  }
  const rawIdentityKeys = examples.raw_source_records.map(
    ({ source_manifest_id: manifest, raw_record_id: raw }) => `${manifest}\0${raw}`
  )
  if (new Set(rawIdentityKeys).size !== rawIdentityKeys.length) fail("duplicate raw composite identity")
  const rawIdentities = new Set(examples.raw_source_records.map(({ source_manifest_id: manifest, raw_record_id: raw }) => `${manifest}\0${raw}`))
  const checkRawBinding = (value, context) => {
    if (!rawIdentities.has(`${value.source_manifest_id}\0${value.raw_record_id}`)) fail(`${context} references unknown raw identity`)
  }

  for (const assignment of examples.surface_assignments) {
    exactKeys(assignment, SCHEMAS.surface_assignment, "surface assignment")
    opaqueId(assignment.assignment_id, "assignment_id", "assignment id")
    opaqueId(assignment.surface_id, "surface_id", "canonical surface id")
    checkRawBinding(assignment, "surface assignment")
  }
  uniqueBy(examples.surface_assignments, "assignment_id", "assignment id")
  uniqueBy(examples.surface_assignments, "surface_id", "canonical surface id")
  const surfaces = new Set(examples.surface_assignments.map(({ surface_id: id }) => id))

  for (const relationship of examples.relationships) {
    exactKeys(relationship, SCHEMAS.relationship, "relationship")
    opaqueId(relationship.relationship_id, "relationship_id", "relationship id")
    if (!contract.vocabularies.relationship_types.includes(relationship.relationship_type)) fail("untyped relationship")
    checkRawBinding(relationship, "relationship")
    if (!surfaces.has(relationship.target_surface_id)) fail("relationship has unknown target surface")
    if (!Array.isArray(relationship.evidence) || relationship.evidence.length === 0) fail("relationship has no evidence")
    relationship.evidence.forEach((reference, index) => artifact(reference, `relationship evidence[${index}]`))
  }
  uniqueBy(examples.relationships, "relationship_id", "relationship id")
  if (!contract.vocabularies.relationship_types.every((type) => examples.relationships.some((row) => row.relationship_type === type))) fail("missing typed relationship example")

  for (const row of examples.rows) {
    exactKeys(row, SCHEMAS.compatibility_ledger_row, "compatibility ledger row")
    if (!surfaces.has(row.surface_id)) fail("row has unknown canonical surface")
    for (const field of ["actor", "input", "observable_interval", "result"]) valueState(row[field], field, contract)
    presentOrMissing(row.source, "bindings", contract.vocabularies.source_states, "source", contract, (binding, context) => {
      exactKeys(binding, ["source_manifest_id", "raw_record_id"], context)
      checkRawBinding(binding, context)
    })
    presentOrMissing(row.evidence, "references", contract.vocabularies.evidence_states, "evidence", contract, artifact)
    presentOrMissing(row.tests, "references", contract.vocabularies.test_states, "tests", contract, (reference, context) => {
      if (typeof reference !== "string" || !reference.startsWith("test:") || !reference.includes("#")) fail(`${context} is not a public test reference`)
    })
    if (!contract.vocabularies.owner_states.includes(row.owners?.state)) fail("owners has unknown state")
    if (row.owners.state === "missing") {
      exactKeys(row.owners, ["state", "reason"], "owners")
      if (!contract.vocabularies.missing_reasons.includes(row.owners.reason)) fail("owners has unknown missing reason")
    } else {
      exactKeys(row.owners, ["state", "specification", "implementation"], "owners")
      for (const owner of [...row.owners.specification, ...row.owners.implementation]) if (!owners.has(owner)) fail(`unknown owner: ${owner}`)
      if (row.owners.specification.length !== 1 || row.owners.implementation.length === 0) fail("row has missing owner")
    }
    if (!contract.vocabularies.issue_states.includes(row.issues?.state)) fail("issues has unknown state")
    if (row.issues.state === "missing") {
      exactKeys(row.issues, ["state", "reason"], "issues")
      if (!contract.vocabularies.missing_reasons.includes(row.issues.reason)) fail("issues has unknown missing reason")
    } else {
      exactKeys(row.issues, ["state", "numbers"], "issues")
      if (!Array.isArray(row.issues.numbers) || row.issues.numbers.some((number) => !Number.isSafeInteger(number) || number <= 0)) fail("issues has invalid issue numbers")
      if (row.issues.numbers.length === 0) fail("row has no owning issue")
    }
    if (!contract.vocabularies.blocker_states.includes(row.blockers?.state)) fail("blockers has unknown state")
    exactKeys(row.blockers, ["state", "numbers"], "blockers")
    if (!Array.isArray(row.blockers.numbers) || row.blockers.numbers.some((number) => !Number.isSafeInteger(number) || number <= 0)) fail("blockers has invalid issue numbers")
    if (row.blockers.state === "none" && row.blockers.numbers.length !== 0) fail("none blocker state has blockers")
    if (row.blockers.state === "present" && row.blockers.numbers.length === 0) fail("present blocker state has no blockers")
    for (const field of ["candidate", "standing"]) {
      if (!contract.vocabularies.proof_states.includes(row[field]?.state)) fail(`${field} has unknown state`)
      exactKeys(row[field], ["state", "artifacts"], field)
      if (!Array.isArray(row[field].artifacts)) fail(`${field} artifacts must be an array`)
      row[field].artifacts.forEach((reference, index) => artifact(reference, `${field}.artifacts[${index}]`, true))
      if (["pass", "fail"].includes(row[field].state) && row[field].artifacts.length === 0) fail(`${field} proof has no immutable artifact`)
    }
    if (!contract.vocabularies.closure_states.includes(row.closure?.state)) fail("closure has unknown state")
    exactKeys(row.closure, ["state", "references"], "closure")
    if (!Array.isArray(row.closure.references) || row.closure.references.some((reference) => typeof reference !== "string" || reference === "")) fail("closure references must be an array")
    if (row.closure.state === "closed" && row.closure.references.length === 0) fail("closed row has no closure reference")
  }
  uniqueBy(examples.rows, "surface_id", "compatibility ledger surface id")
}

async function main() {
  if (process.argv.length !== 4 || process.argv[2] !== "--contract") fail("usage: verify-compatibility-denominator-contract.mjs --contract PATH")
  const contract = JSON.parse(await fs.readFile(process.argv[3], "utf8"))
  validate(contract)
  process.stdout.write("compatibility denominator contract is structurally closed\n")
}

main().catch((error) => {
  process.stderr.write(`compatibility denominator contract failed: ${error.message}\n`)
  process.exitCode = 1
})
