import { readFileSync } from "node:fs"
import { pathToFileURL } from "node:url"

export const GROUPS = ["deepwell", "wws", "framerail", "locales", "workflow"]

/// Files outside `.github/` that `.github/tests/ci-gate-workflow.test.mjs`
/// makes assertions about. Keep in step with the `read(...)` calls there.
const WORKFLOW_POLICY_SUBJECTS = new Set([
  "framerail/package.json",
  "framerail/playwright.config.ts",
])

const selectAll = (selected) => {
  for (const group of GROUPS) selected[group] = true
}

const metadataOnly = (file) =>
  file.startsWith("docs/") ||
  ["AGENTS.md", "CLAUDE.md", "CODEOWNERS", "LICENSE.md", "README.md", "SECURITY.md"].includes(file)

export function classifyChanges(paths, all = false) {
  const selected = Object.fromEntries(GROUPS.map((group) => [group, false]))

  if (all) {
    selectAll(selected)
    return selected
  }

  for (const file of paths) {
    if (!file) continue

    if (
      file === ".github/workflows/ci-gate.yaml" ||
      file === ".github/scripts/classify-changes.mjs" ||
      file === ".github/tests/ci-gate-workflow.test.mjs"
    ) {
      selectAll(selected)
      continue
    }

    if (metadataOnly(file)) continue

    let matched = false

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
    if (file === ".github/codecov.yml") {
      selected.deepwell = true
      selected.wws = true
    }
    if (file === ".github/workflows/full-ci.yaml") {
      selected.deepwell = true
      selected.wws = true
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
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const all = process.argv.includes("--all")
  const paths = all ? [] : readFileSync(0, "utf8").split("\0")
  emitOutputs(classifyChanges(paths, all))
}
