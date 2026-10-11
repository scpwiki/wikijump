#!/usr/bin/env node
// Deliberate audit-snapshot refresh. Regenerates every source and input identity from the committed
// bytes of one exact commit (via a temporary detached worktree), verifies the result against Git
// objects, and only then rewrites docs/development/test-quality-audit.json. Review state is preserved.
// Usage: node scripts/refresh-test-quality-snapshot.mjs --commit <rev> [--check]
import {spawnSync} from "node:child_process";
import {mkdtemp, readFile, rm, writeFile} from "node:fs/promises";
import {tmpdir} from "node:os";
import path from "node:path";
import {fileURLToPath} from "node:url";
import {buildAuditInventory} from "../install/local/wikidot-verification/src/test-quality-inventory.mjs";
import {
  buildSnapshotLedger, commitTree, resolveCommit, verifyPinnedSnapshot,
} from "./lib/test-quality-snapshot.mjs";

const root = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const ledgerPath = path.join(root, "docs/development/test-quality-audit.json");

function parseArguments(values) {
  const options = {check: false, commit: null};
  for (let index = 0; index < values.length; index += 1) {
    if (values[index] === "--check") options.check = true;
    else if (values[index] === "--commit" && values[index + 1]) options.commit = values[++index];
    else throw new Error(`unknown/incomplete option: ${values[index]}`);
  }
  if (!options.commit) throw new Error("--commit <rev> is required; pass the audited commit explicitly");
  return options;
}

const git = (cwd, args) => spawnSync("git", args, {cwd, encoding: "utf8"});

async function main() {
  const options = parseArguments(process.argv.slice(2));
  const commit = resolveCommit(root, options.commit);
  if (!commit) throw new Error(`cannot resolve audited commit: ${options.commit}`);
  const tree = commitTree(root, commit);
  const ledger = JSON.parse(await readFile(ledgerPath, "utf8"));

  const temp = await mkdtemp(path.join(tmpdir(), "wikijump-test-quality-snapshot-"));
  const checkout = path.join(temp, "checkout");
  try {
    const added = git(root, ["worktree", "add", "--detach", checkout, commit]);
    if (added.status !== 0) throw new Error(`cannot create audit worktree: ${added.stderr.trim()}`);
    const inventory = await buildAuditInventory(checkout);
    const next = buildSnapshotLedger(ledger, inventory, {commit, tree});
    const summary = verifyPinnedSnapshot(root, next);
    const changed = next.records.filter((record, index) => ledger.records
      .find((old) => old.path === record.path)?.sha256 !== record.sha256).length;
    console.log(JSON.stringify({
      commit: summary.commit, tree: summary.tree, records: summary.records, inputs: summary.inputs,
      records_changed_or_added: changed, written: !options.check,
    }));
    if (!options.check) await writeFile(ledgerPath, `${JSON.stringify(next, null, 2)}\n`);
  } finally {
    git(root, ["worktree", "remove", "--force", checkout]);
    git(root, ["worktree", "prune"]);
    await rm(temp, {recursive: true, force: true});
  }
}

try {
  await main();
} catch (error) {
  console.error(`Test quality snapshot refresh failed: ${error.message}`);
  process.exitCode = 1;
}
