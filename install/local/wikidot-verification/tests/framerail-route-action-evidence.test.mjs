import assert from "node:assert/strict"
import { execFile } from "node:child_process"
import { createHash } from "node:crypto"
import { lstat, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { fileURLToPath } from "node:url"
import { promisify } from "node:util"
import test from "node:test"

import {
  buildTemporalCapturePlan,
  captureUrlsSha256,
  assertLoadingBoundaryPresent,
  assertByteLimit,
  copyRunOwnedStorageStates,
  closeCaptureEgressProxies,
  DOM_MAX_BYTES,
  DIAGNOSTIC_MAX_BYTES,
  runOwnedStorageStatePaths,
  SCREENSHOT_MAX_BYTES,
  SHUTDOWN_TIMEOUT_MS,
  assertRunOwnedStorageStatesAbsent,
  removeRunOwnedStorageStates,
  validateCaptureInputBindings,
  validateSourceIdentity,
  validateTemporalRunContract,
  validateOutputPreflight,
  viewportCropGeometry,
  verifyHistoricalEvidence,
  requireNavigationResponse,
  urlSuffixMatches,
} from "../scripts/capture-framerail-route-action-temporal.mjs"

const root = new URL("../../../../", import.meta.url)
const registryPath = new URL("docs/development/framerail-route-action-evidence.json", root)
const browserContractPath = new URL(
  "install/local/wikidot-verification/fixtures/framerail-route-action-browser/run-contract.json",
  root
)
const execFileAsync = promisify(execFile)
const gitEnvironment = Object.freeze({
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_NO_REPLACE_OBJECTS: "1",
  GIT_OPTIONAL_LOCKS: "0",
  GIT_TERMINAL_PROMPT: "0",
  LANG: "C",
  LC_ALL: "C",
  PATH: "/usr/bin:/bin"
})
const temporalIds = [
  "control:create",
  "control:restore",
  "pane:append",
  "pane:backlinks",
  "pane:delete",
  "pane:edit-meta",
  "pane:layout",
  "pane:lock",
  "pane:move",
  "pane:parent",
  "pane:site-tools",
  "pane:tags",
  "pane:vote",
  "pane:watchers"
]

test("issue #1372 X11 viewport crop excludes headed browser chrome", () => {
  assert.deepEqual(
    viewportCropGeometry({
      innerWidth: 1280,
      innerHeight: 720,
      outerWidth: 1288,
      outerHeight: 805,
      screenX: 0,
      screenY: 0,
      devicePixelRatio: 1,
    }),
    {x: 4, y: 85, width: 1280, height: 720, crop: "1280x720+4+85"},
  )
  assert.throws(
    () => viewportCropGeometry({
      innerWidth: 1280,
      innerHeight: 720,
      outerWidth: 1288,
      outerHeight: 805,
      screenX: 200,
      screenY: 200,
      devicePixelRatio: 1,
    }),
    /outside the owned capture display/u,
  )
})
const missingIntervals = ["denial", "failure", "loading", "selection", "settled", "success"]

function validCaptureInputs(contract, urls) {
  const oracle = {type: "event", event: {kind: "response", method: "POST", status: 200, url_suffix: "/fixture-result"}}
  const bySubject = Object.fromEntries(contract.subjects.map(({id}) => [id, oracle]))
  const urlsSha256 = captureUrlsSha256(urls)
  return {
    fixture: {
      descriptor: {
        schema: "wikijump.framerail_route_action_fixture_identity.v1",
        evidence_registry: contract.evidence_registry,
        urls,
        urls_sha256: urlsSha256,
        run_owned_fixture: {
          restored: [{path: "/tmp/run-owned-fixture", sha256: "a".repeat(64)}],
          removed: []
        }
      }
    },
    failureControl: {
      descriptor: {
        schema: "wikijump.framerail_route_action_failure_control_identity.v1",
        evidence_registry: contract.evidence_registry,
        urls_sha256: urlsSha256,
        result_oracles: {denial: bySubject, failure: bySubject}
      }
    }
  }
}

const uniqueSorted = (values) =>
  values.length === new Set(values).size &&
  values.every((value, index) => !index || values[index - 1] < value)

async function git(args, encoding = "utf8") {
  const { stdout } = await execFileAsync("/usr/bin/git", args, {
    cwd: fileURLToPath(root),
    encoding,
    env: gitEnvironment,
    maxBuffer: 16 * 1024 * 1024
  })
  return stdout
}

const gitBlob = (revision, path) => git(["cat-file", "blob", `${revision}:${path}`], null)

async function verifyRegistry(registry) {
  assert.equal(registry.schema, "wikijump.framerail_route_action_evidence.v1")
  assert.deepEqual(Object.keys(registry), ["schema", "source_revision", "inventory", "current_inventory", "records"])
  assert.match(registry.source_revision, /^[0-9a-f]{40}$/u)
  assert.deepEqual(Object.keys(registry.inventory), ["path", "sha256"])
  assert.equal(registry.inventory.path, "docs/development/compatibility-surface-inventory.json")
  assert.match(registry.inventory.sha256, /^[0-9a-f]{64}$/u)
  assert.ok(Array.isArray(registry.records))

  const resolvedRevision = (await git([
    "rev-parse",
    "--verify",
    `${registry.source_revision}^{commit}`
  ])).trim()
  assert.equal(resolvedRevision, registry.source_revision)
  const inventoryBytes = await gitBlob(registry.source_revision, registry.inventory.path)
  assert.equal(
    createHash("sha256").update(inventoryBytes).digest("hex"),
    registry.inventory.sha256
  )
  const historicalInventory = JSON.parse(inventoryBytes.toString("utf8"))
  assert.deepEqual(Object.keys(registry.current_inventory),["source_revision","path","sha256"])
  assert.match(registry.current_inventory.source_revision,/^[0-9a-f]{40}$/u)
  assert.equal(registry.current_inventory.path,registry.inventory.path)
  assert.match(registry.current_inventory.sha256,/^[0-9a-f]{64}$/u)
  const currentBytes=await gitBlob(registry.current_inventory.source_revision,registry.current_inventory.path)
  assert.equal(createHash("sha256").update(currentBytes).digest("hex"),registry.current_inventory.sha256)
  const inventory=JSON.parse(currentBytes.toString("utf8"))
  const expected = inventory.surfaces
    .filter(({ kind }) => kind === "framerail_route" || kind === "framerail_server_action")
    .map(({ surface_id }) => surface_id)
  const actual = registry.records.map(({ surface_id }) => surface_id)
  assert.deepEqual(actual, expected)
  assert.equal(new Set(actual).size, actual.length)

  const cases = new Map(
    historicalInventory.surfaces
      .filter(({ kind }) => kind === "open43_audit_case")
      .map(({ existing_refs, surface_id }) => [
        surface_id.slice("open43-audit-case:".length),
        existing_refs.issues[0]
      ])
  )
  const temporal = []

  for (const record of registry.records) {
    assert.deepEqual(Object.keys(record), ["surface_id", "tests", "tracking", "temporal"])
    assert.ok(uniqueSorted(record.tests), `${record.surface_id} test links are not sorted and unique`)
    assert.ok(record.tracking.length > 0, `${record.surface_id} has no issue or case link`)
    assert.ok(
      uniqueSorted(
        record.tracking.map(
          (link) => `${String(link.issue).padStart(10, "0")}:${link.case ?? ""}`
        )
      ),
      `${record.surface_id} tracking links are not sorted and unique`
    )

    for (const reference of record.tests) {
      const match = /^(framerail\/tests\/[^:]+)::(.+)$/u.exec(reference)
      assert.ok(match, `${record.surface_id} has a noncanonical test link: ${reference}`)
      const source = await readFile(new URL(match[1], root), "utf8")
      const escaped = match[2].replace(/[.*+?^${}()|[\]\\]/gu, "\\$&")
      const anchors = source.match(new RegExp(`\\b(?:test|it)\\(\\s*["']${escaped}["']`, "gu")) ?? []
      assert.equal(anchors.length, 1, `${record.surface_id} has an unresolved test link: ${reference}`)
    }

    for (const link of record.tracking) {
      assert.ok(Number.isSafeInteger(link.issue) && link.issue > 0)
      assert.deepEqual(Object.keys(link), link.case ? ["issue", "case"] : ["issue"])
      if (link.case) {
        assert.equal(
          cases.get(link.case),
          link.issue,
          `${record.surface_id} has an unresolved issue/case link`
        )
      }
      else {
        const expectedIssues = new Map([
          ["framerail-route:/feed/forum/{feed}",[2091,2104]],
          ["framerail-route:/feed/site-changes.xml",[1372,2087]],
        ]).get(record.surface_id) ?? [1372]
        assert.ok(expectedIssues.includes(link.issue), `${record.surface_id} has an unresolved issue link`)
        if (expectedIssues.length>1 || expectedIssues[0]!==1372) assert.deepEqual(record.tracking.filter(item=>!item.case).map(item=>item.issue).sort((a,b)=>a-b),expectedIssues)
      }
    }

    for (const gap of record.temporal) {
      assert.deepEqual(Object.keys(gap), ["id", "status", "issue", "missing_intervals"])
      assert.equal(gap.status, "missing")
      assert.equal(gap.issue, 1372)
      assert.deepEqual(gap.missing_intervals, missingIntervals)
      temporal.push(gap.id)
    }
  }

  assert.deepEqual(temporal.sort(), temporalIds)
}

test("Framerail route and action evidence links resolve exactly", async () => {
  const registry = JSON.parse(await readFile(registryPath, "utf8"))
  await verifyRegistry(registry)
})

test("Framerail evidence verifier rejects stale and unresolved links", async () => {
  const registry = JSON.parse(await readFile(registryPath, "utf8"))
  const stale = structuredClone(registry)
  stale.records[0].tests = ["framerail/tests/not-present.test.js::not present"]
  await assert.rejects(verifyRegistry(stale))

  const unresolved = structuredClone(registry)
  unresolved.records[0].tracking = [{ issue: 754, case: "NOT_A_CASE" }]
  await assert.rejects(verifyRegistry(unresolved))

  const missing = structuredClone(registry)
  missing.records.pop()
  await assert.rejects(verifyRegistry(missing))

  const duplicate = structuredClone(registry)
  duplicate.records[1] = duplicate.records[0]
  await assert.rejects(verifyRegistry(duplicate))

  const duplicateLink = structuredClone(registry)
  duplicateLink.records[0].tests.push(duplicateLink.records[0].tests[0])
  await assert.rejects(verifyRegistry(duplicateLink))
})

test("Framerail evidence verifier rejects invented and poisoned source identities", async () => {
  const registry = JSON.parse(await readFile(registryPath, "utf8"))
  const invented = structuredClone(registry)
  invented.source_revision = "0000000000000000000000000000000000000000"
  await assert.rejects(verifyRegistry(invented))

  const poisoned = structuredClone(registry)
  poisoned.source_revision = "HEAD^{commit};touch /tmp/framerail-evidence-poisoned"
  await assert.rejects(verifyRegistry(poisoned))

  const staleInventory = structuredClone(registry)
  staleInventory.inventory.sha256 = "0".repeat(64)
  await assert.rejects(verifyRegistry(staleInventory))
  const staleCurrent=structuredClone(registry)
  staleCurrent.current_inventory.sha256="0".repeat(64)
  await assert.rejects(verifyRegistry(staleCurrent))
  const inventedCurrent=structuredClone(registry)
  inventedCurrent.current_inventory.source_revision="0".repeat(40)
  await assert.rejects(verifyRegistry(inventedCurrent))

})

test("issue #1372 browser run contract is complete, source-bound, and executable", async () => {
  const contract = JSON.parse(await readFile(browserContractPath, "utf8"))
  assert.equal(contract.schema, "wikijump.framerail_route_action_browser_run.v1")
  assert.equal(contract.issue, 1372)
  assert.equal(contract.status, "ready_for_capture")
  assert.equal(contract.repository, "Rokurolize/wikijump")
  assert.match(contract.source_revision, /^[0-9a-f]{40}$/u)
  const currentRegistry = JSON.parse(await readFile(registryPath, "utf8"))
  assert.equal(contract.source_revision, currentRegistry.source_revision)

  const registryBytes = await gitBlob(contract.source_revision, contract.evidence_registry.path)
  assert.equal(
    createHash("sha256").update(registryBytes).digest("hex"),
    contract.evidence_registry.sha256
  )
  const registry = JSON.parse(registryBytes.toString("utf8"))
  const expectedSubjectIds = registry.records.flatMap(({ temporal }) => temporal.map(({ id }) => id))
  assert.deepEqual(contract.subjects.map(({ id }) => id).sort(), expectedSubjectIds.sort())
  assert.equal(new Set(contract.subjects.map(({ id }) => id)).size, 14)
  assert.deepEqual(contract.required_intervals, missingIntervals)
  assert.equal(contract.subjects.length * contract.required_intervals.length, 84)
  for (const subject of contract.subjects) {
    assert.ok(["missing_page", "saved_page"].includes(subject.kind))
    assert.ok(subject.trigger_selectors.length > 0, `${subject.id} has no trigger`)
    assert.match(subject.settled_predicate.selector, /^[#.]/u)
    assert.equal(subject.settled_predicate.state, "visible")
  }

  assert.deepEqual(contract.actor_classes, ["denied", "permitted"])
  assert.deepEqual(contract.result_controls, ["denial", "failure", "success"])
  assert.equal(contract.layout, "wikidot")
  assert.deepEqual(contract.preflight_required, [
    "browser_identity",
    "capture_script_identity",
    "evidence_output_path",
    "failure_control_identity",
    "fixture_identity",
    "historical_evidence_identity",
    "run_contract_identity",
    "run_id",
    "repository_identity",
    "scenario_storage_state_identity",
    "runtime_identity"
  ])
  assert.deepEqual(contract.evidence_fields, [
    "actor_class", "browser_identity", "capture_errors", "console_errors", "dom", "dom_sha256",
    "failed_requests", "http_errors", "interval", "page_errors", "runtime_identity",
    "scenario", "screenshot", "screenshot_sha256", "status", "source_revision", "subject_id", "timestamp", "url"
  ])
  assert.equal(contract.authority, undefined)
  assert.equal(contract.capture.script, "install/local/wikidot-verification/scripts/capture-framerail-route-action-temporal.mjs")
  assert.match(contract.capture.script_sha256, /^[0-9a-f]{64}$/u)
  const captureScript = await readFile(new URL("../../scripts/capture-framerail-route-action-temporal.mjs", browserContractPath), "utf8")
  assert.ok(
    captureScript.indexOf('scenario, "selection", navigationStatus, outputDir') < captureScript.indexOf("const loadingSignal = subject.loading.kind"),
    "selection evidence must be sealed before activation wait timeouts are armed"
  )
  assert.ok(
    captureScript.indexOf("await waitForCandidateDocumentReady(page") < captureScript.indexOf("const loadingSignal = subject.loading.kind") &&
      captureScript.indexOf("await waitForCandidateDocumentReady(page") > captureScript.indexOf('scenario, "selection", navigationStatus, outputDir'),
    "success activation must wait for the browser document readiness boundary after selection evidence"
  )
  assert.ok(
    captureScript.indexOf("await waitForCandidateNetworkIdle(page") > captureScript.indexOf("await waitForCandidateDocumentReady(page") &&
      captureScript.indexOf("await waitForCandidateNetworkIdle(page") < captureScript.indexOf("const loadingSignal = subject.loading.kind"),
    "success activation must wait for the browser network-idle hydration boundary before arming signals"
  )
  assert.equal(
    (captureScript.match(/await waitForCandidateNetworkIdle\(page/gu) ?? []).length,
    2,
    "both success and result scenarios must await browser hydration before interaction"
  )
  assert.match(
    captureScript,
    /X11_IMPORT_EXECUTABLE[\s\S]*"-window",[\s\S]*"root",[\s\S]*"-crop",[\s\S]*geometry\.crop[\s\S]*"jpeg:-"[\s\S]*DISPLAY: captureDisplay\.display/u,
    "temporal screenshots must use the bounded run-owned X11 viewport evidence path"
  )
  assert.match(captureScript, /headless: false/u)
  assert.match(captureScript, /DISPLAY: captureDisplay\.display/u)
  assert.match(captureScript, /contract: contractIdentity,/u)
  assert.doesNotMatch(captureScript, /contract: contractIdentity\.identity/u)
  assert.doesNotMatch(captureScript, /requestAnimationFrame/u)
  assert.match(captureScript, /"-nolisten",\s*"tcp",\s*"-pn"/u)
  assert.match(captureScript, /await captureDisplay\.close\(\)/u)
  assert.doesNotMatch(captureScript, /Page\.captureScreenshot/u)
  assert.doesNotMatch(captureScript, /Page\.stopLoading/u)
  assert.match(
    captureScript,
    /page\.goto\(url, \{waitUntil: "commit", timeout: args\.timeoutMs\}\)/u,
    "temporal navigation must hand readiness to the exact trigger and predicate waits"
  )
  const failureControlSource = captureScript.slice(
    captureScript.indexOf("async function armFailureControl"),
    captureScript.indexOf("function armResultOracle"),
  )
  assert.match(
    failureControlSource,
    /await route\.fallback\(\)/u,
    "failure-control observation must fall through to the context-level request gate for non-target requests",
  )
  assert.doesNotMatch(
    failureControlSource,
    /route\.continue\(/u,
    "failure-control observation must not bypass the context-level request gate",
  )
  assert.equal(
    createHash("sha256").update(captureScript).digest("hex"),
    contract.capture.script_sha256
  )
  assert.deepEqual(contract.capture.scenarios, {
    denial: {result_oracle_source: "failure_control_identity", intervals: ["denial"]},
    failure: {result_oracle_source: "failure_control_identity", intervals: ["failure"]},
    success: {intervals: ["selection", "loading", "settled", "success"]}
  })
  assert.deepEqual(contract.subjects.find(({id}) => id === "control:create").loading, {
    kind: "navigation",
    url_suffix: "/edit/true",
    status: 200
  })
  assert.deepEqual(contract.historical_evidence, {
    path: "install/local/wikidot-verification/artifacts/page-pane-lazy-browser-20260713.json",
    sha256: "17b9b5215d40c32123ada66b43c5d5a37ea4a06a37bf2ebf99c0595e39c61ba9",
    schema: "wikijump.page_pane_lazy_browser.v1",
    classification: "historical_history_only"
  })
  const historicalIdentity = await verifyHistoricalEvidence(contract.historical_evidence)
  assert.equal(historicalIdentity.sha256, contract.historical_evidence.sha256)
  assert.equal(contract.cleanup.required, true)
  assert.deepEqual(contract.cleanup.capture_owned_fields, [
    "browser_sessions_closed",
    "egress_proxies_closed",
    "request_gate_flushed",
    "capture_lock_released",
    "storage_states_removed"
  ])

  const urls = {
    denial: {missing_page: "https://framerail.invalid/denial-missing", saved_page: "https://framerail.invalid/denial-saved"},
    failure: {missing_page: "https://framerail.invalid/failure-missing", saved_page: "https://framerail.invalid/failure-saved"},
    success: {missing_page: "https://framerail.invalid/success-missing", saved_page: "https://framerail.invalid/success-saved"}
  }
  assert.equal(validateTemporalRunContract(contract).subjects.length, 14)
  const plan = buildTemporalCapturePlan(contract, urls)
  assert.equal(plan.length, 84)
  assert.equal(new Set(plan.map(({subject, interval}) => `${subject.id}:${interval}`)).size, 84)
  assert.equal(plan.find(({subject}) => subject.id === "control:create").url, urls.denial.missing_page)
  assert.equal(plan.find(({subject}) => subject.id === "pane:append").url, urls.denial.saved_page)
})

test("issue #1372 capture seam requires observable settled boundaries", async () => {
  const contract = JSON.parse(await readFile(browserContractPath, "utf8"))
  const missingPredicate = structuredClone(contract)
  delete missingPredicate.subjects[0].settled_predicate
  assert.throws(() => validateTemporalRunContract(missingPredicate), /no exact settled predicate/u)

  const attachedPredicate = structuredClone(contract)
  attachedPredicate.subjects.find(({id}) => id === "pane:watchers").settled_predicate.state = "attached"
  assert.throws(() => validateTemporalRunContract(attachedPredicate), /no exact settled predicate/u)
})

test("issue #1372 capture seam rejects a stale historical artifact digest", async () => {
  const contract = JSON.parse(await readFile(browserContractPath, "utf8"))
  const stale = {...contract.historical_evidence, sha256: "0".repeat(64)}
  await assert.rejects(verifyHistoricalEvidence(stale), /SHA-256 does not match/u)
})

test("issue #1372 capture seam rejects authority state and narrowed-boundary drift", async () => {
  const contract = JSON.parse(await readFile(browserContractPath, "utf8"))
  const staleAuthority = structuredClone(contract)
  staleAuthority.authority = {source: "/home/roku/wjlab/state/current.json"}
  assert.throws(() => validateTemporalRunContract(staleAuthority), /authority state/u)

  const missingBoundary = structuredClone(contract)
  missingBoundary.capture.scenarios.success.intervals = ["selection", "loading", "success"]
  assert.throws(() => validateTemporalRunContract(missingBoundary), /wrong interval boundary/u)

  const duplicateBoundary = structuredClone(contract)
  duplicateBoundary.capture.scenarios.success.intervals = ["selection", "loading", "settled", "settled", "success"]
  assert.throws(() => validateTemporalRunContract(duplicateBoundary), /wrong interval boundary/u)
})

test("issue #1372 capture seam rejects all-success URLs and arbitrary identity files", async () => {
  const contract = JSON.parse(await readFile(browserContractPath, "utf8"))
  const urls = {
    denial: {missing_page: "https://framerail.invalid/denial-missing", saved_page: "https://framerail.invalid/denial-saved"},
    failure: {missing_page: "https://framerail.invalid/failure-missing", saved_page: "https://framerail.invalid/failure-saved"},
    success: {missing_page: "https://framerail.invalid/success-missing", saved_page: "https://framerail.invalid/success-saved"}
  }
  const identities = validCaptureInputs(contract, urls)
  const allSuccessUrls = {...urls, denial: urls.success, failure: urls.success}
  assert.throws(
    () => validateCaptureInputBindings(contract, allSuccessUrls, identities),
    /fixture identity is not bound to the supplied URLs/u
  )

  for (const key of ["fixture", "failureControl"]) {
    const arbitrary = structuredClone(identities)
    arbitrary[key] = {identity: {label: key, path: `/tmp/${key}`, sha256: "0".repeat(64), size: 1}}
    assert.throws(() => validateCaptureInputBindings(contract, urls, arbitrary), /identity descriptor is required/u)
  }
})

test("issue #1372 capture seam rejects an unbound create navigation", async () => {
  const contract = JSON.parse(await readFile(browserContractPath, "utf8"))
  const wrongDestination = structuredClone(contract)
  wrongDestination.subjects.find(({id}) => id === "control:create").loading.url_suffix = "/anywhere"
  assert.throws(() => validateTemporalRunContract(wrongDestination), /exact navigation destination and status/u)

  const wrongStatus = structuredClone(contract)
  wrongStatus.subjects.find(({id}) => id === "control:create").loading.status = 302
  assert.throws(() => validateTemporalRunContract(wrongStatus), /exact navigation destination and status/u)
})

test("issue #1372 capture seam rejects incomplete result oracles and vanished loading", async () => {
  const contract = JSON.parse(await readFile(browserContractPath, "utf8"))
  const urls = {
    denial: {missing_page: "https://framerail.invalid/denial-missing", saved_page: "https://framerail.invalid/denial-saved"},
    failure: {missing_page: "https://framerail.invalid/failure-missing", saved_page: "https://framerail.invalid/failure-saved"},
    success: {missing_page: "https://framerail.invalid/success-missing", saved_page: "https://framerail.invalid/success-saved"}
  }
  const incomplete = validCaptureInputs(contract, urls)
  delete incomplete.failureControl.descriptor.result_oracles.denial[contract.subjects[0].id]
  assert.throws(
    () => validateCaptureInputBindings(contract, urls, incomplete),
    /does not cover every subject/u
  )
  assert.throws(
    () => assertLoadingBoundaryPresent(false, "pane:append"),
    /loading predicate was not true at capture/u
  )
  assert.doesNotThrow(() => assertLoadingBoundaryPresent(true, "pane:append"))
})

test("issue #1372 capture seam rejects missing exact response status and oversized artifacts", async () => {
  const contract = JSON.parse(await readFile(browserContractPath, "utf8"))
  const urls = {
    denial: {missing_page: "https://framerail.invalid/denial-missing", saved_page: "https://framerail.invalid/denial-saved"},
    failure: {missing_page: "https://framerail.invalid/failure-missing", saved_page: "https://framerail.invalid/failure-saved"},
    success: {missing_page: "https://framerail.invalid/success-missing", saved_page: "https://framerail.invalid/success-saved"}
  }
  const invalid = validCaptureInputs(contract, urls)
  delete invalid.failureControl.descriptor.result_oracles.denial[contract.subjects[0].id].event.status
  assert.throws(() => validateCaptureInputBindings(contract, urls, invalid), /exact response status/u)
  assert.throws(() => assertByteLimit(Buffer.alloc(DOM_MAX_BYTES + 1), DOM_MAX_BYTES, "DOM artifact"), /byte limit/u)
  assert.throws(() => assertByteLimit(Buffer.alloc(SCREENSHOT_MAX_BYTES + 1), SCREENSHOT_MAX_BYTES, "screenshot artifact"), /byte limit/u)
  assert.throws(() => assertByteLimit(Buffer.alloc(DIAGNOSTIC_MAX_BYTES + 1), DIAGNOSTIC_MAX_BYTES, "diagnostics"), /byte limit/u)
})

test("issue #1372 storage states use run-owned copies and reject a pre-existing output root", async () => {
  const rootPath = await mkdtemp("/tmp/issue-1372-storage-")
  const outputDir = `${rootPath}/capture`
  const sourceStates = {}
  for (const scenario of ["denial", "failure", "success"]) {
    const sourcePath = `${rootPath}/${scenario}.json`
    const bytes = `{"cookies":[],"origins":[],"scenario":"${scenario}"}\n`
    await writeFile(sourcePath, bytes)
    sourceStates[scenario] = {path: sourcePath, sha256: createHash("sha256").update(bytes).digest("hex")}
  }
  await mkdir(outputDir)
  await assert.rejects(validateOutputPreflight(outputDir, []), /output directory already exists/u)
  await rm(outputDir, {recursive: true})
  await mkdir(outputDir)
  const targets = runOwnedStorageStatePaths(outputDir)
  const copies = await copyRunOwnedStorageStates(sourceStates, targets)
  assert.notEqual(copies.denial.path, sourceStates.denial.path)
  assert.equal((await lstat(copies.denial.path)).isFile(), true)
  await removeRunOwnedStorageStates(copies)
  await assert.rejects(lstat(copies.denial.path), {code: "ENOENT"})
  assert.equal((await lstat(sourceStates.denial.path)).isFile(), true)
  await rm(rootPath, {recursive: true})
})

test("issue #1372 cleanup closes an acquired proxy after partial startup", async () => {
  const closed = []
  const sourceProxy = {close: async () => closed.push("source")}
  const cleanup = await closeCaptureEgressProxies(sourceProxy, null)
  assert.deepEqual(closed, ["source"])
  assert.equal(cleanup.allClosed, false)
  assert.equal(cleanup.error, null)
})

test("issue #1372 cleanup bounds proxy shutdown", async () => {
  const cleanup = await closeCaptureEgressProxies({close: () => new Promise(() => {})}, null, 5)
  assert.equal(cleanup.allClosed, false)
  assert.match(cleanup.error?.message ?? "", /timed out after 5ms/u)
  assert.equal(SHUTDOWN_TIMEOUT_MS > 0, true)
})

test("issue #1372 rejects stale runtime source identity", () => {
  assert.throws(
    () => validateSourceIdentity(
      {wikijump_commit: "1".repeat(40), wikijump_tree: "2".repeat(40)},
      {wikijump_commit: "3".repeat(40), wikijump_tree: "2".repeat(40)},
    ),
    /actual clean capture source identity/u,
  )
})

test("issue #1372 rejects wrong navigation destination or status", () => {
  const event = {url_suffix: "/edit/true", status: 200}
  assert.throws(() => requireNavigationResponse({url: () => "https://candidate.invalid/wrong", status: () => 200}, event, "create"), /exact destination and status/u)
  assert.throws(() => requireNavigationResponse({url: () => "https://candidate.invalid/edit/true", status: () => 302}, event, "create"), /exact destination and status/u)
})

test("issue #1372 request url suffix matching ignores only a trailing query", () => {
  assert.equal(urlSuffixMatches("https://candidate.invalid/route-action-missing/edit/true/__data.json?x-sveltekit-invalidated=01", "/edit/true/__data.json"), true)
  assert.equal(urlSuffixMatches("https://candidate.invalid/page?/deletedGet", "?/deletedGet"), true)
  assert.equal(urlSuffixMatches("https://candidate.invalid/edit/true/__data.json?x-sveltekit-invalidated=01", "/edit/true"), false)
  assert.equal(urlSuffixMatches("https://candidate.invalid/edit/true/__data.json?other=1", "/edit/true/__data.json#frag"), false)
})

test("issue #1372 rejects a leftover run-owned storage state", async (t) => {
  const rootPath = await mkdtemp("/tmp/issue-1372-leftover-")
  t.after(() => rm(rootPath, {recursive: true, force: true}))
  const target = `${rootPath}/storage-state-denial.json`
  await writeFile(target, "{}\n")
  await assert.rejects(
    assertRunOwnedStorageStatesAbsent({denial: {path: target}}),
    /remains after cleanup/u,
  )
})
