import assert from "node:assert/strict"
import { existsSync, readFileSync } from "node:fs"
import path from "node:path"
import test from "node:test"
import { fileURLToPath } from "node:url"

import { classifyChanges, GROUPS } from "../scripts/classify-changes.mjs"

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..")
const read = (file) => readFileSync(path.join(root, file), "utf8")
const workflow = (name) => read(`.github/workflows/${name}`)
const hasYamlLine = (source, expected) => source.split("\n").some((line) => line.trim() === expected)

test("one central workflow owns required checks without reacting to labels", () => {
  const source = workflow("ci-gate.yaml")
  const trigger = source.slice(source.indexOf("on:\n"), source.indexOf("\npermissions:\n"))

  assert.match(trigger, /^\s*pull_request:$/m)
  assert.doesNotMatch(trigger, /^\s*paths(?:-ignore)?:$/m)
  for (const action of ["opened", "synchronize", "reopened", "edited", "ready_for_review", "converted_to_draft"]) {
    assert.ok(hasYamlLine(trigger, `- ${action}`), action)
  }
  assert.doesNotMatch(trigger, /^      - (?:labeled|unlabeled)$/m)
  assert.doesNotMatch(source, /landing|full-ci/)
  assert.match(source, /^permissions:\n  contents: read$/m)
  assert.doesNotMatch(source, /id-token:/)
})

test("base edits rerun central CI while metadata edits stay isolated", () => {
  const source = workflow("ci-gate.yaml")
  const concurrency = source.slice(source.indexOf("concurrency:\n"), source.indexOf("\njobs:\n"))
  const classify = source.slice(source.indexOf("  classify:\n"), source.indexOf("  workflow_policy:\n"))
  const gate = source.slice(source.indexOf("  gate:\n"))

  for (const section of [concurrency, classify, gate]) {
    assert.match(section, /github\.event\.action != 'edited' \|\| github\.event\.changes\.base != null/)
  }
  assert.match(concurrency, /format\('ci-pr-\{0\}', github\.event\.pull_request\.number\)/)
  assert.match(concurrency, /format\('ci-run-\{0\}', github\.run_id\)/)
  assert.match(concurrency, /cancel-in-progress:/)
  assert.match(gate, /format\('CI \/ metadata no-op \(\{0\}\)', github\.run_id\)/)
})

test("PR classification uses three-dot history while push classification uses two endpoints", () => {
  const source = workflow("ci-gate.yaml")
  const classify = source.slice(source.indexOf("      - name: Classify every changed path"), source.indexOf("\n  workflow_policy:"))

  assert.match(classify, /elif \[\[ "\$\{GITHUB_EVENT_NAME\}" == pull_request \]\]; then\n\s+git diff --no-renames --name-only -z "\$\{BASE_SHA\}\.\.\.\$\{HEAD_SHA\}"/)
  assert.match(classify, /else\n\s+git diff --no-renames --name-only -z "\$\{BASE_SHA\}" "\$\{HEAD_SHA\}"/)
  assert.match(source, /fetch-depth: 0/)
  assert.doesNotMatch(classify, /pulls\/.*files|github\.event\.pull_request\.changed_files/)
})

test("component and lockfile changes select required validation", () => {
  assert.equal(classifyChanges(["deepwell/Cargo.lock"]).deepwell, true)
  assert.equal(classifyChanges(["wws/Cargo.lock"]).wws, true)
  assert.equal(classifyChanges(["locales/validator/Cargo.lock"]).locales, true)
  assert.equal(classifyChanges(["framerail/pnpm-lock.yaml"]).framerail, true)
  assert.equal(classifyChanges(["install/prod/deepwell/config.toml"]).deepwell, true)

  const toolchain = classifyChanges(["rust-toolchain.toml"])
  assert.equal(toolchain.deepwell, true)
  assert.equal(toolchain.wws, true)
  assert.equal(toolchain.locales, true)
})

test("classifier and gate changes fail closed", () => {
  for (const file of [
    ".github/workflows/ci-gate.yaml",
    ".github/scripts/classify-changes.mjs",
    ".github/tests/ci-gate-workflow.test.mjs"
  ]) {
    const selected = classifyChanges([file])
    for (const group of GROUPS) assert.equal(selected[group], true, `${file}: ${group}`)
  }

  const manual = classifyChanges([], true)
  for (const group of GROUPS) assert.equal(manual[group], true, group)
})

test("Full CI changes select each optional component and workflow policy", () => {
  const selected = classifyChanges([".github/workflows/full-ci.yaml"])
  for (const group of ["deepwell", "wws", "framerail", "workflow"]) assert.equal(selected[group], true, group)
  assert.equal(selected.locales, false)
})

test("documentation is cheap and unknown paths fail closed", () => {
  const docs = classifyChanges(["README.md", "AGENTS.md", "docs/development.md"])
  for (const group of GROUPS) assert.equal(docs[group], false, group)

  for (const file of ["new-service/config.toml", "unexpected-root.json", "install/new-tier/config.toml"]) {
    const selected = classifyChanges([file])
    for (const group of GROUPS) assert.equal(selected[group], true, `${file}: ${group}`)
  }
})

