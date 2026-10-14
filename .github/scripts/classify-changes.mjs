import { readFileSync } from "node:fs"
import { pathToFileURL } from "node:url"

export const GROUPS = ["deepwell", "wws", "framerail", "locales", "workflow", "theme_lab"]

/// Files outside `.github/` that `.github/tests/ci-gate-workflow.test.mjs`
/// makes assertions about. Keep in step with the `read(...)` calls there.
const WORKFLOW_POLICY_SUBJECTS = new Set([
  "framerail/package.json",
  "framerail/playwright.config.ts",
])

const selectAll = (selected) => {
  for (const group of GROUPS) selected[group] = true
  selected.verification = true
}

const selectWorkflow = (selected) => {
  selected.workflow = true
}

const metadataOnly = (file) =>
  file.startsWith("docs/") ||
  ["AGENTS.md", "CLAUDE.md", "CODEOWNERS", "LICENSE.md", "README.md", "SECURITY.md"].includes(file)

const machineAuthorityDocument = (file) =>
  file.startsWith("docs/development/") && file.endsWith(".json")

const verificationInput = (file) =>
  file.startsWith("install/local/wikidot-verification/") ||
  file.startsWith("install/standing/") ||
  machineAuthorityDocument(file) ||
  file.startsWith("docs/wikidot-specifications/") ||
  [
    ".github/workflows/wikidot-verification.yaml",
    "scripts/data/wikidot-implementation-ledger.json",
    "scripts/data/wikidot-live-observations.json",
    "scripts/data/wikidot-spec-supplements.json",
    "scripts/generate-wikidot-specifications.mjs",
    "scripts/refresh-test-quality-snapshot.mjs",
    "scripts/run-test-quality-audit.mjs",
    "scripts/initialize-wikidot-implementation-ledger.mjs",
    "scripts/lib/wikidot-implementation-ledger.mjs"
  ].includes(file) ||
  // Dated detailed-spec evidence aliases are consumed only by the
  // specification generator and implementation-ledger checks.
  file.startsWith("scripts/data/wikidot-detailed-") ||
  // The test-quality audit's library is exercised only by the verification
  // package's test-quality tests.
  file.startsWith("scripts/lib/test-quality-")

const verificationOnly = (file) =>
  verificationInput(file) && file !== ".github/workflows/wikidot-verification.yaml"

export function classifyChanges(paths, all = false) {
  const selected = {
    ...Object.fromEntries(GROUPS.map((group) => [group, false])),
    verification: false
  }

  if (all) {
    selectAll(selected)
    return selected
  }

  for (const file of paths) {
    if (!file) continue

    if (verificationInput(file)) selected.verification = true

    if (
      file === ".github/workflows/ci-gate.yaml" ||
      file === ".github/scripts/classify-changes.mjs" ||
      file === ".github/tests/ci-gate-workflow.test.mjs" ||
      file === "scripts/preflight.sh"
    ) {
      selectWorkflow(selected)
      continue
    }

    if (metadataOnly(file) || verificationOnly(file)) continue

    let matched = false

    if (file.startsWith("install/local/theme-lab/")) {
      selected.theme_lab = true
      matched = true
    }

    if (file.startsWith(".github/")) {
      selected.workflow = true
      matched = true
    }
    // The workflow policy tests assert about these files, so a change to one
    // has to run them. Without this the guard reads files whose changes cannot
    // trigger it, and a violation lands on develop and surfaces on some later
    // unrelated PR instead.
    if (WORKFLOW_POLICY_SUBJECTS.has(file)) {
      selected.workflow = true
    }
    if (file === ".github/workflows/full-ci.yaml") {
      selected.framerail = true
    }

    if (file === "rust-toolchain.toml") {
      selected.deepwell = true
      selected.wws = true
      selected.locales = true
      matched = true
    }
    for (const group of ["deepwell", "wws", "framerail", "locales"]) {
      if (file.startsWith(`${group}/`)) {
        selected[group] = true
        matched = true
      }
    }
    for (const group of ["deepwell", "wws", "framerail"]) {
      if (new RegExp(`^install/(?:local|dev|prod)/${group}/`, "u").test(file)) {
        selected[group] = true
        matched = true
      }
    }

    if (!matched) selectAll(selected)
  }

  return selected
}

function emitOutputs(selected) {
  for (const group of GROUPS) console.log(`${group}=${selected[group]}`)
  console.log(`verification=${selected.verification}`)
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const all = process.argv.includes("--all")
  const paths = all ? [] : readFileSync(0, "utf8").split("\0")
  emitOutputs(classifyChanges(paths, all))
}
