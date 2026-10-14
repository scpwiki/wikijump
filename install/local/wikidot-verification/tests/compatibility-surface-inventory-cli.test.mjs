import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import fs from "node:fs/promises"
import os from "node:os"
import path from "node:path"
import test from "node:test"
import { fileURLToPath } from "node:url"

const toolRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..")
const repositoryRoot = path.resolve(toolRoot, "../../..")
const cliPath = path.join(toolRoot, "scripts/build-compatibility-surface-inventory.mjs")
const ftmlRevision = /ftml\s*=\s*\{[^\n]*\brev\s*=\s*"([0-9a-f]{40})"/u.exec(
  await fs.readFile(path.join(repositoryRoot, "deepwell/Cargo.toml"), "utf8")
)?.[1]
assert.ok(ftmlRevision, "deepwell/Cargo.toml must pin a full FTML commit")

const sha256 = (value) => createHash("sha256").update(value).digest("hex")
const supportedSourceLock = JSON.parse(await fs.readFile(
  path.join(repositoryRoot, "docs/development/wikidot-py-supported-source.json"),
  "utf8"
))
const wikidotPySource = {
  repository: supportedSourceLock.repository,
  commit: supportedSourceLock.commit,
  root_tree: supportedSourceLock.root_tree,
  objects: supportedSourceLock.objects
}
const wikidotPyModuleNames = [
  "changes/SiteChangesListModule",
  "dashboard/messages/DMInboxModule",
  "dashboard/messages/DMSentModule",
  "dashboard/messages/DMViewMessageModule",
  "edit/EditMetaModule",
  "files/PageFilesModule",
  "forum/ForumCommentsListModule",
  "forum/ForumStartModule",
  "forum/ForumViewCategoryModule",
  "forum/ForumViewThreadModule",
  "forum/ForumViewThreadPostsModule",
  "forum/sub/ForumEditPostFormModule",
  "forum/sub/ForumPostRevisionModule",
  "forum/sub/ForumPostRevisionsModule",
  "history/PageRevisionListModule",
  "history/PageSourceModule",
  "history/PageVersionModule",
  "list/ListPagesModule",
  "managesite/ManageSiteMembersApplicationsModule",
  "membership/MembersListModule",
  "pagerate/WhoRatedPageModule",
  "viewsource/ViewSourceModule"
]
const pageActionsFixtureSource = `export const pageActions = {
  edit: editAction,
  deletedGet: deletedGetAction,
  restore: restoreAction
}
`

function cleanupFixture(t, root) {
  t.after(() => fs.rm(root, { recursive: true, force: true }))
}

const listPagesParameters = [
  "p", "pagetype", "page_type", "page-type", "category", "tags", "tag", "parent",
  "created_at", "createdat", "updated_at", "updatedat", "created_by", "createdby",
  "rating", "score", "name", "fullname", "full_slug", "fullslug", "range", "order",
  "offset", "limit", "perpage", "per_page", "separate", "wrapper", "rss", "rsstitle",
  "rssdescription", "rsshome", "rsslimit", "rssonly"
]

async function writeJson(root, relativePath, value) {
  const target = path.join(root, relativePath)
  await fs.mkdir(path.dirname(target), { recursive: true })
  await fs.writeFile(target, `${JSON.stringify(value, null, 2)}\n`)
}

async function writeImplementationLedgerMirrors(root, value) {
  for (const ledgerPath of [
    "docs/wikidot-specifications/implementation-ledger.json",
    "scripts/data/wikidot-implementation-ledger.json"
  ]) {
    await writeJson(root, ledgerPath, value)
  }
}

async function writeText(root, relativePath, value) {
  const target = path.join(root, relativePath)
  await fs.mkdir(path.dirname(target), { recursive: true })
  await fs.writeFile(target, value)
}

async function writeRepositoryFixture(root) {
  for (const file of [
    "catalog-feature-owners.json",
    "deepwell-jsonrpc.json",
    "framerail-actions.json",
    "wws-wikidot-py.json"
  ]) {
    await writeText(
      root,
      `install/local/wikidot-verification/fixtures/compatibility-ownership/${file}`,
      await fs.readFile(
        path.join(
          repositoryRoot,
          "install/local/wikidot-verification/fixtures/compatibility-ownership",
          file
        ),
        "utf8"
      )
    )
  }
  const semantics = JSON.parse(
    await fs.readFile(
      path.join(repositoryRoot, "docs/development/compatibility-surface-semantics.json"),
      "utf8"
    )
  )
  semantics.ftml.catalog_crosswalk = []
  const fixtureLegacyOwners = [
    "Rokurolize/wikidot.py", "deepwell", "docs/wikidot-specifications", "framerail", "wws"
  ]
  semantics.implementation_owners_by_legacy_owner = Object.fromEntries(
    fixtureLegacyOwners.map((owner) => [owner, semantics.implementation_owners_by_legacy_owner[owner]])
  )
  semantics.specification_owner_keys = [
    ...Object.values(semantics.specification_owner_by_kind),
    "catalog.feature:feature-one",
    "open43.case:F123_PUBLIC_CASE"
  ].sort().filter((owner, index, owners) => owner !== owners[index - 1])
  semantics.implementation_owner_keys = [
    ...new Set(Object.values(semantics.implementation_owners_by_legacy_owner).flat())
  ].sort()
  await writeJson(root, "docs/development/compatibility-surface-semantics.json", semantics)
  const catalog = {
    feature_count: 1,
    features: [
      {
        id: "feature-one",
        documentation_status: "documented",
        specification: "specifications/core/feature-one.md",
        sources: [
          {
            path: "fixture/feature-one.txt",
            source_sha256: "1".repeat(64)
          }
        ],
        live_observation_ids: ["observation-one"]
      }
    ]
  }
  await writeJson(root, "docs/wikidot-specifications/catalog.json", catalog)
  await writeJson(root, "docs/wikidot-specifications/live-observations.json", {
    observations: [
      {
        id: "observation-one",
        feature_ids: ["feature-one"]
      }
    ]
  })
  await writeJson(root, "docs/wikidot-specifications/source-coverage.json", {
    page_count: 1,
    listed_page_count: 1,
    excluded_data_record_count: 0,
    unclassified_count: 0,
    classification_counts: { documentation: 1 },
    pages: [
      {
        source_path: "fixture/feature-one.txt",
        source_sha256: "1".repeat(64),
        classification: "documentation",
        feature_ids: ["feature-one"]
      }
    ]
  })
  await writeJson(root, "docs/development/compatibility-catalog-source-attribution.json", {
    schema: "wikijump.compatibility_catalog_source_attribution.v1",
    records: []
  })
  const implementationLedger = {
    catalog_sha256: sha256(`${JSON.stringify(catalog, null, 2)}\n`),
    features: {
      "feature-one": {
        status: "pending",
        tests: [],
        implementation_files: [],
        documentation_evidence: ["specifications/core/feature-one.md"],
        live_oracle_evidence: []
      }
    }
  }
  await writeImplementationLedgerMirrors(root, implementationLedger)
  const deepwellApiSource = 'register!("ping", ping);\n'
  await writeText(root, "deepwell/src/api.rs", deepwellApiSource)
  await writeJson(root, "docs/development/deepwell-jsonrpc-contract-manifest.json", {
    schema: "wikijump.deepwell_jsonrpc_contract_manifest.v1",
    source_identities: {
      jsonrpc_registry: {
        path: "deepwell/src/api.rs",
        sha256: sha256(deepwellApiSource)
      }
    },
    method_count: 1,
    methods: [
      {
        method: "ping",
        endpoint_owner: {
          component: "deepwell",
          source: "deepwell/src/api.rs#ping",
          source_sha256: sha256(deepwellApiSource)
        },
        test_witness: {
          kind: "source_contract_only",
          reference: "fixture/deepwell-contract.test.mjs#ping"
        }
      }
    ]
  })
  await writeText(
    root,
    "deepwell/Cargo.toml",
    `ftml = { git = "https://github.com/Rokurolize/ftml", rev = "${ftmlRevision}" }\n`
  )
  await writeText(
    root,
    "deepwell/Cargo.lock",
    `source = "git+https://github.com/Rokurolize/ftml?rev=${ftmlRevision}#${ftmlRevision}"\n`
  )
  await writeJson(root, "docs/development/wikidot-page-action-surfaces.json", {
    schema: "wikijump.wikidot_page_action_surface_registry.v2",
    evidence_references: ["evidence/page-shell.html"],
    surfaces: [
      {
        action_id: "edit",
        source_status: "implemented",
        public_references: ["framerail/src/routes/+page.svelte"],
        test_references: ["tests/page-edit.test.js"]
      }
    ],
    missing_page_controls: [
      {
        control_id: "create",
        source_status: "implemented",
        operation_bindings: [
          {
            operation_id: "edit",
            public_references: [
              "framerail/src/lib/server/load/page/page-actions.ts#action:edit"
            ]
          }
        ],
        observable_states: [
          "missing-page-settled",
          "editor-loading",
          "editor-settled",
          "save-loading",
          "save-success",
          "save-denial",
          "save-failure",
          "created-page-settled"
        ],
        browser_interval_proof: { status: "missing", issue: 1372 },
        source_identities: [
          { path: "framerail/src/routes/+page.svelte" },
          { path: "framerail/src/lib/server/load/page/page-actions.ts" }
        ],
        issues: [1371, 1372],
        test_references: ["tests/missing-page-create.spec.ts"]
      },
      {
        control_id: "restore",
        source_status: "implemented",
        operation_bindings: [
          {
            operation_id: "deletedGet",
            public_references: [
              "framerail/src/lib/server/load/page/page-actions.ts#action:deletedGet"
            ]
          },
          {
            operation_id: "restore",
            public_references: [
              "framerail/src/lib/server/load/page/page-actions.ts#action:restore"
            ]
          }
        ],
        observable_states: [
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
        ],
        browser_interval_proof: { status: "missing", issue: 1372 },
        source_identities: [
          { path: "framerail/src/routes/+page.svelte" },
          { path: "framerail/src/lib/server/load/page/page-actions.ts" }
        ],
        issues: [1371, 1372],
        test_references: ["tests/missing-page-restore.spec.ts"]
      }
    ]
  })
  await writeText(root, "framerail/src/routes/+page.svelte", "<h1>Fixture</h1>\n")
  await writeText(
    root,
    "framerail/src/routes/+page.server.ts",
    "export const actions = { edit: editAction }\n"
  )
  await writeText(
    root,
    "framerail/src/lib/server/load/page/page-actions.ts",
    pageActionsFixtureSource
  )
  await writeText(
    root,
    "framerail/src/lib/server/ajax-module-connector.js",
    `import { classifyWikidotSiteChangesRequest } from "./wikidot-site-changes.js"
const FORUM_READ_MODULE_PARAMETERS = new Map([
  ["forum/ForumStartModule", [new Set()]]
])
const SITE_CHANGES_MODULE = "changes/SiteChangesListModule"
const MEMBERS_LIST_MODULE = "membership/MembersListModule"
const MEMBERS_LIST_PARAMETERS = new Set(["group", "page"])
const MEMBERS_LIST_DEFAULT_PARAMETERS = new Set(["group"])
const PAGE_READ_MODULE_PARAMETERS = new Map([
  ["viewsource/ViewSourceModule", new Set(["page_id"])],
  ["files/PageFilesModule", new Set(["page_id"])]
])
const LIST_PAGES_PARAMETERS = new Set(${JSON.stringify(listPagesParameters)})
const NEWPAGE_ACTION = "misc/NewPageHelperAction"
const NEWPAGE_EVENT = "createNewPage"
const PAGE_DISCUSSION_ACTION = "ForumAction"
const PAGE_DISCUSSION_EVENT = "createPageDiscussionThread"
if (moduleName !== "list/ListPagesModule") throw new Error()
`
  )
  await writeJson(root, "docs/development/framerail-amc-wire-contracts.json", {
    schema: "wikijump.framerail_amc_wire_contracts.v1",
    modules: [
      {
        module_name: "list/ListPagesModule",
        allowed_parameters: listPagesParameters,
        required_fields: [],
        module_body: "optional_default_template",
        parameter_order: "insignificant",
        duplicate_fields: "last_value",
        unknown_parameters: "non_data_form_ignored;leading_underscore_rejected",
        value_type: "urlencoded_utf8_string",
        callback_index: "accepted_ignored",
        authentication: "cookies_ignored;wikidot_token7_accepted_ignored",
        success_envelope: "status=ok;body=string",
        failure_envelopes: [
          "render_failure:status=not_ok;message=Unable to render ListPages module"
        ],
        implementation_references: [
          "framerail/src/lib/server/ajax-module-connector.js",
          "deepwell/src/services/render/list_pages/ajax.rs"
        ]
      }
    ]
  })
  await writeText(
    root,
    "framerail/src/lib/server/wikidot-site-changes.js",
    `const BROWSER_FIELDS = new Set(["page", "perpage", "pageId", "categoryId", "options"])
const WIKIDOT_PY_FIELDS = new Set(["page", "perpage", "options"])
export const classifyWikidotSiteChangesRequest = (fields) => fields
`
  )
  await writeText(
    root,
    "install/local/wikidot-verification/artifacts/open43-readonly-live-20260810.json",
    await fs.readFile(
      path.join(
        repositoryRoot,
        "install/local/wikidot-verification/artifacts/open43-readonly-live-20260810.json"
      ),
      "utf8"
    )
  )
  await writeText(
    root,
    "framerail/src/lib/server/xmlrpc/methods.ts",
    `const METHOD_DEFINITIONS = {
  "system.listMethods": { help: "fixture", signatures: [["array"]] }
}
`
  )
  await writeJson(root, "docs/development/wikidot-py-supported-source.json", supportedSourceLock)
  await writeJson(root, "docs/development/wikidot-py-amc-client-parity.json", {
    schema: "wikijump.wikidot_py_amc_client_parity.v1",
    source: wikidotPySource,
    modules: wikidotPyModuleNames.map((moduleName) => ({
      module_name: moduleName,
      parameters: [],
      status: "unsupported_unevidenced",
      gap: "fixture gap",
      source_reference: "src/wikidot/module/fixture.py#fixture"
    }))
  })
  await writeText(
    root,
    "wws/src/route.rs",
    `pub fn build_router() {
  Router::new().route("/-/health-check", get(handle_health_check))
}
`
  )

  const audits = Array.from(
    { length: 7 },
    (_, index) => `docs/development/open43-audit-${index + 1}.json`
  )
  const closureAudits = []
  for (const [index, audit] of audits.entries()) {
    const value = {
      schema: `wikijump.open43.fixture_${index + 1}.v1`,
      issues:
        index === 0
          ? [
              {
                issue: 123,
                subrows: [
                  {
                    case_id: "F123_PUBLIC_CASE",
                    classification: "candidate_required",
                    owner: "deepwell",
                    existing_tests: ["tests/public.test.mjs#public case"]
                  }
                ]
              }
            ]
          : []
    }
    const serialized = `${JSON.stringify(value, null, 2)}\n`
    await writeText(root, audit, serialized)
    closureAudits.push({
      path: audit,
      sha256: sha256(serialized),
      issue_count: index === 0 ? 1 : 0,
      case_count: index === 0 ? 1 : 0,
      classification_counts: {
        source_ready: 0,
        needs_source: 0,
        candidate_required: index === 0 ? 1 : 0,
        blocked_evidence: 0
      }
    })
  }
  const routeClasses = Object.fromEntries([
    "anonymous_read_only", "authenticated_read_only", "run_owned_mutation", "local_candidate",
    "live_browser_only", "missing_public_producer", "missing_architecture_domain_authority",
    "missing_security_policy"
  ].map((name) => [name, `${name} fixture route`]))
  await writeJson(root, "docs/development/open43-blocked-evidence-routing.json", {
    schema: "wikijump.open43.blocked_evidence_routing.v1",
    source_audits: audits,
    route_classes: routeClasses,
    counts: { ...Object.fromEntries(Object.keys(routeClasses).map((name) => [name, 0])), total: 0 },
    rows: []
  })
  await writeJson(root, "docs/development/open43-closure-audit-ownership-reconciliation.json", {
    schema: "wikijump.open43.closure_audit_ownership_reconciliation.v1",
    closure_audits: closureAudits,
    after: {
      case_count: 1,
      unique_case_count: 1,
      duplicate_case_ids: [],
      unknown_classifications: [],
      classification_counts: {
        source_ready: 0,
        needs_source: 0,
        candidate_required: 1,
        blocked_evidence: 0
      }
    }
  })
  for (const args of [
    ["init", "-q"],
    ["add", "."],
    ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "fixture"]
  ]) {
    const result = spawnSync("git", args, { cwd: root, encoding: "utf8" })
    assert.equal(result.status, 0, result.stderr)
  }
  const sourceRevision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8"
  }).stdout.trim()
  const reconciliationPath = path.join(
    root,
    "docs/development/open43-closure-audit-ownership-reconciliation.json"
  )
  const reconciliation = JSON.parse(await fs.readFile(reconciliationPath, "utf8"))
  for (const audit of reconciliation.closure_audits) audit.source_revision = sourceRevision
  await fs.writeFile(reconciliationPath, `${JSON.stringify(reconciliation, null, 2)}\n`)
  const commit = spawnSync(
    "git",
    ["add", "."],
    { cwd: root, encoding: "utf8" }
  )
  assert.equal(commit.status, 0, commit.stderr)
  const committed = spawnSync("git", [
    "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
    "commit", "-qm", "bind audit revisions"
  ], {
    cwd: root,
    encoding: "utf8"
  })
  assert.equal(committed.status, 0, committed.stderr)
}

