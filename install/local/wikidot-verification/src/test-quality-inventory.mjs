import {createHash} from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import {
  extractDeclaredPublicTests, extractDeclaredRustModuleReferences,
} from "../../../../scripts/lib/wikidot-implementation-ledger.mjs";
import {runAuditCommand} from "./audit-command.mjs";

export const AUDIT_SCHEMA = "wikijump.test_quality_audit.v1";
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const jsonHash = (value) => sha256(JSON.stringify(value));
const verifiedProofMaps = new WeakSet();
const HASH = /^[0-9a-f]{64}$/u;
const CODE = /\.(?:[cm]?[jt]sx?|svelte|py|rs|sh|css|scss|c|sql)$/u;
const OWNED = /^(?:deepwell\/|framerail\/|install\/local\/wikidot-verification\/)/u;
const TEST_PATH = /(?:\/(?:tests?|fixtures|seeder)\/|(?:^|\/)(?:tests?|[^/]+_tests)\.rs$|\.(?:test|spec)\.[^.]+$)/u;
const STATES = new Set(["pending", "covered", "removed", "boundary", "equivalent"]);
const CLASSIFICATIONS = new Set([
  "genuine missing regression", "integration-owned", "browser/runtime-owned",
  "startup/config-owned", "instrumentation blind spot", "unreachable/dead code",
  "intentionally unsupported / fail-closed", "equivalent / non-actionable mutant",
]);

export function repositoryPath(root, relative) {
  if (typeof relative !== "string" || path.isAbsolute(relative) || relative.includes("\\") ||
      relative.split("/").some((part) => part === ".." || part === "." || part === "")) {
    throw new Error(`invalid repository-relative path: ${relative}`);
  }
  return path.join(root, relative);
}

export async function repositoryFiles(root) {
  const {stdout} = await runAuditCommand("git", ["ls-files", "--cached", "--others", "--exclude-standard", "-z"], {cwd: root, capture: true});
  return [...new Set(stdout.split("\0").filter(Boolean))].sort();
}

function sourceImports(relative, source, available) {
  const result = new Set();
  if (relative.endsWith(".rs")) {
    const directory = path.posix.dirname(relative);
    const stem = path.posix.basename(relative, ".rs");
    const base = ["lib", "main", "mod"].includes(stem) ? directory : `${directory}/${stem}`;
    for (const reference of extractDeclaredRustModuleReferences(source)) {
      const candidates = reference.path ? [`${directory}/${reference.path}`] :
        [`${base}/${reference.name}.rs`, `${base}/${reference.name}/mod.rs`];
      const resolved = candidates.map((file) => path.posix.normalize(file)).filter((file) => available.has(file));
      resolved.forEach((file) => result.add(file));
    }
  } else {
    // Discovery only. An import relationship is not a behavioral ownership
    // claim; closure still requires a named independently reviewed anchor.
    const patterns = [
      /\b(?:import|export)\s+(?:[^;]*?\s+from\s+)?["'](\.[^"']+)["']/gu,
      /\b(?:import|require)\s*\(\s*["'](\.[^"']+)["']\s*\)/gu,
    ];
    for (const pattern of patterns) for (const match of source.matchAll(pattern)) {
      const base = path.posix.normalize(path.posix.join(path.posix.dirname(relative), match[1]));
      for (const candidate of [base, `${base}.ts`, `${base}.js`, `${base}/index.ts`, `${base}/index.js`]) {
        if (available.has(candidate)) result.add(candidate);
      }
    }
    // Shell/Python validation helpers can be invoked as commands rather than
    // imported. Discover literal repository paths without calling them owners.
    for (const match of source.matchAll(/(?:scripts|install\/local\/wikidot-verification)\/[A-Za-z0-9_./-]+\.(?:mjs|js|py|sh)/gu)) {
      if (available.has(match[0])) result.add(match[0]);
    }
  }
  return [...result].sort();
}

