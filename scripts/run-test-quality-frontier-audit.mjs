#!/usr/bin/env node

import {createHash} from "node:crypto";
import {existsSync, mkdirSync, readFileSync, readdirSync, realpathSync, statSync, writeFileSync} from "node:fs";
import {dirname, extname, isAbsolute, relative, resolve} from "node:path";
import {fileURLToPath} from "node:url";

import {
  runValidationCommand,
  withDeepwellIntegrationStack,
} from "../install/local/wikidot-verification/src/deepwell-integration-stack.mjs";
import {summarizeLlvmCoverage, summarizeRawV8Coverage} from "./test-quality-coverage-metrics.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..");
const ledgerPath = process.env.WIKIJUMP_TEST_QUALITY_LEDGER
  ? resolve(process.env.WIKIJUMP_TEST_QUALITY_LEDGER)
  : resolve(root, "docs/development/test-quality-audit-frontier-20261008.json");
const commandName = process.argv[2];
process.chdir(root);

function fail(message) {
  throw new Error(message);
}

function parseOptions(argv) {
  const options = {outputDir: null, owner: null, shard: null, mutationRun: null};
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === "--output-dir") options.outputDir = argv[++index] ?? null;
    else if (argument === "--owner") options.owner = argv[++index] ?? null;
    else if (argument === "--shard") options.shard = argv[++index] ?? null;
    else if (argument === "--mutation-run") options.mutationRun = argv[++index] ?? null;
    else fail(`unknown argument: ${argument}`);
  }
  if (!options.outputDir) fail("--output-dir <directory> is required");
  if (options.shard) {
    const match = /^(0|[1-9][0-9]*)\/([1-9][0-9]*)$/u.exec(options.shard);
    if (!match || Number(match[1]) >= Number(match[2])) {
      fail("--shard must use cargo-mutants zero-based k/n form with 0 <= k < n, for example 0/4");
    }
  }
  options.outputDir = resolve(root, options.outputDir);
  return options;
}

function sha256File(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function repositoryPath(path) {
  return relative(root, path).replaceAll("\\", "/");
}

function walk(directory, predicate) {
  if (!existsSync(directory)) return [];
  const output = [];
  for (const entry of readdirSync(directory, {withFileTypes: true})) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) output.push(...walk(path, predicate));
    else if (entry.isFile() && predicate(path)) output.push(path);
  }
  return output.sort();
}

const EXECUTABLE_SOURCE_EXTENSIONS = new Set([".rs", ".js", ".mjs", ".ts", ".svelte", ".py", ".sh"]);
const VISUAL_SOURCE_EXTENSIONS = new Set([".css", ".scss", ".sass", ".html"]);

function sourceLike(path) {
  return EXECUTABLE_SOURCE_EXTENSIONS.has(extname(path));
}

function productionSourceLike(path) {
  // Styling and HTML document shells are observable Wikidot compatibility
  // surfaces, but are not executable test owners or proven coverage.
  return sourceLike(path) || VISUAL_SOURCE_EXTENSIONS.has(extname(path));
}

function dedicatedTestModule(path) {
  const normalized = repositoryPath(path);
  return /(?:^|\/)tests(?:\/|\.rs$)|\.(?:test|spec)\.[^.]+$/u.test(normalized);
}

function containsInlineRustTests(path) {
  return extname(path) === ".rs" && readFileSync(path, "utf8").includes("#[cfg(test)]");
}

function productionFiles() {
  return [
    ...walk(resolve(root, "deepwell/src"), (path) => productionSourceLike(path) && !dedicatedTestModule(path)),
    ...walk(resolve(root, "deepwell/relation-impl-derive/src"), (path) => productionSourceLike(path) && !dedicatedTestModule(path)),
    ...walk(resolve(root, "framerail/src"), productionSourceLike),
    ...["article-response-fast-path.js", "server.js", "startup-readiness.js"]
      .map((name) => resolve(root, "framerail", name)).filter(existsSync),
    ...walk(resolve(root, "install/local/wikidot-verification/src"), productionSourceLike),
    ...walk(resolve(root, "install/local/wikidot-verification/scripts"), productionSourceLike),
  ];
}