async function replacePageActionsFixture(root, actionSource) {
  const actionPath = "framerail/src/lib/server/load/page/page-actions.ts"
  await writeText(root, actionPath, actionSource)
}

async function refreshCatalogHash(root) {
  const catalogPath = path.join(root, "docs/wikidot-specifications/catalog.json")
  const catalogHash = sha256(await fs.readFile(catalogPath))
  for (const ledgerPath of [
    "docs/wikidot-specifications/implementation-ledger.json",
    "scripts/data/wikidot-implementation-ledger.json"
  ]) {
    const absolutePath = path.join(root, ledgerPath)
    const ledger = JSON.parse(await fs.readFile(absolutePath, "utf8"))
    ledger.catalog_sha256 = catalogHash
    await fs.writeFile(absolutePath, `${JSON.stringify(ledger, null, 2)}\n`)
  }
}

async function writeCatalogOwnerFixture(root, ownerManifest = {
  issue_scope: { status: "unresolved", references: [] },
  owners: [{
    owner: "wikijump.deepwell",
    source_references: ["deepwell/src/api.rs"],
    test_references: ["tests/data-form.test.js#fixture"]
  }]
}, specification = "specifications/data-forms/fixture.md", featureId = "data-forms-fixture") {
  await writeRepositoryFixture(root)
  const catalogPath = path.join(root, "docs/wikidot-specifications/catalog.json")
  const coveragePath = path.join(root, "docs/wikidot-specifications/source-coverage.json")
  const semanticsPath = path.join(root, "docs/development/compatibility-surface-semantics.json")
  const observationsPath = path.join(root, "docs/wikidot-specifications/live-observations.json")
  const catalog = JSON.parse(await fs.readFile(catalogPath, "utf8"))
  const feature = catalog.features[0]
  feature.id = featureId
  feature.specification = specification
  const semantics = JSON.parse(await fs.readFile(semanticsPath, "utf8"))
  semantics.specification_owner_keys = semantics.specification_owner_keys
    .map((owner) => owner === "catalog.feature:feature-one" ? `catalog.feature:${featureId}` : owner)
    .sort()
  const coverage = JSON.parse(await fs.readFile(coveragePath, "utf8"))
  coverage.pages[0].feature_ids = [feature.id]
  const observations = JSON.parse(await fs.readFile(observationsPath, "utf8"))
  observations.observations[0].feature_ids = [feature.id]
  const ledger = {
    catalog_sha256: "",
    features: {
      [feature.id]: {
        status: "implemented",
        tests: [{ path: "tests/data-form.test.js", name: "fixture" }],
        implementation_files: ["deepwell/src/api.rs"],
        documentation_evidence: [specification],
        live_oracle_evidence: []
      }
    },
    implementation_owner_records: { [feature.id]: ownerManifest }
  }
  await fs.writeFile(catalogPath, `${JSON.stringify(catalog, null, 2)}\n`)
  await fs.writeFile(semanticsPath, `${JSON.stringify(semantics, null, 2)}\n`)
  await fs.writeFile(coveragePath, `${JSON.stringify(coverage, null, 2)}\n`)
  await fs.writeFile(observationsPath, `${JSON.stringify(observations, null, 2)}\n`)
  for (const ledgerPath of [
    "docs/wikidot-specifications/implementation-ledger.json",
    "scripts/data/wikidot-implementation-ledger.json"
  ]) {
    await writeJson(root, ledgerPath, ledger)
  }
  await writeText(root, "tests/data-form.test.js", "export function fixture() {}\n")
  await refreshCatalogHash(root)
}

function runCli(root, outputPath, env = process.env) {
  const changedTrackedFiles = spawnSync(
    "git",
    ["status", "--short", "--untracked-files=no"],
    { cwd: root, encoding: "utf8" }
  ).stdout.trim()
  if (changedTrackedFiles) {
    const added = spawnSync("git", ["add", "-u"], { cwd: root, encoding: "utf8" })
    assert.equal(added.status, 0, added.stderr)
    const committed = spawnSync("git", [
      "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
      "commit", "-qm", "fixture mutation"
    ], {
      cwd: root,
      encoding: "utf8"
    })
    assert.equal(committed.status, 0, committed.stderr)
  }
  const sourceRevision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8"
  }).stdout.trim()
  return spawnSync(process.execPath, [
    cliPath,
    "--root",
    root,
    "--output",
    outputPath,
    "--source-revision",
    sourceRevision
  ], {
    encoding: "utf8",
    env
  })
}

test("CLI rejects supported wikidot.py source identity drift", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-inventory-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  const contractPath = path.join(root, "docs/development/wikidot-py-amc-client-parity.json")
  const contract = JSON.parse(await fs.readFile(contractPath, "utf8"))
  const drifts = [
    (source) => { source.repository = "example/wikidot.py" },
    (source) => { source.commit = "a".repeat(40) },
    (source) => { source.root_tree = "a".repeat(40) },
    (source) => { source.objects.find(({ path }) => path === "src/wikidot").oid = "a".repeat(40) },
    (source) => { source.objects.find(({ path }) => path === "src/wikidot/module").oid = "a".repeat(40) },
    (source) => { source.objects.find(({ path }) => path === "src/wikidot/connector").oid = "a".repeat(40) },
    (source) => { source.objects.find(({ path }) => path === "src/wikidot/connector/ajax.py").oid = "a".repeat(40) },
    (source) => { source.objects.find(({ path }) => path === "pyproject.toml").oid = "a".repeat(40) },
    (source) => { source.objects.find(({ path }) => path === "uv.lock").oid = "a".repeat(40) }
  ]

  for (const drift of drifts) {
    const changed = structuredClone(contract)
    drift(changed.source)
    await fs.writeFile(contractPath, `${JSON.stringify(changed, null, 2)}\n`)
    const result = runCli(root, path.join(root, "inventory.json"))
    assert.notEqual(result.status, 0)
    assert.match(result.stderr, /wikidot-py-amc-client-parity\.json source identity drift/u)
  }
})

test("CLI rejects an omitted or duplicate pinned wikidot.py module", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-inventory-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  const contractPath = path.join(root, "docs/development/wikidot-py-amc-client-parity.json")
  const contract = JSON.parse(await fs.readFile(contractPath, "utf8"))

  for (const modules of [contract.modules.slice(1), [...contract.modules, contract.modules[0]]]) {
    await fs.writeFile(contractPath, `${JSON.stringify({ ...contract, modules }, null, 2)}\n`)
    const result = runCli(root, path.join(root, "inventory.json"))
    assert.notEqual(result.status, 0)
    assert.match(result.stderr, /wikidot-py-amc-client-parity\.json module denominator drift/u)
  }
})