test("Deepwell draft and candidate paths are exclusive and parallel after classification", () => {
  const source = workflow("ci-gate.yaml")
  const draft = source.slice(source.indexOf("  deepwell_draft:\n"), source.indexOf("  deepwell_candidate:\n"))
  const candidate = source.slice(source.indexOf("  deepwell_candidate:\n"), source.indexOf("  wws:\n"))
  const gate = source.slice(source.indexOf("  gate:\n"))

  assert.match(draft, /needs\.classify\.outputs\.draft == 'true'/)
  assert.match(candidate, /needs\.classify\.outputs\.candidate == 'true'/)
  assert.doesNotMatch(candidate, /needs:\s*\n\s+- classify|needs: deepwell_draft/)
  assert.doesNotMatch(draft, /services:|DATABASE_URL|Start MinIO|sqlx/)
  assert.match(draft, /-deepwell-draft-/)
  assert.match(candidate, /-deepwell-candidate-/)

  for (const command of [
    "cargo machete deepwell",
    "cargo fmt --all -- --check",
    "cargo clippy --locked --tests --no-deps",
    "cargo test --locked --lib --no-default-features",
    "Start MinIO",
    "sqlx migrate run",
    "cargo test --locked --all-features"
  ]) assert.ok(candidate.includes(command), command)

  for (const job of ["deepwell_draft", "deepwell_candidate"]) assert.ok(hasYamlLine(gate, `- ${job}`), job)
  assert.match(gate, /needs\.classify\.outputs\.draft == 'true' && 'CI \/ draft gate' \|\| 'CI \/ gate'/)
  assert.doesNotMatch(source, /^  deepwell_(?:fast|integration):$/m)
  assert.doesNotMatch(source, /tarpaulin|coverage\/cobertura/)
})

test("one Full CI workflow owns coverage and browser validation", () => {
  for (const old of ["deepwell.yaml", "wws.yaml", "framerail.yaml"]) {
    assert.equal(existsSync(path.join(root, ".github/workflows", old)), false, old)
  }

  const source = workflow("full-ci.yaml")
  const trigger = source.slice(source.indexOf("on:\n"), source.indexOf("\npermissions:\n"))
  const concurrency = source.slice(source.indexOf("concurrency:\n"), source.indexOf("\njobs:\n"))
  for (const action of ["opened", "synchronize", "reopened", "edited", "ready_for_review", "converted_to_draft", "labeled", "unlabeled", "closed"]) {
    assert.ok(hasYamlLine(trigger, `- ${action}`), action)
  }
  for (const job of ["deepwell_coverage", "export_deepwell_coverage", "wws_coverage", "export_wws_coverage", "framerail_browser"]) {
    assert.ok(hasYamlLine(source, `${job}:`), job)
  }
  // Only the browser job is gated behind the full-ci label on a pull request.
  // Both coverage jobs feed Codecov, whose export already refuses to run on a
  // pull request, so they run on push events instead of duplicating the
  // candidate suite for an artifact nothing reads.
  assert.equal((source.match(/contains\(github\.event\.pull_request\.labels\.\*\.name, 'full-ci'\)/g) ?? []).length, 1)
  for (const job of ["deepwell_coverage", "wws_coverage", "export_deepwell_coverage", "export_wws_coverage"]) {
    const start = source.indexOf(`  ${job}:\n`)
    assert.ok(start >= 0, job)
    const condition = source.slice(start, source.indexOf("\n    runs-on:", start))
    assert.match(condition, /if: \$\{\{ github\.event_name != 'pull_request' \}\}/, job)
  }
  assert.match(concurrency, /format\('full-ci-pr-\{0\}', github\.event\.pull_request\.number\)/)
  assert.match(concurrency, /format\('full-ci-run-\{0\}', github\.run_id\)/)
  assert.match(concurrency, /github\.event\.action == 'unlabeled'\) && github\.event\.label\.name == 'full-ci'/)
  assert.match(concurrency, /github\.event\.action == 'edited' && github\.event\.changes\.base != null/)
  assert.match(concurrency, /cancel-in-progress:/)
  for (const condition of [
    "github.event.pull_request.draft == false",
    "github.event.action != 'closed'",
    "github.event.action != 'converted_to_draft'",
    "github.event.action == 'labeled' && github.event.label.name == 'full-ci'"
  ]) assert.equal(source.split(condition).length - 1, 1, condition)
  assert.ok(source.split("github.event.action == 'edited' && github.event.changes.base != null").length - 1 >= 1)
  const deepwellCoverage = source.slice(source.indexOf("  deepwell_coverage:\n"), source.indexOf("  export_deepwell_coverage:\n"))
  assert.match(deepwellCoverage, /cargo \+nightly tarpaulin.*-- --test-threads 1/)
  assert.match(source, /pnpm --dir framerail test/)
  assert.match(source, /!startsWith\(github\.ref, 'refs\/tags\/'\)/)
})