function testFiles() {
  const files = [
    ...walk(resolve(root, "deepwell/tests"), sourceLike),
    ...walk(resolve(root, "deepwell/src"), (path) => sourceLike(path) && (dedicatedTestModule(path) || containsInlineRustTests(path))),
    ...walk(resolve(root, "deepwell/relation-impl-derive/src"), (path) => sourceLike(path) && containsInlineRustTests(path)),
    ...walk(resolve(root, "framerail/tests"), sourceLike),
    ...walk(resolve(root, "install/local/wikidot-verification/tests"), sourceLike),
  ];
  return [...new Set(files)].sort();
}

function relativeImports(path) {
  const text = readFileSync(path, "utf8");
  const results = new Set();
  // Resolve ordinary imports/exports, bare side-effect imports used by test
  // suite wrappers, and static import()/require() edges. Dynamic expressions
  // have no statically provable target and must not be invented here.
  const pattern = /(?:\bfrom\s*|\bimport\s*(?:\(\s*)?|\brequire\s*\(\s*)["'](\.{1,2}\/[^"']+)["']/gu;
  for (const match of text.matchAll(pattern)) {
    const base = resolve(dirname(path), match[1]);
    for (const candidate of [base, ...[".mjs", ".js", ".ts", ".svelte"].map((suffix) => `${base}${suffix}`)]) {
      if (existsSync(candidate) && statSync(candidate).isFile()) {
        results.add(repositoryPath(candidate));
        break;
      }
    }
  }
  return [...results].sort();
}

function reachableRelativeImports(path) {
  const start = repositoryPath(path);
  const visited = new Set([start]);
  const pending = [path];
  while (pending.length > 0) {
    const next = pending.pop();
    for (const imported of relativeImports(next)) {
      if (visited.has(imported)) continue;
      visited.add(imported);
      if (sourceLike(imported)) pending.push(resolve(root, imported));
    }
  }
  visited.delete(start);
  return [...visited].sort();
}

function nodeOwnerRecord(path) {
  const text = readFileSync(path, "utf8");
  return {
    path: repositoryPath(path),
    sha256: sha256File(path),
    imported_modules: relativeImports(path),
    reachable_imported_modules: reachableRelativeImports(path),
    browser_owner: /\b(?:playwright|chromium|firefox|webkit|page\.goto)\b/u.test(text),
    wrapper_suite: /(?:spawn|execFile|execFileSync|runValidationCommand)[\s\S]{0,400}\b(?:test|playwright|cargo)\b/u.test(text),
  };
}

async function cargoTargets() {
  const manifests = [
    "deepwell/Cargo.toml",
    "deepwell/relation-impl-derive/Cargo.toml",
  ];
  const targets = [];
  for (const manifest of manifests) {
    const {stdout} = await runValidationCommand("cargo", [
      "metadata", "--format-version", "1", "--no-deps", "--offline", "--locked",
      "--manifest-path", manifest,
    ], {capture: true, cwd: root});
    const metadata = JSON.parse(stdout);
    for (const pkg of metadata.packages) {
      for (const target of pkg.targets) {
        targets.push({
          package: pkg.name,
          name: target.name,
          kind: target.kind,
          crate_types: target.crate_types,
          src_path: repositoryPath(target.src_path),
        });
      }
    }
  }
  return [...new Map(targets.map((target) => [`${target.package}:${target.name}:${target.kind.join(",")}`, target])).values()];
}

async function toolVersion(command, args = ["--version"]) {
  try {
    const {stdout, stderr} = await runValidationCommand(command, args, {capture: true, cwd: root});
    return (stdout || stderr).trim().split("\n")[0];
  } catch (error) {
    return `unavailable: ${error.message}`;
  }
}

async function toolVersions() {
  return {
    node: process.version,
    cargo: await toolVersion("cargo"),
    rustc: await toolVersion("rustc"),
    cargo_llvm_cov: await toolVersion("cargo", ["llvm-cov", "--version"]),
    cargo_mutants: await toolVersion("cargo", ["mutants", "--version"]),
    pnpm: await toolVersion("pnpm"),
  };
}

function writeJson(path, value) {
  mkdirSync(dirname(path), {recursive: true});
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function loadLedger() {
  return JSON.parse(readFileSync(ledgerPath, "utf8"));
}

async function inventory(outputDir) {
  const production = productionFiles().map((path) => ({path: repositoryPath(path), sha256: sha256File(path)}));
  const tests = testFiles().map(nodeOwnerRecord);
  const targets = await cargoTargets();
  // Static import reachability is only a candidate owner link. In particular,
  // Rust test ownership, spawned CLI suites, transformed SvelteKit modules,
  // and computed dynamic imports cannot be proved by this graph.
  const candidates = new Map(production.map(({path}) => [path, []]));
  for (const owner of tests) {
    for (const source of owner.reachable_imported_modules) {
      candidates.get(source)?.push(owner.path);
    }
  }
  const staticImportOwnership = [...candidates].map(([source, ownerPaths]) => ({
    source, static_importing_tests: ownerPaths.sort(),
  }));
  const candidateCount = staticImportOwnership.filter(({static_importing_tests}) => static_importing_tests.length > 0).length;
  const report = {
    schema: 1,
    generated_at: new Date().toISOString(),
    tool_versions: await toolVersions(),
    production,
    executable_owners: tests,
    static_import_ownership: {
      semantics: "candidate importer only; not behavioral coverage, test execution, or a missing-test verdict",
      production_file_count: production.length,
      with_static_import_candidates: candidateCount,
      without_static_import_candidates: production.length - candidateCount,
      source_candidates: staticImportOwnership,
    },
    cargo_targets: targets,
    deepwell_binaries: targets.filter((target) => target.kind.includes("bin")),
    proc_macros: targets.filter((target) => target.kind.includes("proc-macro")),
  };
  writeJson(resolve(outputDir, "inventory.json"), report);
  process.stdout.write(`Inventoried ${production.length} production files and ${tests.length} executable test owners.\n`);
}

async function runCoverageCommand(command, args, {env, capture = false} = {}) {
  return runValidationCommand(command, args, {env, capture, cwd: root});
}

async function coverage(outputDir) {
  mkdirSync(outputDir, {recursive: true});
  const summaryPath = resolve(outputDir, "coverage-summary.json");
  if (existsSync(summaryPath)) {
    fail(`coverage output already has a run manifest; use a fresh --output-dir: ${summaryPath}`);
  }
  const production = productionFiles();
  const expectedFiles = (relativePrefix) => production
    .map(repositoryPath)
    .filter((path) => path.startsWith(`${relativePrefix}/`))
    .map((path) => path.slice(relativePrefix.length + 1));
  const report = {
    schema: 1,
    generated_at: new Date().toISOString(),
    tool_versions: await toolVersions(),
    rust_branch_instrumentation: "unsupported on the repository stable toolchain; line/function/region coverage plus mutation is retained instead",
    commands: [],
    instrumentation_limitations: [],
    measurements: {},
    complete: false,
  };

  const record = async (name, command, args, options = {}) => {
    const row = {name, command: [command, ...args], status: "running"};
    report.commands.push(row);
    writeJson(summaryPath, report);
    try {
      const result = await runCoverageCommand(command, args, options);
      row.status = "completed";
      return result;
    } catch (error) {
      row.status = "failed";
      row.error = String(error.message).slice(0, 1000);
      throw error;
    } finally {
      writeJson(summaryPath, report);
    }
  };

  const recordLlvm = (name, filename, sourceDir, relativePrefix) => {
    report.measurements[name] = summarizeLlvmCoverage(
      JSON.parse(readFileSync(resolve(outputDir, filename), "utf8")),
      {sourcePrefix: resolve(root, sourceDir), expectedFiles: expectedFiles(relativePrefix)},
    );
    writeJson(summaryPath, report);
  };

  await record("deepwell-unit", "cargo", [
    "llvm-cov", "--lib", "--json", "--output-path", resolve(outputDir, "deepwell-unit.json"),
    "--offline", "--locked", "--manifest-path", "deepwell/Cargo.toml",
  ]);
  recordLlvm("deepwell-unit", "deepwell-unit.json", "deepwell/src", "deepwell/src");

  await withDeepwellIntegrationStack(async ({env, cargo}) => {
    await record("deepwell-integration", cargo, [
      "llvm-cov", "--tests", "--json", "--output-path", resolve(outputDir, "deepwell-integration.json"),
      "--offline", "--locked", "--manifest-path", "deepwell/Cargo.toml", "--", "--test-threads", "1",
    ], {env});
    recordLlvm("deepwell-integration", "deepwell-integration.json", "deepwell/src", "deepwell/src");
    await record("deepwell-combined", cargo, [
      "llvm-cov", "--lib", "--tests", "--json", "--output-path", resolve(outputDir, "deepwell-combined.json"),
      "--offline", "--locked", "--manifest-path", "deepwell/Cargo.toml", "--", "--test-threads", "1",
    ], {env});
    recordLlvm("deepwell-combined", "deepwell-combined.json", "deepwell/src", "deepwell/src");
  });

  await record("relation-proc-macro", "cargo", [
    "llvm-cov", "--json", "--output-path", resolve(outputDir, "relation-proc-macro.json"),
    "--offline", "--locked", "--manifest-path", "deepwell/relation-impl-derive/Cargo.toml",
  ]);
  recordLlvm("relation-proc-macro", "relation-proc-macro.json", "deepwell/relation-impl-derive/src", "deepwell/relation-impl-derive/src");

  const framerailCoverage = resolve(outputDir, "framerail-v8");
  mkdirSync(framerailCoverage, {recursive: true});
  if (readdirSync(framerailCoverage).some((name) => /^coverage-.*\.json$/u.test(name))) {
    fail("Framerail V8 output contains previous raw profiles; use a fresh --output-dir");
  }
  await record("framerail-v8", "scripts/run-framerail-unit-tests.sh", [], {
    env: {...process.env, NODE_V8_COVERAGE: framerailCoverage},
  });
  report.measurements["framerail-v8"] = summarizeRawV8Coverage(framerailCoverage, {
    sourcePrefix: resolve(root, "framerail/src"), expectedFiles: expectedFiles("framerail/src"),
  });
  writeJson(summaryPath, report);

  const verifierCoverage = resolve(outputDir, "wikidot-verification-v8");
  mkdirSync(verifierCoverage, {recursive: true});
  if (readdirSync(verifierCoverage).some((name) => /^coverage-.*\.json$/u.test(name))) {
    fail("wikidot-verification V8 output contains previous raw profiles; use a fresh --output-dir");
  }
  try {
    await record("wikidot-verification-v8", "pnpm", ["--dir", "install/local/wikidot-verification", "run", "test:ci"], {
      env: {...process.env, NODE_V8_COVERAGE: verifierCoverage},
      capture: true,
    });
    report.measurements["wikidot-verification-v8"] = summarizeRawV8Coverage(verifierCoverage, {
      sourcePrefix: resolve(root, "install/local/wikidot-verification"),
      expectedFiles: production.map(repositoryPath)
        .filter((path) => path.startsWith("install/local/wikidot-verification/"))
        .map((path) => path.slice("install/local/wikidot-verification/".length)),
    });
  } catch (error) {
    report.instrumentation_limitations.push({
      owner: "wikidot-verification subprocess suites",
      limitation: "NODE_V8_COVERAGE changes frozen subprocess environments; preserve the uninstrumented behavior and audit those owners independently",
      observed_error: String(error.message).slice(0, 1000),
    });
    await record("wikidot-verification-uninstrumented-control", "pnpm", ["--dir", "install/local/wikidot-verification", "run", "test:ci"]);
  }

  report.complete = true;
  writeJson(summaryPath, report);
}

async function listedMutationCount(owner) {
  const {stdout} = await runValidationCommand("cargo", [
    "mutants", "--list", "--manifest-path", "deepwell/Cargo.toml",
    "-f", owner.mutation.file, "-F", owner.mutation.function,
  ], {capture: true, cwd: root});
  return stdout.split("\n").filter((line) => line.trim() !== "").length;
}

function summarizeMutationRun(runOutput) {
  const path = resolve(runOutput, "mutants.out/outcomes.json");
  if (!existsSync(path)) fail(`cargo-mutants did not write outcomes: ${repositoryPath(path)}`);
  const rows = JSON.parse(readFileSync(path, "utf8")).outcomes ?? [];
  const counts = {caught: 0, missed: 0, unviable: 0, timeout: 0};
  for (const row of rows) {
    if (row.summary === "CaughtMutant") counts.caught += 1;
    else if (row.summary === "MissedMutant") counts.missed += 1;
    else if (row.summary === "Unviable" || row.summary === "UnviableMutant") counts.unviable += 1;
    else if (row.summary === "Timeout") counts.timeout += 1;
  }
  return counts;
}

async function mutate(outputDir, ownerId, shard, mutationRun) {
  if (!ownerId) fail("mutate requires --owner <id>");
  const owner = loadLedger().owners.find((row) => row.id === ownerId);
  if (!owner) fail(`unknown audit owner: ${ownerId}`);
  if (!owner.mutation) fail(`owner ${ownerId} has no mutation descriptor`);
  if (owner.mutation.removed) fail(`owner ${ownerId} was removed after reviewed mutation evidence; mutation replay is no longer applicable`);
  const sourcePath = resolve(root, owner.source.path);
  const originalHash = sha256File(sourcePath);
  if (originalHash !== owner.source.sha256) fail(`owner source hash is stale before mutation: ${owner.source.path}`);
  const count = await listedMutationCount(owner);
  if (count !== owner.mutation.inventory_count) {
    fail(`mutation inventory mismatch for ${ownerId}: expected ${owner.mutation.inventory_count}, got ${count}`);
  }

  mkdirSync(outputDir, {recursive: true});
  const runs = mutationRun
    ? owner.mutation.runs.filter((run) => run.id === mutationRun)
    : owner.mutation.runs;
  if (mutationRun && runs.length === 0) fail(`unknown mutation run ${mutationRun} for ${ownerId}`);
  const outcomes = [];
  try {
    for (const run of runs) {
      const runOutput = resolve(outputDir, run.id);
      let exitCode;
      await withDeepwellIntegrationStack(async ({env}) => {
        const result = await runValidationCommand("cargo", [
          "mutants", "--in-place", "--manifest-path", "deepwell/Cargo.toml",
          "-f", owner.mutation.file, "-F", owner.mutation.function,
          "--baseline", "run", "--output", runOutput,
          ...(shard ? ["--shard", shard] : []),
          ...run.cargo_mutants_args,
        ], {env, cwd: root, acceptableExitCodes: [0, 2, 3]});
        exitCode = result.exitCode;
      });
      outcomes.push({
        id: run.id,
        output: repositoryPath(runOutput),
        status: exitCode === 0 ? "completed" : "completed_with_findings",
        exit_code: exitCode,
        ...summarizeMutationRun(runOutput),
      });
    }
  } finally {
    const restoredHash = sha256File(sourcePath);
    if (restoredHash !== originalHash) {
      fail(`mutation runner did not restore ${owner.source.path}; expected ${originalHash}, got ${restoredHash}`);
    }
  }
  writeJson(resolve(outputDir, "mutation-summary.json"), {
    schema: 1,
    owner: ownerId,
    inventory_count: count,
    shard,
    mutation_run: mutationRun,
    outcomes,
  });
}

function verifyMutationEvidence(owner) {
  for (const evidence of owner.mutation?.completed_evidence ?? []) {
    const label = `${owner.id} ${evidence.run} shard ${evidence.shard ?? "all"}`;
    const total = evidence.caught + evidence.missed + evidence.unviable + evidence.timeout;
    if (total !== evidence.mutants) {
      fail(`contradictory mutation evidence for ${label}: outcomes sum to ${total}, expected ${evidence.mutants}`);
    }
    if (owner.status !== "accepted") continue;
    const dispositions = evidence.survivor_dispositions ?? [];
    if (dispositions.length !== evidence.missed) {
      fail(`accepted owner ${label} has ${evidence.missed} survivor(s) but ${dispositions.length} disposition(s)`);
    }
    if (dispositions.some(({classification}) => /pending/iu.test(classification ?? ""))) {
      fail(`accepted owner ${label} has a survivor disposition pending review`);
    }
  }
}

function verifyIdentity(identity, label) {
  if (!identity.path || isAbsolute(identity.path)) fail(`${label} must use a relative path`);
  const path = resolve(root, identity.path);
  const relativePath = relative(root, path);
  if (relativePath === ".." || relativePath.startsWith("../")) fail(`${label} path is outside root`);
  if (!existsSync(path)) fail(`${label} is missing: ${identity.path}`);
  const resolvedLink = relative(root, realpathSync(path));
  if (resolvedLink === ".." || resolvedLink.startsWith("../")) fail(`${label} linked path is outside root`);
  const actual = sha256File(path);
  if (actual !== identity.sha256) fail(`${label} hash is stale: ${identity.path}`);
}

async function verifyPlannedMutationFrontiers(frontiers = []) {
  for (const frontier of frontiers) {
    verifyIdentity({path: frontier.source, sha256: frontier.source_sha256}, `mutation frontier ${frontier.owner_candidate}`);
    if (!new Set(["inventory_only_not_replayed", "replayed_unreviewed_survivors"]).has(frontier.status)) {
      fail(`mutation frontier ${frontier.owner_candidate} claims unsupported status ${frontier.status}`);
    }
    if (frontier.status === "replayed_unreviewed_survivors") {
      const replay = frontier.replay;
      if (!replay) fail(`replayed mutation frontier ${frontier.owner_candidate} has no replay evidence`);
      const total = replay.caught + replay.missed + replay.unviable + replay.timeout;
      if (replay.mutants !== frontier.mutation_count || total !== replay.mutants) {
        fail(`mutation frontier ${frontier.owner_candidate} outcomes sum to ${total}, expected ${frontier.mutation_count}`);
      }
      if (replay.missed === 0) {
        fail(`mutation frontier ${frontier.owner_candidate} has no survivors but is marked unreviewed`);
      }
      const reconciliation = replay.independent_reconciliation;
      if (reconciliation) {
        const missedToCaught = reconciliation.transitions_missed_to_caught;
        const missedToUnviable = reconciliation.transitions_missed_to_unviable ?? 0;
        const unviableToCaught = reconciliation.transitions_unviable_to_caught ?? 0;
        if (reconciliation.distinct_initial_mutants !== frontier.mutation_count ||
            reconciliation.initial_caught + reconciliation.initial_missed +
              (reconciliation.initial_unviable ?? replay.unviable) +
              (reconciliation.initial_timeout ?? replay.timeout) !== frontier.mutation_count ||
            reconciliation.initial_caught + missedToCaught + unviableToCaught !== replay.caught ||
            reconciliation.initial_missed - missedToCaught - missedToUnviable !== replay.missed ||
            (reconciliation.initial_unviable ?? replay.unviable) + missedToUnviable - unviableToCaught !== replay.unviable ||
            reconciliation.final_caught !== replay.caught || reconciliation.final_missed !== replay.missed ||
            !/^[0-9a-f]{64}$/u.test(reconciliation.report_sha256 ?? "")) {
          fail(`mutation frontier ${frontier.owner_candidate} has contradictory independent reconciliation`);
        }
      }
    }
    const {stdout} = await runValidationCommand("cargo", [
      "mutants", "--list", "--manifest-path", "deepwell/Cargo.toml",
      "-f", frontier.source.replace(/^deepwell\//u, ""), "-F", frontier.symbol,
    ], {capture: true, cwd: root});
    const count = stdout.split("\n").filter((line) => line.trim() !== "").length;
    if (count !== frontier.mutation_count) {
      fail(`mutation frontier ${frontier.owner_candidate} inventory changed: ${count} != ${frontier.mutation_count}`);
    }
    const digest = createHash("sha256").update(stdout).digest("hex");
    if (digest !== frontier.mutation_list_sha256) {
      fail(`mutation frontier ${frontier.owner_candidate} list identity changed`);
    }
  }
}

async function verify(outputDir, ownerId) {
  const ledger = loadLedger();
  for (const lockfile of ledger.lockfiles) verifyIdentity(lockfile, "lockfile");
  const owners = ownerId ? ledger.owners.filter((owner) => owner.id === ownerId) : ledger.owners;
  if (ownerId && owners.length === 0) fail(`unknown audit owner: ${ownerId}`);
  if (owners.length === 0) fail("audit ledger has no owners");

  for (const owner of owners) {
    verifyIdentity(owner.source, `source for ${owner.id}`);
    if (!Array.isArray(owner.tests) || owner.tests.length === 0) fail(`owner ${owner.id} has no tests`);
    for (const test of owner.tests) {
      verifyIdentity(test, `test for ${owner.id}`);
      const text = readFileSync(resolve(root, test.path), "utf8");
      for (const anchor of test.anchors ?? []) {
        if (!text.includes(anchor)) fail(`owner ${owner.id} has invalid test anchor ${JSON.stringify(anchor)} in ${test.path}`);
      }
    }
    if (owner.status === "accepted" && owner.acceptance_gaps?.length) {
      fail(`accepted owner ${owner.id} still has unresolved acceptance gaps`);
    }
    verifyMutationEvidence(owner);
    if (owner.mutation) {
      if (owner.mutation.removed) {
        if (owner.status !== "removed_redundant") {
          fail(`removed mutation owner ${owner.id} has contradictory status ${owner.status}`);
        }
        const evidence = owner.mutation.removal_evidence;
        if (!evidence?.classification || !evidence?.differential || !evidence?.validation) {
          fail(`removed mutation owner ${owner.id} lacks reviewed removal evidence`);
        }
        const sourceText = readFileSync(resolve(root, owner.source.path), "utf8");
        if (sourceText.includes(owner.symbol)) {
          fail(`removed owner ${owner.id} still contains symbol ${owner.symbol}`);
        }
      } else {
        if (owner.status === "removed_redundant") {
          fail(`owner ${owner.id} claims removed status but retains a live mutation descriptor`);
        }
        const count = await listedMutationCount(owner);
        if (count !== owner.mutation.inventory_count) {
          fail(`mutation inventory mismatch for ${owner.id}: expected ${owner.mutation.inventory_count}, got ${count}`);
        }
      }
    }
  }

  if (!ownerId) await verifyPlannedMutationFrontiers(ledger.next_mutation_frontier);

  mkdirSync(outputDir, {recursive: true});
  const result = {
    schema: 1,
    verified_at: new Date().toISOString(),
    tool_versions: await toolVersions(),
    owners: owners.map(({id}) => id),
  };
  writeJson(resolve(outputDir, "verification.json"), result);
  process.stdout.write(`Verified ${owners.length} audit owner(s).\n`);
}

async function main() {
  if (!["inventory", "coverage", "mutate", "verify"].includes(commandName)) {
    process.stderr.write("Usage: node scripts/run-test-quality-audit.mjs inventory|coverage|mutate|verify --output-dir <directory> [--owner <id>] [--shard I/N] [--mutation-run <id>]\n");
    process.exitCode = 2;
    return;
  }
  const {outputDir, owner, shard, mutationRun} = parseOptions(process.argv.slice(3));
  if (commandName === "inventory") await inventory(outputDir);
  else if (commandName === "coverage") await coverage(outputDir);
  else if (commandName === "mutate") await mutate(outputDir, owner, shard, mutationRun);
  else await verify(outputDir, owner);
}

main().catch((error) => {
  console.error(error?.stack ?? error?.message ?? String(error));
  process.exitCode = 1;
});