test("CLI discovers declared public surfaces and writes deterministic completion fields", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-inventory-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  const outputPath = path.join(root, "inventory.json")

  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  assert.equal(result.stdout, "wrote 51 compatibility surfaces to inventory.json\n")
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  assert.equal(inventory.schema, "wikijump.compatibility_surface_inventory.v3")
  assert.equal(inventory.sources.live_observations, "docs/wikidot-specifications/live-observations.json")
  assert.equal(inventory.sources.source_coverage, "docs/wikidot-specifications/source-coverage.json")
  assert.deepEqual(Object.keys(inventory.provenance.wikijump), ["source_input_set_sha256"])
  assert.match(inventory.provenance.wikijump.source_input_set_sha256, /^[0-9a-f]{64}$/u)
  assert.equal(inventory.provenance.ftml.commit, ftmlRevision)
  assert.match(inventory.provenance.ftml.tree, /^[0-9a-f]{40}$/u)
  assert.ok(inventory.provenance.registries.length > 10)
  assert.equal(
    new Set(inventory.provenance.registries.map(({ path: registryPath }) => registryPath)).size,
    inventory.provenance.registries.length
  )
  assert.ok(inventory.provenance.registries.every(({ sha256: digest }) => /^[0-9a-f]{64}$/u.test(digest)))
  for (const registry of inventory.provenance.registries) {
    assert.equal(registry.sha256, sha256(await fs.readFile(path.join(root, registry.path))))
  }
  for (const requiredPath of [
    "docs/wikidot-specifications/catalog.json",
    "docs/wikidot-specifications/implementation-ledger.json",
    "scripts/data/wikidot-implementation-ledger.json",
    "docs/wikidot-specifications/live-observations.json",
    "docs/wikidot-specifications/source-coverage.json",
    "deepwell/src/api.rs",
    "framerail/src/lib/server/ajax-module-connector.js",
    "framerail/src/lib/server/xmlrpc/methods.ts",
    "wws/src/route.rs"
  ]) {
    assert.ok(inventory.provenance.registries.some(({ path: registryPath }) => registryPath === requiredPath))
  }
  assert.deepEqual(inventory.sources.wikidot_py_source, wikidotPySource)
  assert.deepEqual(inventory.counts, {
    total: 51,
    by_kind: {
      catalog_feature: 1,
      deepwell_jsonrpc_method: 1,
      framerail_amc_action_shape: 2,
      framerail_amc_module_shape: 16,
      framerail_route: 1,
      framerail_server_action: 1,
      framerail_xmlrpc_method: 1,
      missing_page_control: 2,
      open43_audit_case: 1,
      page_action: 1,
      wikidot_py_amc_module_shape: 22,
      wws_route: 2
    }
  })
  const listPagesSurfaces = inventory.surfaces
    .filter(({ surface_id }) => surface_id.startsWith("framerail-amc-module:list/ListPagesModule"))
    .map(({ surface_id }) => surface_id)
  assert.equal(listPagesSurfaces.length, 9)
  assert.ok(listPagesSurfaces.some((id) => id.includes(`parameters=${[...listPagesParameters].sort().join(",")}`)))
  assert.ok(listPagesSurfaces.includes("framerail-amc-module:list/ListPagesModule:parameter-order=insignificant"))
  assert.ok(listPagesSurfaces.includes("framerail-amc-module:list/ListPagesModule:duplicate-fields=last_value"))
  assert.ok(listPagesSurfaces.includes("framerail-amc-module:list/ListPagesModule:unknown-parameters=non_data_form_ignored;leading_underscore_rejected"))
  assert.ok(listPagesSurfaces.includes("framerail-amc-module:list/ListPagesModule:value-type=urlencoded_utf8_string"))
  assert.ok(listPagesSurfaces.includes("framerail-amc-module:list/ListPagesModule:callback-index=accepted_ignored"))
  assert.ok(listPagesSurfaces.includes("framerail-amc-module:list/ListPagesModule:authentication=cookies_ignored;wikidot_token7_accepted_ignored"))
  assert.ok(listPagesSurfaces.includes("framerail-amc-module:list/ListPagesModule:success-envelope=status=ok;body=string"))
  assert.ok(listPagesSurfaces.includes("framerail-amc-module:list/ListPagesModule:failure-envelopes=render_failure:status=not_ok;message=Unable to render ListPages module"))
  assert.ok(listPagesSurfaces.every((id) => !id.includes("*")))
  assert.deepEqual(
    inventory.surfaces
      .filter(({ surface_id }) => surface_id.startsWith("framerail-amc-module:changes/SiteChangesListModule"))
      .map(({ surface_id }) => surface_id),
    [
      "framerail-amc-module:changes/SiteChangesListModule:parameters=categoryId,options,page,pageId,perpage",
      "framerail-amc-module:changes/SiteChangesListModule:parameters=options,page,perpage"
    ]
  )
  const siteChangesBrowser = inventory.surfaces.find(
    ({ surface_id }) =>
      surface_id ===
      "framerail-amc-module:changes/SiteChangesListModule:parameters=categoryId,options,page,pageId,perpage"
  )
  assert.deepEqual(siteChangesBrowser.evidence, {
    status: "available",
    references: [
      "install/local/wikidot-verification/artifacts/open43-readonly-live-20260810.json#E_OPEN43_SITECHANGES_AMC_20260810"
    ]
  })
  assert.deepEqual(
    inventory.surfaces
      .filter(({ surface_id }) => surface_id.includes("/-/health-check"))
      .map(({ surface_id }) => surface_id),
    [
      "wws-route:GET:/-/health-check",
      "wws-route:HEAD:/-/health-check"
    ]
  )
  const caseSurface = inventory.surfaces.find(
    ({ surface_id }) => surface_id === "open43-audit-case:F123_PUBLIC_CASE"
  )
  assert.deepEqual(caseSurface, {
    surface_id: "open43-audit-case:F123_PUBLIC_CASE",
    kind: "open43_audit_case",
    specification_owner: "open43.case:F123_PUBLIC_CASE",
    implementation_owners: ["wikijump.deepwell"],
    public_reference: [
      "docs/development/open43-audit-1.json#F123_PUBLIC_CASE"
    ],
    existing_refs: {
      issues: [123],
      cases: ["F123_PUBLIC_CASE"],
      tests: ["tests/public.test.mjs#public case"]
    },
    evidence: { status: "available", references: [] },
    source: { status: "implemented", references: [] },
    candidate: {
      status: "pending",
      references: ["candidate-case:F123_PUBLIC_CASE"]
    },
    standing: { status: "pending", references: [] },
    closure: { status: "open", references: [] }
  })
  const missingPageControls = inventory.surfaces.filter(
    ({ kind }) => kind === "missing_page_control"
  )
  assert.deepEqual(
    missingPageControls.map(({ surface_id }) => surface_id),
    ["missing-page-control:create", "missing-page-control:restore"]
  )
  assert.deepEqual(missingPageControls[0].operation_bindings, [
    {
      operation_id: "edit",
      public_references: ["framerail/src/lib/server/load/page/page-actions.ts#action:edit"]
    }
  ])
  assert.deepEqual(missingPageControls[1].operation_bindings, [
    {
      operation_id: "deletedGet",
      public_references: [
        "framerail/src/lib/server/load/page/page-actions.ts#action:deletedGet"
      ]
    },
    {
      operation_id: "restore",
      public_references: ["framerail/src/lib/server/load/page/page-actions.ts#action:restore"]
    }
  ])
  assert.deepEqual(
    missingPageControls.map(({ browser_interval_proof }) => browser_interval_proof),
    [
      { status: "missing", issue: 1372 },
      { status: "missing", issue: 1372 }
    ]
  )
  assert.deepEqual(missingPageControls[0].observable_states, [
    "missing-page-settled",
    "editor-loading",
    "editor-settled",
    "save-loading",
    "save-success",
    "save-denial",
    "save-failure",
    "created-page-settled"
  ])
  assert.deepEqual(missingPageControls[1].observable_states, [
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
  ])
  assert.deepEqual(missingPageControls[0].source_identities, [
    {
      path: "framerail/src/routes/+page.svelte",
      sha256: "b14d226020649601ba32153ab426b08d816efbdaa68fd0618cf3fe6143ee0fee"
    },
    {
      path: "framerail/src/lib/server/load/page/page-actions.ts",
      sha256: sha256(pageActionsFixtureSource)
    }
  ])
  assert.deepEqual(
    missingPageControls.map(({ evidence }) => evidence),
    [
      { status: "missing", references: [] },
      { status: "missing", references: [] }
    ]
  )
  assert.deepEqual(
    inventory.surfaces.map(({ surface_id }) => surface_id),
    [...inventory.surfaces.map(({ surface_id }) => surface_id)].sort()
  )

  const firstOutput = await fs.readFile(outputPath, "utf8")
  const secondResult = runCli(root, outputPath)
  assert.equal(secondResult.status, 0, secondResult.stderr)
  assert.equal(await fs.readFile(outputPath, "utf8"), firstOutput)
})

test("CLI rejects Catalog provenance, record, and source-edge drift", async (t) => {
  const cases = [
    {
      name: "catalog hash",
      mutate: async (root) => {
        const ledgerPath = path.join(root, "docs/wikidot-specifications/implementation-ledger.json")
        const ledger = JSON.parse(await fs.readFile(ledgerPath, "utf8"))
        ledger.catalog_sha256 = "0".repeat(64)
        await writeImplementationLedgerMirrors(root, ledger)
      },
      error: /catalog_sha256 does not match/u
    },
    {
      name: "missing ledger record",
      mutate: async (root) => {
        const ledgerPath = path.join(root, "docs/wikidot-specifications/implementation-ledger.json")
        const ledger = JSON.parse(await fs.readFile(ledgerPath, "utf8"))
        delete ledger.features["feature-one"]
        await writeImplementationLedgerMirrors(root, ledger)
      },
      error: /catalog feature has no ledger entry: feature-one/u
    },
    {
      name: "duplicate catalog record",
      mutate: async (root) => {
        const catalogPath = path.join(root, "docs/wikidot-specifications/catalog.json")
        const catalog = JSON.parse(await fs.readFile(catalogPath, "utf8"))
        catalog.features.push(structuredClone(catalog.features[0]))
        catalog.feature_count += 1
        await fs.writeFile(catalogPath, `${JSON.stringify(catalog, null, 2)}\n`)
        await refreshCatalogHash(root)
      },
      error: /duplicate catalog feature: feature-one/u
    },
    {
      name: "duplicate source edge",
      mutate: async (root) => {
        const coveragePath = path.join(root, "docs/wikidot-specifications/source-coverage.json")
        const coverage = JSON.parse(await fs.readFile(coveragePath, "utf8"))
        coverage.pages[0].feature_ids.push("feature-one")
        await fs.writeFile(coveragePath, `${JSON.stringify(coverage, null, 2)}\n`)
      },
      error: /has duplicate feature edges/u
    },
    {
      name: "missing source record",
      mutate: async (root) => {
        const coveragePath = path.join(root, "docs/wikidot-specifications/source-coverage.json")
        const coverage = JSON.parse(await fs.readFile(coveragePath, "utf8"))
        coverage.pages = []
        coverage.page_count = 0
        coverage.listed_page_count = 0
        coverage.classification_counts.documentation = 0
        await fs.writeFile(coveragePath, `${JSON.stringify(coverage, null, 2)}\n`)
      },
      error: /source coverage drift/u
    }
  ]

  for (const fixtureCase of cases) {
    await t.test(fixtureCase.name, async (t) => {
      const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-catalog-provenance-"))
      cleanupFixture(t, root)
      await writeRepositoryFixture(root)
      await fixtureCase.mutate(root)

      const result = runCli(root, path.join(root, "inventory.json"))

      assert.equal(result.status, 1, result.stderr)
      assert.match(result.stderr, fixtureCase.error)
    })
  }
})

test("CLI rejects FTML identity drift and ignores inherited Git controls", async (t) => {
  const driftRoot = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-ftml-drift-"))
  cleanupFixture(t, driftRoot)
  await writeRepositoryFixture(driftRoot)
  await writeText(
    driftRoot,
    "deepwell/Cargo.lock",
    'source = "git+https://github.com/Rokurolize/ftml?rev=0000000000000000000000000000000000000000#0000000000000000000000000000000000000000"\n'
  )
  const drift = runCli(driftRoot, path.join(driftRoot, "inventory.json"))
  assert.equal(drift.status, 1, drift.stderr)
  assert.match(drift.stderr, /FTML manifest and lock identities do not match/u)

  const isolatedRoot = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-git-isolation-"))
  cleanupFixture(t, isolatedRoot)
  await writeRepositoryFixture(isolatedRoot)
  const isolated = runCli(isolatedRoot, path.join(isolatedRoot, "inventory.json"), {
    ...process.env,
    GIT_DIR: path.join(isolatedRoot, "missing.git"),
    GIT_WORK_TREE: path.join(isolatedRoot, "missing-worktree"),
    PATH: path.join(isolatedRoot, "missing-bin")
  })
  assert.equal(isolated.status, 0, isolated.stderr)
})