function declaredTests(file, source) {
  if (file.endsWith(".py")) {
    return [...source.matchAll(/^\s*(?:async\s+)?def\s+(test_[A-Za-z0-9_]+)\s*\(/gmu)].map((match) => match[1]);
  }
  return [...(extractDeclaredPublicTests(file, source) ?? [])].sort();
}

export async function buildAuditInventory(root, files = undefined) {
  files ??= await repositoryFiles(root);
  const available = new Set(files);
  const seeds = files.filter((file) => OWNED.test(file) || file.startsWith("scripts/data/") ||
    file.startsWith("docs/wikidot-specifications/") ||
    (file.startsWith("docs/development/") && file.endsWith(".json") && !file.includes("test-quality-audit")) ||
    file.startsWith("install/local/theme-lab/ports/authority-evidence/native-theme-previewer-blank-20261003/") ||
    file.startsWith("install/local/theme-lab/ports/authority-evidence/wikifot-native-site-theme-reset-20261003/") || [
    "scripts/run-test-quality-audit.mjs", "scripts/run-test-no-external-network.sh", "scripts/test-network-guard.c",
    "install/local/theme-lab/fixtures/scp-jp-sidebar.html", "install/local/theme-lab/fixtures/scp-jp-sidebar.provenance.json",
  ].includes(file));
  const sources = new Map();
  const queue = [...seeds];
  for (let index = 0; index < queue.length; index += 1) {
    const file = queue[index];
    if (sources.has(file)) continue;
    const bytes = await fs.readFile(repositoryPath(root, file));
    const source = CODE.test(file) ? bytes.toString("utf8") : "";
    const imports = sourceImports(file, source, available);
    const names = declaredTests(file, source);
    const fixture = /\/(?:fixtures|seeder)\//u.test(file);
    const testOnly = TEST_PATH.test(file);
    sources.set(file, {
      path: file, sha256: sha256(bytes),
      role: !CODE.test(file) ? "input" : fixture ? "fixture" : testOnly ? "test" : "production",
      imports, tests: names.map((name) => ({name, anchor: `${file}#${name}`})),
    });
    for (const dependency of imports) {
      if (!sources.has(dependency)) queue.push(dependency);
    }
  }
  const entries = [...sources.values()].sort((a, b) => a.path.localeCompare(b.path));
  const byPath = new Map(entries.map((entry) => [entry.path, entry]));
  const reachableTests = (file) => {
    const seen = new Set(), tests = new Set();
    const visit = (current) => {
      if (seen.has(current)) return;
      seen.add(current);
      const entry = byPath.get(current);
      if (!entry) return;
      for (const test of entry.tests) tests.add(test.anchor);
      entry.imports.forEach(visit);
    };
    visit(file);
    return [...tests].sort();
  };
  const importedTests = new Set(entries.flatMap((entry) => entry.imports));
  const entrypoints = entries.filter((entry) => entry.role === "test" && /\.test\.[cm]?[jt]s$/u.test(entry.path) && !importedTests.has(entry.path)).map((entry) => entry.path);
  const executables = entries.filter((entry) => entrypoints.includes(entry.path) || /\.spec\.ts$/u.test(entry.path)).map((entry) => ({
    kind: entry.path.endsWith(".spec.ts") ? "browser" : "node", path: entry.path,
    anchors: reachableTests(entry.path),
  }));
  if (available.has("deepwell/Cargo.toml")) {
    const metadata = JSON.parse((await runAuditCommand("cargo", ["metadata", "--offline", "--locked", "--no-deps", "--format-version", "1", "--manifest-path", "deepwell/Cargo.toml"], {cwd: root, capture: true})).stdout);
    for (const pkg of metadata.packages) for (const target of pkg.targets) {
      executables.push({kind: "rust", package: pkg.name, target: target.name,
        cargo_kinds: target.kind, required_features: target["required-features"] ?? [],
        path: path.relative(root, target.src_path).split(path.sep).join("/"),
        anchors: reachableTests(path.relative(root, target.src_path).split(path.sep).join("/")),
      });
    }
  }
  const inputs = [];
  for (const file of ["deepwell/Cargo.toml", "deepwell/Cargo.lock", "deepwell/relation-impl-derive/Cargo.toml",
    "framerail/package.json", "framerail/pnpm-lock.yaml", "install/local/wikidot-verification/package.json",
    "install/local/wikidot-verification/pnpm-lock.yaml", ".cargo/config.toml", "rust-toolchain.toml"]) {
    if (available.has(file)) inputs.push({path: file, sha256: sha256(await fs.readFile(repositoryPath(root, file)))});
  }
  return {schema: AUDIT_SCHEMA, entries, inputs, entrypoints, executables, digest: jsonHash({entries, inputs, entrypoints, executables})};
}

export function createAuditLedger(inventory, owners = []) {
  return {
    schema: AUDIT_SCHEMA,
    scope: ["deepwell", "framerail", "wikidot-verification", "executed shared helpers"],
    inventory_digest: inventory.digest,
    inputs: inventory.inputs,
    owners,
    records: inventory.entries.filter((entry) => entry.role === "production").map((entry) => ({
      path: entry.path, sha256: entry.sha256, status: "pending", anchors: [], findings: [], review: null,
    })),
  };
}

function requireValue(condition, message) {
  if (!condition) throw new Error(message);
}

function independentReview(review, context) {
  requireValue(typeof review?.author === "string" && review.author.length > 0 &&
    typeof review.reviewer === "string" && review.reviewer.length > 0 &&
    review.author !== review.reviewer && review.accepted === true &&
    Array.isArray(review.evidence) && review.evidence.length > 0,
  `independent review with bound evidence required: ${context}`);
}

// Proofs whose inventory binding is stale, recorded instead of rejected when a
// caller asks for an incomplete report, so the report names the evidence to
// regenerate rather than stopping at the first stale proof.
const staleProofIds = new WeakMap();

export async function loadAuditProofs(root, ledger, inventory, {allowStale = false} = {}) {
  const proofs = new Map();
  const stale = new Set();
  for (const reference of ledger.proofs ?? []) {
    requireValue(typeof reference.id === "string" && !proofs.has(reference.id), "duplicate/invalid proof id");
    const bytes = await fs.readFile(repositoryPath(root, reference.path));
    requireValue(sha256(bytes) === reference.sha256, `stale proof bytes: ${reference.id}`);
    const proof = JSON.parse(bytes);
    requireValue(proof.id === reference.id && proof.kind === reference.kind, `proof identity mismatch: ${reference.id}`);
    // A mutation proof is bound to its owner's exact source and definition
    // by validateMutationReceipt; tying it to the whole-repository inventory
    // would invalidate it whenever any unrelated production file changed.
    if (!["observation", "mutation"].includes(proof.kind) && proof.inventory_digest !== ledger.inventory_digest) {
      requireValue(allowStale, `stale proof inventory: ${reference.id}`);
      stale.add(reference.id);
    }
    proofs.set(reference.id, proof);
  }
  for (const proof of proofs.values()) {
    if (!["assessment", "acceptance"].includes(proof.kind)) {
      requireValue(proof.artifacts?.length > 0, `missing supporting artifacts: ${proof.id}`);
      for (const artifact of proof.artifacts) {
        requireValue(HASH.test(artifact.sha256), `invalid artifact hash: ${proof.id}`);
        const file = path.isAbsolute(artifact.path) ? artifact.path : repositoryPath(root, artifact.path);
        requireValue((await fs.lstat(file)).isFile(), `artifact must be a regular file: ${proof.id}`);
        requireValue(sha256(await fs.readFile(file)) === artifact.sha256, `stale supporting artifact: ${proof.id}`);
      }
    }
    if (proof.kind === "acceptance") {
      const delivery = proof.delivery;
      requireValue(/^[0-9a-f]{40}$/u.test(delivery?.merged_revision ?? ""), "invalid merged Git identity");
      const parents = (await runAuditCommand("git", ["rev-list", "--parents", "-n", "1", delivery.merged_revision], {cwd: root, capture: true})).stdout.trim().split(" ").slice(1);
      requireValue(parents.length === 2 && jsonHash(parents) === jsonHash(delivery.merge_parents), "unverified two-parent merge identity");
      requireValue(/^https:\/\/github\.com\/Rokurolize\/wikijump\/pull\/[1-9][0-9]*$/u.test(delivery.pr_url), "invalid delivered PR identity");
      const pr = JSON.parse((await runAuditCommand("gh", ["pr", "view", delivery.pr_url, "--json", "state,mergeCommit,baseRefName"], {cwd: root, capture: true})).stdout);
      requireValue(pr.state === "MERGED" && pr.baseRefName === "develop" && pr.mergeCommit?.oid === delivery.merged_revision, "PR delivery does not match immutable merge");
      inventory ??= await buildAuditInventory(root);
      const files = [...inventory.entries.map((entry) => entry.path), ...ledger.inputs.map((input) => input.path)];
      const tracked = new Set((await runAuditCommand("git", ["ls-tree", "-r", "--name-only", "HEAD"], {cwd: root, capture: true})).stdout.trim().split("\n"));
      requireValue(files.every((file) => tracked.has(file)), "uncommitted audit sources cannot bind delivery");
      await runAuditCommand("git", ["diff", "--exit-code", delivery.merged_revision, "--", ...files], {cwd: root, capture: true});
    }
  }
  verifiedProofMaps.add(proofs);
  staleProofIds.set(proofs, stale);
  return proofs;
}

export function validateAuditLedger(ledger, inventory, {allowIncomplete = false, proofs = new Map()} = {}) {
  requireValue(ledger.schema === AUDIT_SCHEMA, "unsupported test-quality ledger schema");
  requireValue(ledger.inventory_digest === inventory.digest, "stale inventory identity");
  requireValue(jsonHash(ledger.inputs) === jsonHash(inventory.inputs), "stale lock/toolchain inputs");
  const expected = new Map(inventory.entries.filter((entry) => entry.role === "production").map((entry) => [entry.path, entry]));
  const anchors = new Set(inventory.entries.flatMap((entry) => entry.tests.map((test) => test.anchor)));
  const owners = new Map();
  for (const owner of ledger.owners) {
    requireValue(typeof owner.id === "string" && /^[a-z0-9-]+$/u.test(owner.id) && !owners.has(owner.id), "duplicate/invalid executable owner id");
    owners.set(owner.id, owner);
    requireValue(["rust", "node", "browser", "python", "startup"].includes(owner.kind), `unsupported owner kind: ${owner.id}`);
    requireValue(owner.files.length > 0 && owner.anchors.length > 0, `empty executable owner: ${owner.id}`);
    for (const file of owner.files) requireValue(expected.has(file), `unknown executable owner source: ${file}`);
    for (const anchor of owner.anchors) requireValue(anchors.has(anchor), `nonexistent executable owner anchor: ${anchor}`);
  }
  const stale = new Set(staleProofIds.get(proofs) ?? []);
  const proofFor = (id, kind) => {
    const proof = proofs.get(id);
    requireValue(proof && proof.kind === kind, `missing/wrong bound ${kind} proof: ${id}`);
    if (proof.inventory_digest !== inventory.digest && !["observation", "mutation"].includes(kind)) stale.add(id);
    return proof;
  };
  const review = (value, context) => {
    independentReview(value, context);
    for (const id of value.evidence) requireValue(proofs.has(id), `missing review evidence: ${id}`);
  };
  // Validate every mutation proof, including ones not yet accepted for closure.
  let unresolvedMutations = 0;
  const reviewedMutationOwners = new Set();
  for (const proof of proofs.values()) if (proof.kind === "mutation") {
    const owner = owners.get(proof.owner);
    requireValue(owner, `missing mutation executable owner: ${proof.owner}`);
    reviewedMutationOwners.add(owner.id);
    const sourceHash = owner.kind === "rust" ? jsonHash(owner.files.map((file) => [file, expected.get(file).sha256])) : expected.get(owner.files[0]).sha256;
    unresolvedMutations += validateMutationReceipt(proof, {owner: owner.id, definition: owner, sourceHash, inventoryHash: owner.mutation_inventory_sha256, ownerHash: jsonHash(owner), proofs, allowIncomplete}).unresolved;
  }
  for (const owner of owners.values()) {
    if (owner.mutation_inventory_sha256 && !reviewedMutationOwners.has(owner.id)) unresolvedMutations += 1;
  }
  const seen = new Set();
  let pending = 0;
  for (const record of ledger.records) {
    requireValue(!seen.has(record.path), `duplicate production record: ${record.path}`);
    seen.add(record.path);
    requireValue(expected.has(record.path), `unknown production record: ${record.path}`);
    requireValue(record.sha256 === expected.get(record.path).sha256, `stale source: ${record.path}`);
    requireValue(STATES.has(record.status), `invalid status: ${record.path}`);
    for (const anchor of record.anchors) requireValue(anchors.has(anchor), `nonexistent named anchor: ${anchor}`);
    if (record.status === "pending" || record.findings.some((finding) => finding.status === "pending")) pending += 1;
    if (record.status !== "pending") {
      review(record.review, record.path);
      requireValue(record.findings.length > 0, `material-region assessment required: ${record.path}`);
      requireValue(record.owner_ids?.length > 0, `behavioral owner required: ${record.path}`);
      for (const id of record.owner_ids) {
        const owner = owners.get(id);
        requireValue(owner?.files.includes(record.path), `owner does not own source: ${record.path}`);
        requireValue(record.anchors.some((anchor) => owner.anchors.includes(anchor)), `owner anchor mismatch: ${record.path}`);
      }
      const assessment = proofFor(record.assessment, "assessment");
      requireValue(assessment.path === record.path && assessment.source_sha256 === record.sha256 &&
        jsonHash(assessment.findings) === jsonHash(record.findings) &&
        jsonHash(assessment.owner_ids) === jsonHash(record.owner_ids), `assessment/source disagreement: ${record.path}`);
      review(assessment.review, record.path);
    }
    for (const finding of record.findings) {
      requireValue(CLASSIFICATIONS.has(finding.classification), `invalid classification: ${record.path}`);
      requireValue(STATES.has(finding.status), `invalid finding status: ${record.path}`);
      requireValue(!(finding.status !== "pending" && finding.classification === "genuine missing regression"), `unresolved regression classified terminal: ${record.path}`);
      if (finding.status !== "pending") {
        review(finding.review, record.path);
        requireValue(finding.evidence?.length > 0, `finding evidence required: ${record.path}`);
        for (const id of finding.evidence) requireValue(proofs.has(id), `missing finding evidence: ${id}`);
        if (finding.classification === "instrumentation blind spot") {
          requireValue(record.anchors.length > 0 && finding.instrumentation_evidence, `blind spot requires instrumentation proof and behavioral owner: ${record.path}`);
          proofFor(finding.instrumentation_evidence, "coverage");
        }
      }
    }
  }
  for (const file of expected.keys()) requireValue(seen.has(file), `omitted production owner: ${file}`);
  let accepted = false;
  if (ledger.acceptance) {
    requireValue(verifiedProofMaps.has(proofs), "closure requires artifact and delivery verification");
    const acceptance = proofFor(ledger.acceptance, "acceptance");
    review(acceptance.review, "closure");
    requireValue(pending === 0 && unresolvedMutations === 0 && acceptance.records_sha256 === jsonHash(ledger.records), "acceptance does not bind resolved records and mutations");
    for (const owner of owners.values()) if (owner.mutation_inventory_sha256) {
      requireValue([...proofs.values()].some((proof) => proof.kind === "mutation" && proof.owner === owner.id), `missing declared mutation run: ${owner.id}`);
    }
    const checks = new Map([
      ["offline:portable", ["pnpm", "--dir", "install/local/wikidot-verification", "offline:portable"]],
      ["proc-macro", ["cargo", "test", "deepwell/Cargo.toml", "deepwell-relation-impl-derive", "--lib"]],
      ["generated-contracts", ["node", "install/local/wikidot-verification/scripts/verify-repository-generated-contracts.mjs", "--full"]],
      ["corpus-pinned-literals", ["node", "install/local/wikidot-verification/scripts/check-corpus-pinned-literals.mjs"]],
      ["wikijump-identifier-leaks", ["node", "install/local/wikidot-verification/scripts/check-wikijump-identifier-leaks.mjs"]],
      ["preflight:final", ["scripts/preflight.sh", "--final"]],
    ]);
    for (const [check, requiredArgs] of checks) {
      requireValue(acceptance.validations.some((id) => {
        const validation = proofFor(id, "validation");
        return validation.check === check && validation.code === 0 &&
          requiredArgs.every((arg) => validation.command?.includes(arg)) &&
          Object.values(validation.tool_versions ?? {}).length > 0 &&
          Object.values(validation.tool_versions).every((version) => typeof version === "string" && version.length > 0) &&
          HASH.test(validation.log_sha256) && validation.artifacts.some((artifact) => artifact.sha256 === validation.log_sha256);
      }), `missing successful closure validation: ${check}`);
    }
    requireValue(acceptance.delivery?.merge_parents?.length === 2 && acceptance.delivery.merged_revision && acceptance.delivery.pr_url, "normal two-parent PR delivery required");
    requireValue(!acceptance.standing_required || acceptance.standing?.merged_revision === acceptance.delivery.merged_revision, "merged-head standing proof required");
    accepted = true;
  }
  requireValue(allowIncomplete || (pending === 0 && unresolvedMutations === 0 && stale.size === 0 && accepted), `${pending} production records remain unresolved or final acceptance is missing${stale.size ? `; stale proofs to regenerate: ${[...stale].sort().join(", ")}` : ""}; issue #1990 cannot close`);
  return {
    production_files: expected.size,
    unresolved_records: pending,
    unresolved_mutations: unresolvedMutations,
    stale_proofs: [...stale].sort(),
    closure_ready: pending === 0 && unresolvedMutations === 0 && stale.size === 0 && accepted,
  };
}

export function validateMutationReceipt(receipt, {owner, definition, sourceHash, inventoryHash, ownerHash, proofs = new Map(), allowIncomplete = false}) {
  requireValue(receipt.owner === owner && receipt.source_sha256 === sourceHash && receipt.owner_sha256 === ownerHash, "stale mutation source/owner identity");
  requireValue(receipt.inventory_sha256 === inventoryHash, "mismatched mutation inventory");
  requireValue(jsonHash(receipt.inventory) === inventoryHash, "mutation inventory bytes disagree with identity");
  requireValue(definition && jsonHash(definition) === ownerHash, "missing/mismatched executable owner definition");
  if (definition.kind === "node") requireValue(jsonHash(receipt.command) === jsonHash(definition.command), "mutation command does not match owner");
  else requireValue(receipt.runs?.length > 0 && receipt.runs.every((run) => definition.targets.some((target) => jsonHash(target) === jsonHash(run.target))), "mutation targets do not match owner");
  const artifactHashes = new Set((receipt.artifacts ?? []).map((artifact) => artifact.sha256));
  requireValue(receipt.source_restored === true && receipt.tool && receipt.command?.length > 0 && receipt.baselines?.length > 0 && receipt.baselines.every((baseline) => baseline.code === 0 && baseline.elapsed_ms > 0 && HASH.test(baseline.log_sha256) && artifactHashes.has(baseline.log_sha256)), "successful bound mutation baseline/restoration required");
  const expected = new Set(receipt.inventory.map((entry) => entry.id));
  requireValue(expected.size === receipt.inventory.length, "duplicate mutation inventory id");
  const seen = new Set();
  let unresolved = 0;
  for (const outcome of receipt.outcomes) {
    requireValue(expected.has(outcome.id) && !seen.has(outcome.id), "unknown/duplicate mutation outcome");
    seen.add(outcome.id);
    requireValue(["caught", "missed", "unviable", "timeout"].includes(outcome.outcome), "unknown mutation outcome");
    const targetMatches = definition.kind === "node" ? jsonHash(outcome.target) === jsonHash(definition.command) : definition.targets.some((target) => jsonHash(target) === jsonHash(outcome.target));
    requireValue(targetMatches && HASH.test(outcome.log_sha256) && artifactHashes.has(outcome.log_sha256), "mutation outcome lacks owning target/log identity");
    if (outcome.outcome !== "caught") {
      const dispositions = outcome.outcome === "missed" ? ["equivalent / non-actionable mutant", "intentionally unsupported / fail-closed"] : outcome.outcome === "timeout" ? ["demonstrated mutation-induced nonprogress"] : ["compilation failure"];
      if (!outcome.disposition) { unresolved += 1; continue; }
      requireValue(dispositions.includes(outcome.disposition), "contradictory mutation disposition");
      independentReview(outcome.review, outcome.id);
      requireValue(outcome.evidence?.length > 0, "mutation disposition requires independent evidence");
      for (const id of [...outcome.evidence, ...outcome.review.evidence]) requireValue(proofs.has(id), `missing mutation disposition evidence: ${id}`);
    }
  }
  requireValue(expected.size === seen.size, "unexecuted mutation inventory entries");
  requireValue(allowIncomplete || unresolved === 0, "unresolved surviving/unviable/timed-out mutations require independent disposition");
  return {unresolved};
}