test("Full CI cancellation and execution policy handles label lifecycle cheaply", () => {
  const active = ({ action, label = null, baseChanged = false }) =>
    !["labeled", "unlabeled", "edited"].includes(action) ||
    (["labeled", "unlabeled"].includes(action) && label === "full-ci") ||
    (action === "edited" && baseChanged)
  const run = ({ action, label = null, baseChanged = false, draft = false, hasFullCi = false }) =>
    !draft && hasFullCi && !["closed", "converted_to_draft"].includes(action) && (
      !["labeled", "unlabeled", "edited"].includes(action) ||
      (action === "labeled" && label === "full-ci") ||
      (action === "edited" && baseChanged)
    )

  for (const action of ["opened", "synchronize", "reopened", "ready_for_review"]) {
    assert.equal(active({ action }), true, `${action}: active`)
    assert.equal(run({ action, hasFullCi: true }), true, `${action}: run`)
    assert.equal(run({ action }), false, `${action}: no label`)
  }
  assert.equal(active({ action: "labeled", label: "full-ci" }), true)
  assert.equal(run({ action: "labeled", label: "full-ci", hasFullCi: true }), true)
  assert.equal(active({ action: "unlabeled", label: "full-ci" }), true)
  assert.equal(run({ action: "unlabeled", label: "full-ci", hasFullCi: true }), false)
  assert.equal(active({ action: "labeled", label: "docs" }), false)
  assert.equal(run({ action: "labeled", label: "docs", hasFullCi: true }), false)
  assert.equal(active({ action: "edited", baseChanged: true }), true)
  assert.equal(run({ action: "edited", baseChanged: true, hasFullCi: true }), true)
  assert.equal(active({ action: "edited" }), false)
  assert.equal(active({ action: "converted_to_draft" }), true)
  assert.equal(run({ action: "converted_to_draft", hasFullCi: true }), false)
  assert.equal(active({ action: "closed" }), true)
  assert.equal(run({ action: "closed", hasFullCi: true }), false)
})

test("OIDC is isolated from jobs that execute pull request code", () => {
  const source = workflow("full-ci.yaml")
  for (const [coverage, exporter, next] of [
    ["deepwell_coverage", "export_deepwell_coverage", "wws_coverage"],
    ["wws_coverage", "export_wws_coverage", "framerail_browser"]
  ]) {
    const coverageSource = source.slice(source.indexOf(`  ${coverage}:\n`), source.indexOf(`  ${exporter}:\n`))
    const exporterStart = source.indexOf(`  ${exporter}:\n`)
    const exporterSource = source.slice(exporterStart, source.indexOf(`  ${next}:\n`, exporterStart))
    assert.doesNotMatch(coverageSource, /id-token:/)
    assert.match(exporterSource, /github\.event_name != 'pull_request'/)
    assert.match(exporterSource, /^\s*id-token: write$/m)
    assert.doesNotMatch(exporterSource, /actions\/checkout|\brun:/)
  }
})

test("Framerail unit and browser suites remain separate", () => {
  const pkg = JSON.parse(read("framerail/package.json"))
  const gate = workflow("ci-gate.yaml")
  const full = workflow("full-ci.yaml")
  const playwright = read("framerail/playwright.config.ts")

  assert.match(pkg.scripts["test:unit"], /^node --test(?: tests\/[\w-]+\.test\.(?:js|ts))+$/)
  assert.doesNotMatch(pkg.scripts["test:unit"], /\.spec\.(?:js|ts)/)
  assert.equal(pkg.scripts.test, "playwright test")
  for (const command of ["build", "test:unit", "lint"]) assert.ok(gate.includes(`pnpm --dir framerail ${command}`), command)
  assert.match(full, /pnpm --dir framerail test/)
  assert.doesNotMatch(playwright, /\.test\.(?:js|ts)/)
})

test("central gate owns workflow policy and locales validation", () => {
  for (const old of ["workflow-lint.yaml", "locales.yaml"]) {
    assert.equal(existsSync(path.join(root, ".github/workflows", old)), false, old)
  }
  const source = workflow("ci-gate.yaml")
  assert.match(source, /node --test \.github\/tests\/\*\.test\.mjs/)
  assert.match(source, /cargo run --locked/)
})

test("actions in touched workflows are immutable pins with version comments", () => {
  for (const name of ["ci-gate.yaml", "full-ci.yaml"]) {
    const source = workflow(name)
    const uses = [...source.matchAll(/^\s*uses:\s*([^\s#]+)\s+#\s+(\S+)$/gm)]
    assert.ok(uses.length > 0, name)
    for (const [, action, version] of uses) {
      assert.match(action, /^[^@]+@[0-9a-f]{40}$/, `${name}: ${action}`)
      assert.match(version, /^v\d+(?:\.\d+)*$/, `${name}: ${version}`)
    }
    assert.equal(uses.length, (source.match(/^\s*uses:/gm) ?? []).length, name)
  }
})