test("CLI keeps an explicit source revision across metadata commits and rejects registry drift", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-source-revision-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  const sourceRevision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8"
  }).stdout.trim()
  const firstOutput = path.join(root, "first-inventory.json")
  const first = spawnSync(
    process.execPath,
    [cliPath, "--root", root, "--output", firstOutput, "--source-revision", sourceRevision],
    { encoding: "utf8" }
  )
  assert.equal(first.status, 0, first.stderr)

  const generatorPath = "install/local/wikidot-verification/scripts/build-compatibility-surface-inventory.mjs"
  await writeText(root, generatorPath, "// generator metadata only\n")
  assert.equal(
    spawnSync("git", ["add", generatorPath, path.basename(firstOutput)], { cwd: root }).status,
    0
  )
  assert.equal(spawnSync("git", [
    "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
    "commit", "-qm", "metadata only"
  ], { cwd: root }).status, 0)
  const secondOutput = path.join(root, "second-inventory.json")
  const second = spawnSync(
    process.execPath,
    [cliPath, "--root", root, "--output", secondOutput, "--source-revision", sourceRevision],
    { encoding: "utf8" }
  )
  assert.equal(second.status, 0, second.stderr)
  assert.equal(await fs.readFile(secondOutput, "utf8"), await fs.readFile(firstOutput, "utf8"))
  assert.match(
    JSON.parse(await fs.readFile(secondOutput, "utf8")).provenance.wikijump.source_input_set_sha256,
    /^[0-9a-f]{64}$/u
  )

  await fs.appendFile(path.join(root, "deepwell/src/api.rs"), 'register!("drift", drift);\n')
  const drift = spawnSync(
    process.execPath,
    [cliPath, "--root", root, "--output", path.join(root, "drift.json"), "--source-revision", sourceRevision],
    { encoding: "utf8" }
  )
  assert.equal(drift.status, 1, drift.stderr)
  assert.match(
    drift.stderr,
    /deepwell-jsonrpc-contract-manifest\.json (?:does not match the Deepwell method denominator|JSON-RPC registry identity drift)|registry blob drift: deepwell\/src\/api\.rs/u
  )
})

test("CLI emits the pinned FTML raw manifest without changing the public denominator", async (t) => {
  const outputRoot = await fs.mkdtemp(path.join(repositoryRoot, ".compatibility-ftml-raw-"))
  cleanupFixture(t, outputRoot)
  const sourceRevision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repositoryRoot,
    encoding: "utf8"
  }).stdout.trim()
  const outputPath = path.join(outputRoot, "inventory.json")

  const result = spawnSync(process.execPath, [
    cliPath,
    "--root",
    repositoryRoot,
    "--output",
    outputPath,
    "--source-revision",
    sourceRevision
  ], { encoding: "utf8" })

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const trackedInventory = JSON.parse(await fs.readFile(
    path.join(repositoryRoot, "docs/development/compatibility-surface-inventory.json"),
    "utf8"
  ))
  assert.equal(inventory.counts.total, trackedInventory.counts.total)
  assert.deepEqual(inventory.ftml_raw_surface_manifest.counts, {
    lexer_rules: 62,
    parser_functions: 3,
    canonical_blocks: 63,
    block_aliases: 47,
    typed_modules: 7,
    ast_variants: 48,
    delayed_forms: 9,
    generated_runtime_kinds: 2,
    renderer_modules: 30,
    wikidot_fixtures: 138,
    total: 409
  })
  assert.equal(inventory.ftml_raw_surface_manifest.catalog_crosswalk.length, 41)
  assert.deepEqual(inventory.ftml_raw_surface_manifest.source, inventory.provenance.ftml)
  assert.ok(
    inventory.ftml_raw_surface_manifest.records.some(
      ({ surface_id: surfaceId }) => surfaceId === "ftml.tokenizer:document"
    )
  )
  assert.ok(
    inventory.ftml_raw_surface_manifest.records.some(
      ({ surface_id: surfaceId }) => surfaceId === "ftml.fixture:test/link/canonical-inline"
    )
  )
  const byId = new Map(inventory.surfaces.map((record) => [record.surface_id, record]))
  const bibliography = byId.get("catalog-feature:syntax-bibliography")
  assert.equal(bibliography.source.status, "implemented")
  assert.ok(bibliography.source.references.length > 0)
  assert.ok(
    bibliography.source.references.every((reference) =>
      reference.startsWith(`Rokurolize/ftml@${inventory.provenance.ftml.commit}:`)
    )
  )
  const iftags = byId.get("catalog-feature:syntax-iftags")
  assert.equal(iftags.source.status, "implemented")
  assert.deepEqual(iftags.source.references, [
    "deepwell/src/services/render/iftags.rs#pub(super) fn resolve_outermost_wikidot_iftags("
  ])
  assert.deepEqual(iftags.existing_refs.tests, [
    "deepwell/tests/page.rs#page_render_basalt_rate_does_not_claim_active_iftags_through_eof"
  ])
})

test("CLI projects the audited registry issue owners and catalog implementation boundary", async (t) => {
  const outputRoot = await fs.mkdtemp(path.join(repositoryRoot, ".compatibility-audited-ownership-"))
  cleanupFixture(t, outputRoot)
  const outputPath = path.join(outputRoot, "inventory.json")
  const sourceRevision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repositoryRoot,
    encoding: "utf8"
  }).stdout.trim()
  const result = spawnSync(process.execPath, [
    cliPath,
    "--root",
    repositoryRoot,
    "--output",
    outputPath,
    "--source-revision",
    sourceRevision
  ], { encoding: "utf8" })
  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const byId = new Map(inventory.surfaces.map((record) => [record.surface_id, record]))
  const catalog = inventory.surfaces.filter((record) => record.kind === "catalog_feature")
  const registryRows = inventory.surfaces.filter((record) => [
    "deepwell_jsonrpc_method",
    "framerail_amc_action_shape",
    "framerail_amc_module_shape",
    "framerail_route",
    "framerail_server_action",
    "framerail_xmlrpc_method",
    "page_action",
    "wikidot_py_amc_module_shape",
    "wws_route"
  ].includes(record.kind))

  assert.deepEqual(
    catalog.filter((record) => record.implementation_owners.length === 0)
      .map(({ surface_id: surfaceId }) => surfaceId),
    [
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
    ]
  )
  assert.equal(
    catalog.filter((record) => !record.surface_id.startsWith("catalog-feature:api-"))
      .every((record) => record.existing_refs.issues.length === 1),
    true
  )
  assert.deepEqual(byId.get("catalog-feature:avatars").existing_refs.issues, [1392])
  assert.deepEqual(byId.get("catalog-feature:module-listpages").existing_refs.issues, [1383])
  assert.deepEqual(byId.get("catalog-feature:module-comments").existing_refs.issues, [1034])
  assert.deepEqual(byId.get("catalog-feature:module-members").existing_refs.issues, [1032])
  assert.deepEqual(byId.get("catalog-feature:module-managesite").existing_refs.issues, [1038])
  assert.deepEqual(byId.get("catalog-feature:module-sitechanges").existing_refs.issues, [1035])
  assert.deepEqual(byId.get("catalog-feature:account-lifecycle").existing_refs.issues, [1387])
  assert.deepEqual(byId.get("catalog-feature:api-pages-select").existing_refs.issues, [])
  assert.equal(registryRows.every((record) => record.existing_refs.issues.length > 0), true)
  assert.deepEqual(byId.get("deepwell-jsonrpc:admin_view").existing_refs.issues, [1368])
  assert.deepEqual(byId.get("wws-route:GET:/robots.txt").existing_refs.issues, [1369])
  assert.deepEqual(
    byId.get("wws-route:GET:/local--html/{page_slug}/{id}/{domain}").existing_refs.issues,
    [1370]
  )
  assert.deepEqual(byId.get("framerail-route:/").existing_refs.issues, [1372])
  assert.deepEqual(byId.get("framerail-route:/local--favicon/{filename}").existing_refs.issues, [756])
  assert.deepEqual(byId.get("framerail-amc-action:ForumAction:createPageDiscussionThread").existing_refs.issues, [839])
  assert.deepEqual(byId.get("framerail-amc-action:misc/NewPageHelperAction:createNewPage").existing_refs.issues, [1371])
  assert.deepEqual(byId.get("framerail-xmlrpc:system.listMethods").existing_refs.issues, [1375])
  assert.deepEqual(
    byId.get("wikidot-py-amc-module:changes/SiteChangesListModule:parameters=options,page,perpage").existing_refs.issues,
    [1376]
  )
})

test("CLI projects the current Deepwell contract evidence without promoting source-only test gaps", async (t) => {
  const outputRoot = await fs.mkdtemp(path.join(repositoryRoot, ".compatibility-deepwell-contract-"))
  cleanupFixture(t, outputRoot)
  const outputPath = path.join(outputRoot, "inventory.json")
  const sourceRevision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repositoryRoot,
    encoding: "utf8"
  }).stdout.trim()
  const result = spawnSync(process.execPath, [
    cliPath,
    "--root",
    repositoryRoot,
    "--output",
    outputPath,
    "--source-revision",
    sourceRevision
  ], { encoding: "utf8" })
  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const rows = inventory.surfaces.filter((record) => record.kind === "deepwell_jsonrpc_method")
  const manifest = JSON.parse(
    await fs.readFile(path.join(repositoryRoot, "docs/development/deepwell-jsonrpc-contract-manifest.json"), "utf8")
  )
  assert.equal(rows.length, manifest.method_count)
  assert.equal(rows.every(({ evidence }) => evidence.status === "available"), true)
  assert.equal(rows.every(({ evidence }) =>
    evidence.references.includes("docs/development/deepwell-jsonrpc-contract-manifest.json")
  ), true)
  assert.equal(rows.filter(({ existing_refs: existingRefs }) => existingRefs.tests.length > 0).length, rows.length)
  assert.equal(rows.filter(({ existing_refs: existingRefs }) => existingRefs.tests.length === 0).length, 0)
  assert.deepEqual(
    rows.find(({ surface_id: surfaceId }) => surfaceId === "deepwell-jsonrpc:wikidot_categories_page_list_module")
      .existing_refs.issues,
    [1368]
  )
  assert.deepEqual(
    rows.find(({ surface_id: surfaceId }) => surfaceId === "deepwell-jsonrpc:authorization_token_issue")
      .existing_refs.tests,
    ["deepwell/tests/auth.rs#authorization_token_issue_requires_admin_request_context"]
  )
  assert.deepEqual(
    rows.find(({ surface_id: surfaceId }) => surfaceId === "deepwell-jsonrpc:admin_view")
      .existing_refs.tests,
    [
      "install/local/wikidot-verification/tests/deepwell-jsonrpc-contract-manifest.test.mjs#Deepwell JSON-RPC manifest exactly covers the current registered contract"
    ]
  )
  assert.deepEqual(
    rows.find(({ surface_id: surfaceId }) => surfaceId === "deepwell-jsonrpc:category_get")
      .existing_refs.tests,
    ["deepwell/tests/page/page_move.rs#page_move_render_failure_rolls_back_destination_identity"]
  )
  assert.deepEqual(
    rows.find(({ surface_id: surfaceId }) => surfaceId === "deepwell-jsonrpc:membership_email_invitation_accept")
      .existing_refs.tests,
    ["deepwell/tests/page/membership.rs#membership_email_invitation_matches_hash_one_use_and_cancel_contract"]
  )
})

test("CLI projects WWS route-contract evidence while keeping hash-domain evidence partial", async (t) => {
  const outputRoot = await fs.mkdtemp(path.join(repositoryRoot, ".compatibility-wws-contract-"))
  cleanupFixture(t, outputRoot)
  const outputPath = path.join(outputRoot, "inventory.json")
  const sourceRevision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repositoryRoot,
    encoding: "utf8"
  }).stdout.trim()
  const result = spawnSync(process.execPath, [
    cliPath,
    "--root",
    repositoryRoot,
    "--output",
    outputPath,
    "--source-revision",
    sourceRevision
  ], { encoding: "utf8" })
  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const rows = inventory.surfaces.filter((record) => record.kind === "wws_route")
  assert.equal(rows.length, 50)
  assert.equal(rows.filter(({ evidence }) => evidence.status === "available").length, 47)
  assert.equal(rows.filter(({ evidence }) => evidence.status === "partial").length, 3)
  assert.equal(rows.filter(({ existing_refs: existingRefs }) => existingRefs.tests.length > 0).length, 50)
  assert.equal(rows.filter(({ existing_refs: existingRefs }) => existingRefs.tests.length === 0).length, 0)
  assert.equal(rows.every(({ evidence }) =>
    evidence.references.includes("docs/development/wws-route-registration-denominator.json")
  ), true)
  for (const surfaceId of [
    "wws-route:FALLBACK:/local--html/{page_slug}/{id}/{domain}",
    "wws-route:GET:/local--html/{page_slug}/{id}/{domain}",
    "wws-route:HEAD:/local--html/{page_slug}/{id}/{domain}"
  ]) {
    const row = rows.find(({ surface_id: candidate }) => candidate === surfaceId)
    assert.equal(row.evidence.status, "partial", surfaceId)
    assert.deepEqual(row.existing_refs.tests, [
      "wws/src/handler/text_block.rs#html_terminal_hash_verifies_fetched_bytes"
    ])
  }
})

