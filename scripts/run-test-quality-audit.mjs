#!/usr/bin/env node
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {fileURLToPath} from "node:url";
import {buildAuditInventory, loadAuditProofs, validateAuditLedger} from "../install/local/wikidot-verification/src/test-quality-inventory.mjs";
import {runAuditCommand, withAuditSignals} from "../install/local/wikidot-verification/src/audit-command.mjs";
import {coverageInstrumentationExclusions, runNodeCoverage, runRustCoverage} from "../install/local/wikidot-verification/src/test-quality-coverage.mjs";
import {runNodeMutations, runRustMutations} from "../install/local/wikidot-verification/src/test-quality-mutations.mjs";
import {snapshotStaleness} from "./lib/test-quality-snapshot.mjs";

const root = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const ledgerPath = path.join(root, "docs/development/test-quality-audit.json");

function argumentsFor(values) {
  const [mode, ...args] = values;
  if (mode === "--help" || !mode) return {help: true};
  if (!["inventory", "coverage", "mutate", "verify"].includes(mode)) throw new Error("unknown audit mode");
  const options = {mode, allowIncomplete: false};
  for (let index = 0; index < args.length; index += 1) {
    if (args[index] === "--allow-incomplete") options.allowIncomplete = true;
    else if (["--output-dir", "--owner"].includes(args[index]) && args[index + 1]) {
      options[args[index] === "--owner" ? "owner" : "output"] = args[++index];
    } else throw new Error(`unknown/incomplete option: ${args[index]}`);
  }
  if (!options.output) throw new Error("--output-dir is required");
  options.output = path.resolve(options.output);
  if (options.output === root || options.output.startsWith(`${root}${path.sep}`)) {
    throw new Error("audit reports/targets must live outside the source checkout");
  }
  return options;
}

async function withSnapshotCheckout(commit, action) {
  if (!/^[0-9a-f]{40}$/u.test(commit ?? "")) throw new Error("test-quality ledger has no pinned snapshot commit");
  const temp = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-test-quality-verify-"));
  const checkout = path.join(temp, "checkout");
  try {
    await runAuditCommand("git", ["worktree", "add", "--detach", checkout, commit], {cwd: root, capture: true});
    return await action(checkout);
  } finally {
    await runAuditCommand("git", ["worktree", "remove", "--force", checkout], {cwd: root, capture: true}).catch(() => {});
    await fs.rm(temp, {recursive: true, force: true});
  }
}

async function run(options, signal) {
  await fs.mkdir(path.dirname(options.output), {recursive: true, mode: 0o700});
  await fs.mkdir(options.output, {mode: 0o700});
  const inventory = await buildAuditInventory(root);
  const write = (file, value) => fs.writeFile(path.join(options.output, file), `${JSON.stringify(value, null, 2)}\n`, {mode: 0o600});
  await write("inventory.json", inventory);
  if (options.mode === "inventory") {
    console.log(JSON.stringify({digest: inventory.digest, files: inventory.entries.length, production: inventory.entries.filter((entry) => entry.role === "production").length}));
    return;
  }
  const ledger = JSON.parse(await fs.readFile(ledgerPath, "utf8"));
  if (options.mode === "verify") {
    // The ledger audits a pinned commit; judging it against the working tree
    // would fail on every later production edit. Working-tree drift is
    // reported, not treated as a broken ledger.
    const verification = await withSnapshotCheckout(ledger.snapshot?.commit, async (checkout) => {
      const snapshotInventory = await buildAuditInventory(checkout);
      const allowIncomplete = options.allowIncomplete;
      const proofs = await loadAuditProofs(root, ledger, snapshotInventory, {allowStale: true});
      return validateAuditLedger(ledger, snapshotInventory, {proofs, allowIncomplete});
    });
    verification.working_tree_drift = snapshotStaleness(root, ledger).length;
    await write("verification.json", verification);
    console.log(JSON.stringify(verification));
    return;
  }
  const allowIncomplete = options.mode !== "verify" || options.allowIncomplete;
  const proofs = await loadAuditProofs(root, ledger, inventory, {allowStale: allowIncomplete});
  validateAuditLedger(ledger, inventory, {proofs, allowIncomplete});
  const owner = options.owner ? ledger.owners.find((entry) => entry.id === options.owner) : null;
  if (options.owner && !owner) throw new Error(`unknown executable owner: ${options.owner}`);
  const metadata = {
    inventory_digest: inventory.digest,
    source_revision: (await runAuditCommand("git", ["rev-parse", "HEAD"], {cwd: root, capture: true, signal})).stdout.trim(),
    node: process.version, owner: options.owner ?? null, status: "running",
  };
  await write("run.json", metadata);
  try {
    if (options.mode === "mutate") {
      if (!owner) throw new Error("mutation testing requires --owner (broad sweeps are prohibited)");
      if (!["rust", "node"].includes(owner.kind)) throw new Error("mutation requires a Rust or Node executable owner");
      const mutation = owner.kind === "rust" ? runRustMutations : runNodeMutations;
      await mutation(root, owner, path.join(options.output, owner.id), signal);
    } else if (owner) {
      if (owner.kind !== "node") throw new Error("Rust combined coverage uses coverage without --owner");
      await runNodeCoverage(root, owner.test_files, path.join(options.output, owner.id), {packageRoot: owner.package_root, signal});
    } else {
      await runRustCoverage(root, path.join(options.output, "rust"), signal);
      for (const packageRoot of ["framerail", "install/local/wikidot-verification"]) {
        const tests = inventory.entrypoints.filter((file) => file.startsWith(`${packageRoot}/tests/`));
        const excludedTestFiles = coverageInstrumentationExclusions(packageRoot, tests);
        await runNodeCoverage(root, tests, path.join(options.output, path.basename(packageRoot)), {packageRoot, signal, excludedTestFiles});
      }
    }
    metadata.status = "complete";
  } catch (error) {
    metadata.status = "failed";
    metadata.error = error.message;
    throw error;
  } finally {
    await write("run.json", metadata);
  }
}

const options = argumentsFor(process.argv.slice(2));
if (options.help) {
  console.log("Usage: node scripts/run-test-quality-audit.mjs inventory|coverage|mutate|verify --output-dir DIRECTORY [--owner ID] [--allow-incomplete]");
} else {
  await withAuditSignals(async (signal) => {
    if (["coverage", "mutate"].includes(options.mode) && process.env.WIKIJUMP_TEST_NETWORK_GUARD_ACTIVE !== "1") {
      await runAuditCommand(path.join(root, "scripts/run-test-no-external-network.sh"), [process.execPath, fileURLToPath(import.meta.url), ...process.argv.slice(2)], {cwd: root, signal});
    } else await run(options, signal);
  }).catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