test("CLI projects current Framerail route-action tests without inventing browser evidence", async (t) => {
  const outputRoot = await fs.mkdtemp(path.join(repositoryRoot, ".compatibility-framerail-route-action-"))
  cleanupFixture(t, outputRoot)
  const outputPath = path.join(outputRoot, "inventory.json")
  const sourceRevision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repositoryRoot,
    encoding: "utf8"
  }).stdout.trim()
  const result = spawnSync(process.execPath, [
    cliPath,
    "--root",
    repositoryRoot,
    "--output",
    outputPath,
    "--source-revision",
    sourceRevision
  ], { encoding: "utf8" })
  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const rows = inventory.surfaces.filter(({ kind }) =>
    kind === "framerail_route" || kind === "framerail_server_action"
  )
  const evidence = JSON.parse(
    await fs.readFile(path.join(repositoryRoot, "docs/development/framerail-route-action-evidence.json"), "utf8")
  )
  assert.equal(rows.length, evidence.records.length)
  assert.equal(rows.filter(({ existing_refs: existingRefs }) => existingRefs.tests.length > 0).length, rows.length)
  assert.equal(rows.filter(({ existing_refs: existingRefs }) => existingRefs.tests.length === 0).length, 0)
  assert.equal(rows.every(({ evidence }) => evidence.status === "missing"), true)
  assert.deepEqual(
    rows.find(({ surface_id: surfaceId }) => surfaceId === "framerail-route:/").existing_refs.tests,
    ["framerail/tests/page-workflows.spec.ts#article routes carry load and mutation context through Deepwell"]
  )
  assert.deepEqual(
    rows.find(({ surface_id: surfaceId }) => surfaceId === "framerail-server-action:/?/layout")
      .existing_refs.tests,
    [
      "framerail/tests/release-route-action-boundaries.test.js#previously unattributed public route actions stay wired to their owning handlers"
    ]
  )
  assert.deepEqual(
    rows.find(({ surface_id: surfaceId }) => surfaceId === "framerail-server-action:/?/membershipEmailInvitation")
      .existing_refs.tests,
    [
      "framerail/tests/page-membership-action-boundary.test.js#email invitation action derives the opaque hash from the trusted route, not the request body"
    ]
  )
  assert.deepEqual(
    rows.find(({ surface_id: surfaceId }) => surfaceId === "framerail-server-action:/_admin?/membershipReview")
      .existing_refs.tests,
    [
      "framerail/tests/site-settings-public-boundary.test.js#renders membership policy without exposing a configured password and binds review actions"
    ]
  )
})

test("CLI projects only directly exercised Framerail AMC server tests", async (t) => {
  const outputRoot = await fs.mkdtemp(path.join(repositoryRoot, ".compatibility-framerail-amc-tests-"))
  cleanupFixture(t, outputRoot)
  const outputPath = path.join(outputRoot, "inventory.json")
  const sourceRevision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repositoryRoot,
    encoding: "utf8"
  }).stdout.trim()
  const result = spawnSync(process.execPath, [
    cliPath,
    "--root",
    repositoryRoot,
    "--output",
    outputPath,
    "--source-revision",
    sourceRevision
  ], { encoding: "utf8" })
  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const rows = inventory.surfaces.filter(({ kind }) =>
    kind === "framerail_amc_action_shape" || kind === "framerail_amc_module_shape"
  )
  assert.equal(rows.length, 29)
  assert.equal(rows.filter(({ existing_refs: existingRefs }) => existingRefs.tests.length > 0).length, 29)
  const gaps = rows
    .filter(({ existing_refs: existingRefs }) => existingRefs.tests.length === 0)
    .map(({ surface_id: surfaceId }) => surfaceId)
  assert.deepEqual(gaps, [])
  assert.deepEqual(
    rows
      .filter(({ evidence }) => evidence.status === "available")
      .map(({ surface_id: surfaceId, evidence }) => ({ surfaceId, references: evidence.references })),
    [{
      surfaceId:
        "framerail-amc-module:changes/SiteChangesListModule:parameters=categoryId,options,page,pageId,perpage",
      references: [
        "install/local/wikidot-verification/artifacts/open43-readonly-live-20260810.json#E_OPEN43_SITECHANGES_AMC_20260810"
      ]
    }]
  )
  assert.equal(rows.filter(({ evidence }) => evidence.status === "missing").length, 28)
})

test("CLI rejects semantic registry identity, crosswalk, owner, and edge drift", async (t) => {
  const cases = [
    {
      name: "same-count FTML identity substitution",
      mutate: (semantics) => {
        semantics.ftml.raw_surface_identities[0].surface_id = "ftml.ast:Ancho"
      },
      error: /pinned FTML raw surface identities drift/u
    },
    {
      name: "missing crosswalk row",
      nominate: true,
      mutate: () => {},
      error: /FTML crosswalk does not exactly match Catalog nominations/u
    },
    {
      name: "duplicate crosswalk row",
      nominate: true,
      mutate: (semantics) => {
        const row = {
          feature_id: "feature-one",
          parsed_by: ["ftml.ast:Anchor"],
          rendered_by: [],
          tested_by: [],
          ftml_surfaces: ["ftml.ast:Anchor"],
          runtime_owner: null
        }
        semantics.ftml.catalog_crosswalk = [row, row]
      },
      error: /FTML crosswalk rows must be sorted and unique/u
    },
    {
      name: "missing owner key",
      mutate: (semantics) => {
        semantics.specification_owner_keys = semantics.specification_owner_keys.filter(
          (owner) => owner !== "catalog.feature:feature-one"
        )
      },
      error: /missing or unused specification owner keys/u
    },
    {
      name: "duplicate owner key",
      mutate: (semantics) => {
        semantics.implementation_owner_keys.push(semantics.implementation_owner_keys[0])
      },
      error: /implementation owners must be a sorted unique string array/u
    },
    {
      name: "unused legacy owner key",
      mutate: (semantics) => {
        semantics.implementation_owners_by_legacy_owner.unused = []
      },
      error: /missing or unused legacy owner keys/u
    },
    {
      name: "missing edge type",
      mutate: (semantics) => {
        semantics.relationship_edge_types = semantics.relationship_edge_types.filter(
          (type) => type !== "alias"
        )
      },
      error: /missing, duplicate, or unknown relationship edge types/u
    },
    {
      name: "duplicate edge type",
      mutate: (semantics) => {
        semantics.relationship_edge_types.push("alias")
      },
      error: /edge types must be a sorted unique string array/u
    },
    {
      name: "unknown edge type",
      mutate: (semantics) => {
        semantics.relationship_edge_types.push("invented_by")
      },
      error: /edge types must be a sorted unique string array/u
    }
  ]

  for (const fixtureCase of cases) {
    await t.test(fixtureCase.name, async (t) => {
      const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-semantics-"))
      cleanupFixture(t, root)
      await writeRepositoryFixture(root)
      const semanticsPath = path.join(root, "docs/development/compatibility-surface-semantics.json")
      const semantics = JSON.parse(await fs.readFile(semanticsPath, "utf8"))
      fixtureCase.mutate(semantics)
      await fs.writeFile(semanticsPath, `${JSON.stringify(semantics, null, 2)}\n`)
      if (fixtureCase.nominate) {
        const catalogPath = path.join(root, "docs/wikidot-specifications/catalog.json")
        const catalog = JSON.parse(await fs.readFile(catalogPath, "utf8"))
        catalog.features[0].suggested_tdd_seams = ["FTML public parse/render"]
        await fs.writeFile(catalogPath, `${JSON.stringify(catalog, null, 2)}\n`)
        await refreshCatalogHash(root)
      }

      const result = runCli(root, path.join(root, "inventory.json"))

      assert.equal(result.status, 1, result.stderr)
      assert.match(result.stderr, fixtureCase.error)
    })
  }
})

test("CLI emits closed owner keys and typed edges without double-counting FTML records", async (t) => {
  const outputRoot = await fs.mkdtemp(path.join(repositoryRoot, ".compatibility-owner-edges-"))
  cleanupFixture(t, outputRoot)
  const outputPath = path.join(outputRoot, "inventory.json")
  const sourceRevision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repositoryRoot,
    encoding: "utf8"
  }).stdout.trim()
  const result = spawnSync(process.execPath, [
    cliPath,
    "--root",
    repositoryRoot,
    "--output",
    outputPath,
    "--source-revision",
    sourceRevision
  ], { encoding: "utf8" })

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  assert.deepEqual(
    inventory.surfaces.find(({ surface_id: surfaceId }) => surfaceId === "catalog-feature:page-tags").implementation_owners,
    ["wikijump.framerail"]
  )
  const siteStructureFeatures = inventory.surfaces.filter(({ kind, public_reference }) =>
    kind === "catalog_feature" && public_reference.some((reference) =>
      reference.startsWith("docs/wikidot-specifications/specifications/site-structure/")
    )
  )
  assert.equal(siteStructureFeatures.length, 13)
  assert.deepEqual(
    siteStructureFeatures.filter(({ implementation_owners: owners }) => owners.length > 0)
      .map(({ surface_id: surfaceId }) => surfaceId),
    [
      "catalog-feature:content-pages",
      "catalog-feature:forum-categories",
      "catalog-feature:forum-category-groups",
      "catalog-feature:forum-posts",
      "catalog-feature:forum-threads",
      "catalog-feature:forums-overview",
      "catalog-feature:page-categories",
      "catalog-feature:page-forum-integration",
      "catalog-feature:page-inclusions",
      "catalog-feature:page-links",
      "catalog-feature:page-parent-relations",
      "catalog-feature:page-tags",
      "catalog-feature:site-identity"
    ]
  )
  assert.equal(
    siteStructureFeatures.filter(({ implementation_owners: owners }) => owners.length === 0).length,
    0
  )
  assert.deepEqual(
    inventory.surfaces.find(({ surface_id: surfaceId }) => surfaceId === "catalog-feature:page-parent-relations").implementation_owners,
    ["wikijump.deepwell"]
  )
  assert.deepEqual(inventory.relationship_edge_types, [
    "alias",
    "equivalence",
    "implemented_by",
    "parsed_by",
    "rendered_by",
    "tested_by"
  ])
  const specificationOwners = new Set(inventory.owner_keys.specification)
  const implementationOwners = new Set(inventory.owner_keys.implementation)
  assert.ok(inventory.surfaces.every((record) => !("public_owner" in record)))
  assert.ok(inventory.surfaces.every((record) => specificationOwners.has(record.specification_owner)))
  assert.ok(
    inventory.surfaces.every((record) =>
      record.implementation_owners.every((owner) => implementationOwners.has(owner))
    )
  )
  const publicIds = new Set(inventory.surfaces.map(({ surface_id: surfaceId }) => surfaceId))
  const rawIds = new Set(
    inventory.ftml_raw_surface_manifest.records.map(({ surface_id: surfaceId }) => surfaceId)
  )
  assert.ok([...rawIds].every((surfaceId) => !publicIds.has(surfaceId)))
  const trackedInventory = JSON.parse(await fs.readFile(
    path.join(repositoryRoot, "docs/development/compatibility-surface-inventory.json"),
    "utf8"
  ))
  assert.equal(inventory.counts.total, trackedInventory.counts.total)
  assert.equal(
    inventory.relationship_edges.filter(({ type }) => type === "alias").length,
    47
  )
  const edgeKeys = inventory.relationship_edges.map(
    ({ source, type, target }) => `${source}\u0000${type}\u0000${target}`
  )
  assert.equal(new Set(edgeKeys).size, edgeKeys.length)
  assert.ok(
    inventory.relationship_edges.every(({ type }) =>
      inventory.relationship_edge_types.includes(type)
    )
  )
})

test("CLI rejects a canonical implementation ledger mirror mismatch before discovery", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-ledger-mismatch-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  const mirrorPath = path.join(root, "docs/wikidot-specifications/implementation-ledger.json")
  const mirror = JSON.parse(await fs.readFile(mirrorPath, "utf8"))
  mirror.features["feature-one"].status = "implemented"
  await fs.writeFile(mirrorPath, `${JSON.stringify(mirror, null, 2)}\n`)

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 1)
  assert.match(result.stderr, /scripts\/data\/wikidot-implementation-ledger\.json and docs\/wikidot-specifications\/implementation-ledger\.json must be byte-identical/u)
})

test("CLI keeps cited data-form owners and fills only audited ownerless rows", async (t) => {
  const outputRoot = await fs.mkdtemp(path.join(repositoryRoot, ".compatibility-data-form-owners-"))
  cleanupFixture(t, outputRoot)
  const outputPath = path.join(outputRoot, "inventory.json")
  const sourceRevision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repositoryRoot,
    encoding: "utf8"
  }).stdout.trim()
  const result = spawnSync(process.execPath, [
    cliPath,
    "--root",
    repositoryRoot,
    "--output",
    outputPath,
    "--source-revision",
    sourceRevision
  ], { encoding: "utf8" })

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const rows = inventory.surfaces.filter(({ public_reference }) =>
    public_reference[0]?.startsWith("docs/wikidot-specifications/specifications/data-forms/")
  )
  const expectedBound = new Map([
    ["catalog-feature:data-forms-checkbox-field", ["wikijump.deepwell", "wikijump.framerail"]],
    ["catalog-feature:data-forms-creating-new-page", ["wikijump.deepwell", "wikijump.framerail"]],
    ["catalog-feature:data-forms-dataforms-and-listpages", ["wikijump.deepwell"]],
    ["catalog-feature:data-forms-displaying", ["wikijump.deepwell"]],
    ["catalog-feature:data-forms-field-properties", ["wikijump.deepwell", "wikijump.framerail"]],
    ["catalog-feature:data-forms-file-field", ["wikijump.deepwell", "wikijump.framerail"]],
    ["catalog-feature:data-forms-hidden-field", ["wikijump.deepwell", "wikijump.framerail"]],
    ["catalog-feature:data-forms-hints", ["wikijump.deepwell", "wikijump.framerail"]],
    ["catalog-feature:data-forms-images", ["wikijump.deepwell", "wikijump.framerail"]],
    ["catalog-feature:data-forms-select-field", ["wikijump.deepwell", "wikijump.framerail"]],
    ["catalog-feature:data-forms-selecting-and-sorting", ["wikijump.deepwell"]],
    ["catalog-feature:data-forms-tags", ["wikijump.framerail"]],
    ["catalog-feature:data-forms-text-field", ["wikijump.deepwell", "wikijump.framerail"]],
    ["catalog-feature:data-forms-wiki-field", ["wikijump.deepwell", "wikijump.framerail"]]
  ])
  const expectedAuditedFallback = [
    "catalog-feature:data-forms-css-styling",
    "catalog-feature:data-forms-date-field",
    "catalog-feature:data-forms-deleting-form",
    "catalog-feature:data-forms-howto",
    "catalog-feature:data-forms-links",
    "catalog-feature:data-forms-output-style",
    "catalog-feature:data-forms-overview",
    "catalog-feature:data-forms-pagepath",
    "catalog-feature:data-forms-pagepath-field",
    "catalog-feature:data-forms-password-field",
    "catalog-feature:data-forms-static-field",
    "catalog-feature:data-forms-url-field",
    "catalog-feature:data-forms-youtube"
  ]
  assert.equal(rows.length, 27)
  assert.deepEqual(rows.map(({ surface_id }) => surface_id), [...expectedBound.keys(), ...expectedAuditedFallback].sort())
  for (const row of rows) {
    const inferredOwners = [...new Set((row.source?.references ?? []).flatMap((reference) => {
      if (reference.startsWith("deepwell/")) return ["wikijump.deepwell"]
      if (reference.startsWith("framerail/")) return ["wikijump.framerail"]
      if (reference.startsWith("wws/")) return ["wikijump.wws"]
      return []
    }))].sort()
    assert.deepEqual(
      row.implementation_owners,
      expectedBound.get(row.surface_id) ?? inferredOwners,
      row.surface_id
    )
    assert.ok(!row.implementation_owners.includes("wikijump"), row.surface_id)
  }
  assert.equal(rows.filter(({ implementation_owners }) => implementation_owners.length > 0).length, 27)
  assert.equal(rows.filter(({ implementation_owners }) => implementation_owners.length === 0).length, 0)
  const canonicalLedger = JSON.parse(
    await fs.readFile(path.join(repositoryRoot, "scripts/data/wikidot-implementation-ledger.json"), "utf8")
  )
  assert.equal(Object.keys(canonicalLedger.implementation_owner_records).length, 114)
  assert.equal(
    Object.values(canonicalLedger.implementation_owner_records).filter(({ issue_scope }) =>
      issue_scope.status === "resolved" &&
      JSON.stringify(issue_scope.references) === JSON.stringify([1504])
    ).length,
    2
  )
  assert.equal(
    Object.values(canonicalLedger.implementation_owner_records).filter(({ issue_scope }) =>
      issue_scope.status === "unresolved" && issue_scope.references.length === 0
    ).length,
    111
  )
})

test("CLI keeps canonical module owners and fills audited ownerless modules", async (t) => {
  const outputRoot = await fs.mkdtemp(path.join(repositoryRoot, ".compatibility-module-owners-"))
  cleanupFixture(t, outputRoot)
  const outputPath = path.join(outputRoot, "inventory.json")
  const sourceRevision = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repositoryRoot,
    encoding: "utf8"
  }).stdout.trim()
  const result = spawnSync(process.execPath, [
    cliPath,
    "--root",
    repositoryRoot,
    "--output",
    outputPath,
    "--source-revision",
    sourceRevision
  ], { encoding: "utf8" })

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const moduleRows = inventory.surfaces.filter(({ public_reference }) =>
    public_reference[0]?.startsWith("docs/wikidot-specifications/specifications/module/")
  )
  const canonicalLedger = JSON.parse(
    await fs.readFile(path.join(repositoryRoot, "scripts/data/wikidot-implementation-ledger.json"), "utf8")
  )
  assert.equal(moduleRows.length, 74)
  assert.deepEqual(
    moduleRows.filter(({ implementation_owners }) =>
      implementation_owners.length === 1 && implementation_owners[0] === "wikijump"
    )
      .map(({ surface_id: surfaceId }) => surfaceId),
    []
  )
  for (const row of moduleRows) {
    const featureId = row.surface_id.slice("catalog-feature:".length)
    const recordedOwners = canonicalLedger.implementation_owner_records[featureId].owners
      .map(({ owner }) => owner)
    const inferredOwners = [...new Set((row.source?.references ?? []).flatMap((reference) => {
      if (reference.startsWith("deepwell/")) return ["wikijump.deepwell"]
      if (reference.startsWith("framerail/")) return ["wikijump.framerail"]
      if (reference.startsWith("wws/")) return ["wikijump.wws"]
      return []
    }))].sort()
    const expectedOwners = recordedOwners.length > 0 ? recordedOwners : inferredOwners
    assert.deepEqual(row.implementation_owners, expectedOwners, row.surface_id)
    assert.ok(!row.implementation_owners.includes("wikijump"), row.surface_id)
  }
  assert.deepEqual(
    moduleRows.find(({ surface_id: surfaceId }) => surfaceId === "catalog-feature:module-comments").implementation_owners,
    ["wikijump.deepwell", "wikijump.framerail"]
  )
  assert.deepEqual(
    moduleRows.find(({ surface_id: surfaceId }) => surfaceId === "catalog-feature:module-listpages").implementation_owners,
    ["wikijump.deepwell", "wikijump.framerail"]
  )
})

test("CLI does not infer a data-form owner from a cited source path", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-data-form-owner-negative-"))
  cleanupFixture(t, root)
  await writeCatalogOwnerFixture(root, {
    issue_scope: { status: "unresolved", references: [] },
    owners: []
  })
  const outputPath = path.join(root, "inventory.json")
  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const row = inventory.surfaces.find(({ surface_id }) => surface_id === "catalog-feature:data-forms-fixture")
  assert.deepEqual(row.implementation_owners, [])
})

test("CLI does not infer a site-structure owner from a cited source path", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-site-structure-owner-negative-"))
  cleanupFixture(t, root)
  await writeCatalogOwnerFixture(root, {
    issue_scope: { status: "unresolved", references: [] },
    owners: []
  }, "specifications/site-structure/fixture.md")
  const outputPath = path.join(root, "inventory.json")
  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const row = inventory.surfaces.find(({ surface_id }) => surface_id === "catalog-feature:data-forms-fixture")
  assert.deepEqual(row.implementation_owners, [])
})

test("CLI does not infer a module owner from a cited source path", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-module-owner-negative-"))
  cleanupFixture(t, root)
  await writeCatalogOwnerFixture(root, {
    issue_scope: { status: "unresolved", references: [] },
    owners: []
  }, "specifications/module/fixture.md", "module-fixture")
  const outputPath = path.join(root, "inventory.json")
  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const row = inventory.surfaces.find(({ surface_id: surfaceId }) => surfaceId === "catalog-feature:module-fixture")
  assert.deepEqual(row.implementation_owners, [])
})

test("tracked compatibility inventory exactly matches one generator run", async (t) => {
  const outputRoot = await fs.mkdtemp(path.join(repositoryRoot, ".compatibility-inventory-byte-compare-"))
  cleanupFixture(t, outputRoot)
  const trackedPath = path.join(repositoryRoot, "docs/development/compatibility-surface-inventory.json")
  const trackedBytes = await fs.readFile(trackedPath)
  const outputPath = path.join(outputRoot, "inventory.json")
  const result = spawnSync(process.execPath, [
    cliPath,
    "--root",
    repositoryRoot,
    "--output",
    outputPath
  ], { encoding: "utf8" })

  assert.equal(result.status, 0, result.stderr)
  assert.deepEqual(await fs.readFile(outputPath), trackedBytes)
})

test("CLI rejects an omitted or duplicate missing-page control", async (t) => {
  for (const mutation of ["omit", "duplicate"]) {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), `compatibility-missing-page-${mutation}-`))
    cleanupFixture(t, root)
    await writeRepositoryFixture(root)
    const registryPath = path.join(root, "docs/development/wikidot-page-action-surfaces.json")
    const registry = JSON.parse(await fs.readFile(registryPath, "utf8"))
    registry.missing_page_controls = mutation === "omit"
      ? registry.missing_page_controls.slice(0, 1)
      : [registry.missing_page_controls[0], registry.missing_page_controls[0]]
    await fs.writeFile(registryPath, `${JSON.stringify(registry, null, 2)}\n`)

    const result = runCli(root, path.join(root, "inventory.json"))

    assert.equal(result.status, 1, mutation)
    assert.match(
      result.stderr,
      /must declare exactly one create and one restore control/u,
      mutation
    )
  }
})

test("CLI binds missing-page source identities to the current source bytes", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-missing-page-bytes-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  const edited = `${pageActionsFixtureSource}\n// edited after the registry was written\n`
  await writeText(root, "framerail/src/lib/server/load/page/page-actions.ts", edited)

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(path.join(root, "inventory.json"), "utf8"))
  const control = inventory.surfaces.find(({ surface_id: id }) => id === "missing-page-control:create")
  assert.deepEqual(
    control.source_identities.find(({ path: sourcePath }) => sourcePath.endsWith("page-actions.ts")),
    { path: "framerail/src/lib/server/load/page/page-actions.ts", sha256: sha256(edited) }
  )
})

test("CLI rejects a pinned missing-page source hash", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-missing-page-pin-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  const registryPath = path.join(root, "docs/development/wikidot-page-action-surfaces.json")
  const registry = JSON.parse(await fs.readFile(registryPath, "utf8"))
  registry.missing_page_controls[0].source_identities[0].sha256 = "0".repeat(64)
  await fs.writeFile(registryPath, `${JSON.stringify(registry, null, 2)}\n`)

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 1)
  assert.match(result.stderr, /create has an invalid source identity/u)
  await assert.rejects(fs.access(path.join(root, "inventory.json")))
})

test("CLI rejects a missing-page source identity outside the repository", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-missing-page-path-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  const registryPath = path.join(root, "docs/development/wikidot-page-action-surfaces.json")
  const registry = JSON.parse(await fs.readFile(registryPath, "utf8"))
  registry.missing_page_controls[0].source_identities[0].path = "../outside-source.svelte"
  await fs.writeFile(registryPath, `${JSON.stringify(registry, null, 2)}\n`)

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 1)
  assert.match(result.stderr, /create has an invalid source identity/u)
  await assert.rejects(fs.access(path.join(root, "inventory.json")))
})

test("CLI confines inventory inputs and output to the repository", async (t) => {
  const outputRoot = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-inventory-output-path-"))
  cleanupFixture(t, outputRoot)
  await writeRepositoryFixture(outputRoot)
  const outsideOutput = path.join(path.dirname(outputRoot), "compatibility-inventory-outside.json")
  await fs.rm(outsideOutput, { force: true })
  const outside = runCli(outputRoot, outsideOutput)
  assert.equal(outside.status, 1)
  assert.match(outside.stderr, /public reference is outside the repository/u)
  await assert.rejects(fs.access(outsideOutput))

  const symlinkRoot = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-inventory-output-link-"))
  cleanupFixture(t, symlinkRoot)
  await writeRepositoryFixture(symlinkRoot)
  const target = path.join(path.dirname(symlinkRoot), "compatibility-inventory-target.json")
  await fs.writeFile(target, "untouched\n")
  const symlink = path.join(symlinkRoot, "inventory.json")
  await fs.symlink(target, symlink)
  const linked = runCli(symlinkRoot, symlink)
  assert.equal(linked.status, 1)
  assert.match(linked.stderr, /output must not be a symbolic link/u)
  assert.equal(await fs.readFile(target, "utf8"), "untouched\n")

  const auditRoot = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-inventory-audit-path-"))
  cleanupFixture(t, auditRoot)
  await writeRepositoryFixture(auditRoot)
  const routingPath = path.join(auditRoot, "docs/development/open43-blocked-evidence-routing.json")
  const reconciliationPath = path.join(auditRoot, "docs/development/open43-closure-audit-ownership-reconciliation.json")
  const routing = JSON.parse(await fs.readFile(routingPath, "utf8"))
  const reconciliation = JSON.parse(await fs.readFile(reconciliationPath, "utf8"))
  const escaped = "docs/development/../../../outside-audit.json"
  routing.source_audits[0] = escaped
  reconciliation.closure_audits[0].path = escaped
  await fs.writeFile(routingPath, `${JSON.stringify(routing, null, 2)}\n`)
  await fs.writeFile(reconciliationPath, `${JSON.stringify(reconciliation, null, 2)}\n`)
  const audit = runCli(auditRoot, path.join(auditRoot, "inventory.json"))
  assert.equal(audit.status, 1)
  assert.match(audit.stderr, /public reference is outside the repository/u)
})

test("CLI rejects malformed or contradictory available browser proof", async (t) => {
  const invalidProofs = [
    { status: "available", references: [null] },
    { status: "available", references: [""] },
    { status: "available", references: ["./evidence/create.json"] },
    { status: "available", references: [" evidence/create.json"] },
    { status: "available", references: ["evidence/create.json", "evidence/create.json"] },
    { status: "available", references: ["evidence/create.json"], issue: 1372 },
    { status: "missing", issue: 1372, references: ["evidence/create.json"] },
    { status: "missing", issue: 1372, extra: true }
  ]
  for (const [index, proof] of invalidProofs.entries()) {
    const root = await fs.mkdtemp(
      path.join(os.tmpdir(), `compatibility-missing-page-proof-${index}-`)
    )
    cleanupFixture(t, root)
    await writeRepositoryFixture(root)
    const registryPath = path.join(root, "docs/development/wikidot-page-action-surfaces.json")
    const registry = JSON.parse(await fs.readFile(registryPath, "utf8"))
    registry.missing_page_controls[0].browser_interval_proof = proof
    await fs.writeFile(registryPath, `${JSON.stringify(registry, null, 2)}\n`)

    const result = runCli(root, path.join(root, "inventory.json"))

    assert.equal(result.status, 1, JSON.stringify(proof))
    assert.match(result.stderr, /create has invalid browser_interval_proof/u)
  }
})

test("CLI accepts canonical available browser proof without adding an issue", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-missing-page-proof-ok-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  const registryPath = path.join(root, "docs/development/wikidot-page-action-surfaces.json")
  const registry = JSON.parse(await fs.readFile(registryPath, "utf8"))
  registry.missing_page_controls[0].browser_interval_proof = {
    status: "available",
    references: ["evidence/create-browser.json#settled"]
  }
  await fs.writeFile(registryPath, `${JSON.stringify(registry, null, 2)}\n`)
  const outputPath = path.join(root, "inventory.json")

  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const create = inventory.surfaces.find(
    ({ surface_id }) => surface_id === "missing-page-control:create"
  )
  assert.deepEqual(create.browser_interval_proof, {
    status: "available",
    references: ["evidence/create-browser.json#settled"]
  })
})

test("CLI rejects an operation anchor absent from the declared pageActions object", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-missing-page-action-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  const actionSource = pageActionsFixtureSource.replace("edit: editAction,\n", "")
  await replacePageActionsFixture(root, actionSource)

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 1)
  assert.match(result.stderr, /create edit is not declared by .*#pageActions/u)
})

test("CLI ignores pageActions declarations shadowed by TypeScript comments and strings", async (t) => {
  const realDeclaration = pageActionsFixtureSource.replace("edit: editAction,\n", "")
  const shadows = {
    comment: "// export const pageActions = { edit: fake, deletedGet: fake, restore: fake }",
    string: 'const shadow = "export const pageActions = { edit: fake, deletedGet: fake, restore: fake }"'
  }
  for (const [kind, shadow] of Object.entries(shadows)) {
    await t.test(kind, async (t) => {
      const root = await fs.mkdtemp(
        path.join(os.tmpdir(), `compatibility-missing-page-${kind}-`)
      )
      cleanupFixture(t, root)
      await writeRepositoryFixture(root)
      await replacePageActionsFixture(root, `${shadow}\n${realDeclaration}`)

      const result = runCli(root, path.join(root, "inventory.json"))

      assert.equal(result.status, 1)
      assert.match(result.stderr, /create edit is not declared by .*#pageActions/u)
    })
  }
})

test("CLI rejects duplicate pageActions declarations", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-page-actions-duplicate-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  await replacePageActionsFixture(root, `${pageActionsFixtureSource.trimEnd()}; export const pageActions = {}\n`)

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 1)
  assert.match(result.stderr, /duplicate exported pageActions declarations/u)
})

test("CLI fails closed on declaration-shaped RegExp text", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-page-actions-regexp-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  await replacePageActionsFixture(root, "const shadow = /export const pageActions = {}/\n")

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 1)
  assert.match(result.stderr, /declaration-shaped text outside a supported top-level declaration/u)
})

test("CLI discovers GET, implicit HEAD, and FALLBACK for the production composite route", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-wws-composite-"))
  await writeRepositoryFixture(root)
  await writeText(
    root,
    "wws/src/route.rs",
    `pub fn build_router() {
  Router::new().route(
    "/{page_slug}/code",
    on(MethodFilter::GET, handle_default_code_redirect)
      .fallback(redirect_to_main),
  )
}
`
  )
  const outputPath = path.join(root, "inventory.json")

  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  assert.deepEqual(
    inventory.surfaces
      .filter(({ kind }) => kind === "wws_route")
      .map(({ surface_id }) => surface_id),
    [
      "wws-route:FALLBACK:/{page_slug}/code",
      "wws-route:GET:/{page_slug}/code",
      "wws-route:HEAD:/{page_slug}/code"
    ]
  )
  const firstOutput = await fs.readFile(outputPath, "utf8")
  const secondResult = runCli(root, outputPath)
  assert.equal(secondResult.status, 0, secondResult.stderr)
  assert.equal(await fs.readFile(outputPath, "utf8"), firstOutput)
})

test("CLI aggregates same-path GET and any handlers into effective dispatch slots", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-wws-aggregate-"))
  await writeRepositoryFixture(root)
  await writeText(
    root,
    "wws/src/route.rs",
    `pub fn build_router() {
  Router::new()
    .route("/-/file/{page_slug}/{filename}", get(handle_file_fetch))
    .route("/-/file/{page_slug}/{filename}", any(handle_invalid_method))
}
`
  )
  const outputPath = path.join(root, "inventory.json")

  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  assert.deepEqual(
    inventory.surfaces
      .filter(({ kind }) => kind === "wws_route")
      .map(({ surface_id }) => surface_id),
    [
      "wws-route:FALLBACK:/-/file/{page_slug}/{filename}",
      "wws-route:GET:/-/file/{page_slug}/{filename}",
      "wws-route:HEAD:/-/file/{page_slug}/{filename}"
    ]
  )
})

test("CLI preserves standalone any as one ANY dispatch surface", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-wws-any-"))
  await writeRepositoryFixture(root)
  await writeText(
    root,
    "wws/src/route.rs",
    `pub fn build_router() {
  Router::new().route("/-/health-check", any(handle_health_check))
}
`
  )
  const outputPath = path.join(root, "inventory.json")

  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  assert.deepEqual(
    inventory.surfaces
      .filter(({ kind }) => kind === "wws_route")
      .map(({ surface_id }) => surface_id),
    ["wws-route:ANY:/-/health-check"]
  )
})

test("CLI accepts a closed MethodFilter or-chain with a route-local fallback", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-wws-filter-"))
  await writeRepositoryFixture(root)
  await writeText(
    root,
    "wws/src/route.rs",
    `pub fn build_router() {
  Router::new().route(
    "/filtered",
    on(MethodFilter::GET.or(MethodFilter::HEAD).or(MethodFilter::OPTIONS), handle_filtered)
      .fallback(handle_unmatched),
  )
}
`
  )
  const outputPath = path.join(root, "inventory.json")

  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  assert.deepEqual(
    inventory.surfaces
      .filter(({ kind }) => kind === "wws_route")
      .map(({ surface_id }) => surface_id),
    [
      "wws-route:FALLBACK:/filtered",
      "wws-route:GET:/filtered",
      "wws-route:HEAD:/filtered",
      "wws-route:OPTIONS:/filtered"
    ]
  )
})

test("CLI lets an explicit HEAD handler override GET's implicit HEAD surface", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-wws-head-"))
  await writeRepositoryFixture(root)
  await writeText(
    root,
    "wws/src/route.rs",
    `pub fn build_router() {
  Router::new()
    .route("/resource", get(handle_get))
    .route("/resource", head(handle_head))
}
`
  )
  const outputPath = path.join(root, "inventory.json")

  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const routes = inventory.surfaces.filter(({ kind }) => kind === "wws_route")
  assert.deepEqual(routes.map(({ surface_id }) => surface_id), [
    "wws-route:GET:/resource",
    "wws-route:HEAD:/resource"
  ])
  assert.deepEqual(
    routes.find(({ surface_id }) => surface_id === "wws-route:HEAD:/resource").public_reference,
    ["wws/src/route.rs#head:/resource:handle_head"]
  )
})

test("CLI ignores route-like text in Rust comments, strings, and test-only source", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-wws-lexical-"))
  await writeRepositoryFixture(root)
  await writeText(
    root,
    "wws/src/route.rs",
    `fn identity<'a>(value: &'a str) -> &'a str { value }
pub fn build_router() {
  let quote = '"';
  // .route("/comment", get(comment_handler))
  let example = ".route(\\"/string\\", any(string_handler))";
  let raw = r#".route("/raw-string", get(raw_handler))"#;
  /* .route("/block-comment", get(block_handler)) */
  Router::new().route("/production", any(handle_production))
}

#[cfg(test)]
mod tests {
  fn fixture() { Router::new().route("/test-only", get(test_handler)); }
}
`
  )
  const outputPath = path.join(root, "inventory.json")

  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  assert.deepEqual(
    inventory.surfaces
      .filter(({ kind }) => kind === "wws_route")
      .map(({ surface_id }) => surface_id),
    ["wws-route:ANY:/production"]
  )
})

test("CLI discovers production routes on both sides of a cfg(test) module", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-wws-cfg-test-"))
  await writeRepositoryFixture(root)
  await writeText(
    root,
    "wws/src/route.rs",
    `pub fn first_router() {
  Router::new().route("/before-tests", any(handle_before))
}

#[cfg(test)]
mod tests {
  fn fixture() {
    Router::new().route("/test-only", get(test_handler));
  }
}

pub fn second_router() {
  Router::new().route("/after-tests", get(handle_after))
}
`
  )
  const outputPath = path.join(root, "inventory.json")

  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  assert.deepEqual(
    inventory.surfaces
      .filter(({ kind }) => kind === "wws_route")
      .map(({ surface_id }) => surface_id),
    [
      "wws-route:ANY:/before-tests",
      "wws-route:GET:/after-tests",
      "wws-route:HEAD:/after-tests"
    ]
  )
})

test("CLI fails closed for an unsupported cfg(test)-attributed item", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-wws-cfg-item-"))
  await writeRepositoryFixture(root)
  await writeText(
    root,
    "wws/src/route.rs",
    `pub fn first_router() {
  Router::new().route("/before-tests", any(handle_before))
}

#[cfg(test)]
fn fixture() {
  Router::new().route("/test-only", get(test_handler));
}

pub fn second_router() {
  Router::new().route("/after-tests", any(handle_after))
}
`
  )
  const outputPath = path.join(root, "inventory.json")

  const result = runCli(root, outputPath)

  assert.equal(result.status, 1, result.stderr)
  assert.match(result.stderr, /wws\/src\/route\.rs contains an unsupported cfg\(test\) item/u)
  await assert.rejects(fs.access(outputPath))
})

test("CLI fails closed for unknown, service, dynamic, and malformed WWS route shapes", async () => {
  const cases = [
    ["unknown filter", 'on(MethodFilter::COPY, handler).fallback(unmatched)'],
    ["service endpoint", "service(handler)"],
    ["unknown chain", "get(handler).layer(layer)"],
    ["dynamic path", null],
    ["malformed route", null]
  ]
  for (const [name, endpoint] of cases) {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-wws-closed-"))
    await writeRepositoryFixture(root)
    const declaration = name === "dynamic path"
      ? ".route(route_path, get(handler))"
      : name === "malformed route"
        ? '.route("/malformed", get(handler)'
        : `.route("/closed", ${endpoint})`
    await writeText(
      root,
      "wws/src/route.rs",
      `pub fn build_router() { Router::new()${declaration} }
`
    )
    const outputPath = path.join(root, "inventory.json")

    const result = runCli(root, outputPath)

    assert.equal(result.status, 1, `${name}: ${result.stderr}`)
    assert.match(result.stderr, /wws\/src\/route\.rs contains (?:an unsupported|an unbalanced|an unterminated) route declaration/u)
    await assert.rejects(fs.access(outputPath))
  }
})

test("CLI rejects a duplicate discovered surface", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-duplicate-"))
  await writeRepositoryFixture(root)
  await fs.appendFile(path.join(root, "deepwell/src/api.rs"), 'register!("ping", ping_again);\n')
  const outputPath = path.join(root, "inventory.json")

  const result = runCli(root, outputPath)

  assert.equal(result.status, 1)
  assert.match(result.stderr, /duplicate surface_id: deepwell-jsonrpc:ping/u)
  await assert.rejects(fs.access(outputPath))
})

test("CLI rejects an implementation ledger orphan", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-orphan-"))
  await writeRepositoryFixture(root)
  const ledgerPath = path.join(
    root,
    "docs/wikidot-specifications/implementation-ledger.json"
  )
  const ledger = JSON.parse(await fs.readFile(ledgerPath, "utf8"))
  ledger.features["orphan-feature"] = { status: "pending", tests: [] }
  await writeImplementationLedgerMirrors(root, ledger)

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 1)
  assert.match(result.stderr, /orphan ledger feature: orphan-feature/u)
})

test("CLI projects Catalog source completion only from exact revision witnesses", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-catalog-source-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  const attributionPath = path.join(
    root,
    "docs/development/compatibility-catalog-source-attribution.json"
  )
  await fs.writeFile(attributionPath, `${JSON.stringify({
    schema: "wikijump.compatibility_catalog_source_attribution.v1",
    records: [{
      surface_id: "catalog-feature:feature-one",
      sources: [{ path: "deepwell/src/api.rs", anchor: 'register!("ping", ping);' }],
      tests: [{ path: "framerail/src/routes/+page.svelte", anchor: "<h1>Fixture</h1>" }]
    }]
  }, null, 2)}\n`)
  const outputPath = path.join(root, "inventory.json")

  const result = runCli(root, outputPath)

  assert.equal(result.status, 0, result.stderr)
  const inventory = JSON.parse(await fs.readFile(outputPath, "utf8"))
  const feature = inventory.surfaces.find(
    ({ surface_id: surfaceId }) => surfaceId === "catalog-feature:feature-one"
  )
  assert.equal(feature.source.status, "implemented")
  assert.deepEqual(feature.source.references, [
    'deepwell/src/api.rs#register!("ping", ping);'
  ])
  assert.deepEqual(feature.existing_refs.tests, [
    "framerail/src/routes/+page.svelte#<h1>Fixture</h1>"
  ])
})

test("CLI rejects Catalog source attribution witness drift", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-catalog-source-drift-"))
  cleanupFixture(t, root)
  await writeRepositoryFixture(root)
  const attributionPath = path.join(
    root,
    "docs/development/compatibility-catalog-source-attribution.json"
  )
  await fs.writeFile(attributionPath, `${JSON.stringify({
    schema: "wikijump.compatibility_catalog_source_attribution.v1",
    records: [{
      surface_id: "catalog-feature:feature-one",
      sources: [{ path: "deepwell/src/api.rs", anchor: "missing-source-anchor" }],
      tests: [{ path: "framerail/src/routes/+page.svelte", anchor: "<h1>Fixture</h1>" }]
    }]
  }, null, 2)}\n`)

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 1)
  assert.match(
    result.stderr,
    /source witness drifted for catalog-feature:feature-one/u
  )
})

test("CLI rejects invalid catalog live-observation links", async (t) => {
  const cases = [
    {
      name: "unknown feature",
      mutate: ({ observations }) => observations.observations[0].feature_ids.push("missing-feature"),
      error: /unknown catalog feature: missing-feature/u
    },
    {
      name: "forward only",
      mutate: ({ observations }) => observations.observations[0].feature_ids = [],
      error: /catalog feature-one links observation-one without a reverse link/u
    },
    {
      name: "reverse only",
      mutate: ({ catalog }) => catalog.features[0].live_observation_ids = [],
      error: /live observation observation-one links feature-one without a reverse link/u
    },
    {
      name: "duplicate",
      mutate: ({ observations }) => observations.observations[0].feature_ids.push("feature-one"),
      error: /live observation observation-one has duplicate feature link: feature-one/u
    }
  ]

  for (const fixtureCase of cases) {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-observation-links-"))
    cleanupFixture(t, root)
    await writeRepositoryFixture(root)
    const catalogPath = path.join(root, "docs/wikidot-specifications/catalog.json")
    const observationsPath = path.join(root, "docs/wikidot-specifications/live-observations.json")
    const catalog = JSON.parse(await fs.readFile(catalogPath, "utf8"))
    const observations = JSON.parse(await fs.readFile(observationsPath, "utf8"))
    fixtureCase.mutate({ catalog, observations })
    await fs.writeFile(catalogPath, `${JSON.stringify(catalog, null, 2)}\n`)
    await fs.writeFile(observationsPath, `${JSON.stringify(observations, null, 2)}\n`)
    await refreshCatalogHash(root)

    const result = runCli(root, path.join(root, "inventory.json"))

    assert.equal(result.status, 1, fixtureCase.name)
    assert.match(result.stderr, fixtureCase.error, fixtureCase.name)
  }
})

test("CLI rejects an audit case without an issue owner", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-owner-"))
  await writeRepositoryFixture(root)
  const auditPath = path.join(root, "docs/development/open43-audit-1.json")
  const audit = JSON.parse(await fs.readFile(auditPath, "utf8"))
  delete audit.issues[0].issue
  const serializedAudit = `${JSON.stringify(audit, null, 2)}\n`
  await fs.writeFile(auditPath, serializedAudit)
  const reconciliationPath = path.join(
    root,
    "docs/development/open43-closure-audit-ownership-reconciliation.json"
  )
  const reconciliation = JSON.parse(await fs.readFile(reconciliationPath, "utf8"))
  reconciliation.closure_audits[0].sha256 = sha256(serializedAudit)
  await fs.writeFile(reconciliationPath, `${JSON.stringify(reconciliation, null, 2)}\n`)

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 1)
  assert.match(result.stderr, /audit case without an issue owner/u)
})

test("CLI rejects a status outside the closed vocabulary", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-status-"))
  await writeRepositoryFixture(root)
  const ledgerPath = path.join(
    root,
    "docs/wikidot-specifications/implementation-ledger.json"
  )
  const ledger = JSON.parse(await fs.readFile(ledgerPath, "utf8"))
  ledger.features["feature-one"].status = "mostly-done"
  await writeImplementationLedgerMirrors(root, ledger)

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 1)
  assert.match(result.stderr, /unknown ledger status for feature-one: mostly-done/u)
})

test("CLI rejects a stale Open43 audit digest", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-audit-digest-"))
  await writeRepositoryFixture(root)
  await fs.appendFile(
    path.join(root, "docs/development/open43-audit-1.json"),
    "\n"
  )

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 1)
  assert.match(result.stderr, /reconciliation digest does not match/u)
})

test("CLI rejects ambiguous or mismatched nested Open43 source provenance", async (t) => {
  await t.test("ignores poisoned Git process state", async (t) => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-audit-git-state-"))
    cleanupFixture(t, root)
    await writeRepositoryFixture(root)
    const shim = path.join(root, "bin")
    await fs.mkdir(shim)
    await fs.writeFile(path.join(shim, "git"), "#!/bin/sh\nexit 97\n", { mode: 0o755 })
    const auditPath = path.join(root, "docs/development/open43-audit-1.json")
    const audit = JSON.parse(await fs.readFile(auditPath, "utf8"))
    audit.evidence = [{
      path: "docs/wikidot-specifications/catalog.json",
      sha256: sha256(await fs.readFile(path.join(root, "docs/wikidot-specifications/catalog.json")))
    }]
    const serializedAudit = `${JSON.stringify(audit, null, 2)}\n`
    await fs.writeFile(auditPath, serializedAudit)
    const reconciliationPath = path.join(
      root,
      "docs/development/open43-closure-audit-ownership-reconciliation.json"
    )
    const reconciliation = JSON.parse(await fs.readFile(reconciliationPath, "utf8"))
    reconciliation.closure_audits[0].sha256 = sha256(serializedAudit)
    await fs.writeFile(reconciliationPath, `${JSON.stringify(reconciliation, null, 2)}\n`)

    const result = runCli(root, path.join(root, "inventory.json"), {
      ...process.env,
      PATH: shim,
      GIT_DIR: path.join(root, "poisoned-git-dir")
    })

    assert.equal(result.status, 0, result.stderr)
  })

  for (const [name, mutate, error] of [
    [
      "missing revision interpretation",
      (reconciliation) => { delete reconciliation.closure_audits[0].source_revision },
      /has no source_revision/u
    ],
    [
      "mismatched digest",
      (_reconciliation, audit) => {
        audit.evidence = [{ path: "docs/wikidot-specifications/catalog.json", sha256: "0".repeat(64) }]
      },
      /nested source digest does not match/u
    ]
  ]) {
    await t.test(name, async (t) => {
      const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-audit-source-"))
      cleanupFixture(t, root)
      await writeRepositoryFixture(root)
      const auditPath = path.join(root, "docs/development/open43-audit-1.json")
      const reconciliationPath = path.join(
        root,
        "docs/development/open43-closure-audit-ownership-reconciliation.json"
      )
      const audit = JSON.parse(await fs.readFile(auditPath, "utf8"))
      const reconciliation = JSON.parse(await fs.readFile(reconciliationPath, "utf8"))
      mutate(reconciliation, audit)
      const serializedAudit = `${JSON.stringify(audit, null, 2)}\n`
      await fs.writeFile(auditPath, serializedAudit)
      reconciliation.closure_audits[0].sha256 = sha256(serializedAudit)
      await fs.writeFile(reconciliationPath, `${JSON.stringify(reconciliation, null, 2)}\n`)

      const result = runCli(root, path.join(root, "inventory.json"))

      assert.equal(result.status, 1, result.stderr)
      assert.match(result.stderr, error)
    })
  }

  await t.test("accepts an explicit historical source revision", async (t) => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-audit-history-"))
    cleanupFixture(t, root)
    await writeRepositoryFixture(root)
    const catalogPath = path.join(root, "docs/wikidot-specifications/catalog.json")
    const historicalCatalog = await fs.readFile(catalogPath)
    const reconciliationPath = path.join(
      root,
      "docs/development/open43-closure-audit-ownership-reconciliation.json"
    )
    const reconciliation = JSON.parse(await fs.readFile(reconciliationPath, "utf8"))
    const historicalRevision = reconciliation.closure_audits[0].source_revision
    await fs.appendFile(catalogPath, "\n")
    await refreshCatalogHash(root)
    assert.equal(spawnSync("git", ["add", "."], { cwd: root }).status, 0)
    assert.equal(spawnSync("git", [
      "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
      "commit", "-qm", "new source"
    ], { cwd: root }).status, 0)
    const currentRevision = spawnSync("git", ["rev-parse", "HEAD"], {
      cwd: root,
      encoding: "utf8"
    }).stdout.trim()
    for (const audit of reconciliation.closure_audits) audit.source_revision = currentRevision
    const auditPath = path.join(root, "docs/development/open43-audit-1.json")
    const audit = JSON.parse(await fs.readFile(auditPath, "utf8"))
    audit.evidence = [{
      path: "docs/wikidot-specifications/catalog.json",
      sha256: sha256(historicalCatalog),
      source_revision: historicalRevision
    }]
    const serializedAudit = `${JSON.stringify(audit, null, 2)}\n`
    await fs.writeFile(auditPath, serializedAudit)
    reconciliation.closure_audits[0].sha256 = sha256(serializedAudit)
    await fs.writeFile(reconciliationPath, `${JSON.stringify(reconciliation, null, 2)}\n`)

    const result = runCli(root, path.join(root, "inventory.json"))

    assert.equal(result.status, 0, result.stderr)
  })
})

test("CLI rejects a blocked-evidence routing row outside the audit denominator", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "compatibility-routing-extra-"))
  await writeRepositoryFixture(root)
  const routingPath = path.join(root, "docs/development/open43-blocked-evidence-routing.json")
  const routing = JSON.parse(await fs.readFile(routingPath, "utf8"))
  routing.rows.push({
    case_id: "F999_NOT_IN_AUDITS",
    route_class: "anonymous_read_only",
    status: "not_attempted_not_safe",
    reason: "fixture"
  })
  routing.counts.anonymous_read_only = 1
  routing.counts.total = 1
  await fs.writeFile(routingPath, `${JSON.stringify(routing, null, 2)}\n`)

  const result = runCli(root, path.join(root, "inventory.json"))

  assert.equal(result.status, 1)
  assert.match(result.stderr, /routing rows do not exactly match blocked_evidence cases/u)
})
