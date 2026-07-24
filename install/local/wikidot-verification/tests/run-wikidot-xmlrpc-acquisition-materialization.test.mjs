import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  materializeExactCoordinator,
  WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS,
} from "../scripts/run-wikidot-xmlrpc-acquisition.mjs";
import {verifyMaterializedDescriptor} from "../scripts/run-wikidot-xmlrpc-acquisition-materialized.mjs";
import {WIKIDOT_XMLRPC_CANONICAL_COORDINATOR_SOURCE_PATHS} from "../src/wikidot-xmlrpc-acquisition-runner.mjs";
import {
  git,
  LOCAL_IMPORT,
  REPOSITORY_ROOT,
  runPrivateCoordinator,
  runnerOptions,
} from "./support/run-wikidot-xmlrpc-acquisition-fixture.mjs";

test("coordinator closure declares every transitive local import", async () => {
  assert.deepEqual(
    WIKIDOT_XMLRPC_CANONICAL_COORDINATOR_SOURCE_PATHS,
    WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS,
  );
  const declared = new Set(WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS);
  const pending = [...declared];
  const visited = new Set();
  while (pending.length !== 0) {
    const sourcePath = pending.pop();
    if (visited.has(sourcePath)) continue;
    visited.add(sourcePath);
    const filePath = path.join(REPOSITORY_ROOT, sourcePath);
    const source = await fs.readFile(filePath, "utf8");
    for (const match of source.matchAll(LOCAL_IMPORT)) {
      if (!match[1].startsWith(".")) continue;
      const importedPath = path.relative(
        REPOSITORY_ROOT,
        path.resolve(path.dirname(filePath), match[1]),
      );
      const normalized = importedPath.split(path.sep).join("/");
      assert.equal(
        declared.has(normalized),
        true,
        `${sourcePath} imports undeclared ${normalized}`,
      );
      pending.push(normalized);
    }
  }
  assert.equal(visited.size, declared.size);
});

test("bootstrap and materialized entrypoint have no pre-verification local import", async () => {
  const source = await fs.readFile(
    path.join(
      REPOSITORY_ROOT,
      "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition.mjs",
    ),
    "utf8",
  );
  const specifiers = [...source.matchAll(LOCAL_IMPORT)].map(
    (match) => match[1],
  );
  assert.equal(specifiers.length > 0, true);
  assert.equal(
    specifiers.every((specifier) => specifier.startsWith("node:")),
    true,
  );
  assert.equal(/\b(?:createRequire|eval)\s*\(/u.test(source), false);
  const entrypoint = await fs.readFile(
    path.join(
      REPOSITORY_ROOT,
      "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition-materialized.mjs",
    ),
    "utf8",
  );
  const entrypointSpecifiers = [...entrypoint.matchAll(LOCAL_IMPORT)].map(
    (match) => match[1],
  );
  assert.equal(
    entrypointSpecifiers.every((specifier) => specifier.startsWith("node:")),
    true,
  );
  assert.equal(
    entrypoint.indexOf(
      "await verifyDescriptorAgainstGit(descriptor, identity);",
    ) < entrypoint.indexOf("const bootstrap = await import("),
    true,
  );
  assert.equal(
    entrypoint.indexOf(
      "await verifyDescriptorAgainstGit(descriptor, identity);",
    ) < entrypoint.indexOf("const coordinator = await import("),
    true,
  );
  assert.match(entrypoint, /createReadStream\(null, \{/u);
  assert.doesNotMatch(entrypoint, /readFile\("\/proc\/self\/fd\/3"\)/u);
  const coordinator = await fs.readFile(
    path.join(
      REPOSITORY_ROOT,
      "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs",
    ),
    "utf8",
  );
  assert.match(coordinator, /MATERIALIZED_DESCRIPTOR_FD = 4/u);
  assert.match(coordinator, /createReadStream\(null, \{/u);
  assert.doesNotMatch(coordinator, /readFile\("\/proc\/self\/fd\/\$\{/u);
});

test("bootstrap materializes the complete exact coordinator closure privately", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "xmlrpc-bootstrap-"));
  const checkout = path.join(root, "checkout");
  const capsules = path.join(root, "capsules");
  await fs.mkdir(checkout);
  await fs.mkdir(capsules, { mode: 0o700 });
  await fs.chmod(capsules, 0o700);
  t.after(() => fs.rm(root, { force: true, recursive: true }));
  for (const sourcePath of WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS) {
    const destination = path.join(checkout, sourcePath);
    await fs.mkdir(path.dirname(destination), { recursive: true });
    await fs.copyFile(path.join(REPOSITORY_ROOT, sourcePath), destination);
  }
  git(checkout, ["init", "--initial-branch=main"]);
  git(checkout, ["add", "."]);
  git(checkout, ["commit", "-m", "coordinator"]);
  const commit = git(checkout, ["rev-parse", "HEAD"]);
  const tree = git(checkout, ["rev-parse", "HEAD^{tree}"]);
  await fs.writeFile(
    path.join(
      checkout,
      "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs",
    ),
    "throw new Error('mutable checkout must not execute');\n",
  );
  const coordinator = await materializeExactCoordinator({
    capsuleParent: capsules,
    wikijumpCommit: commit,
    wikijumpGitDirectory: path.join(checkout, ".git"),
    wikijumpTree: tree,
  });
  t.after(() =>
    fs.rm(coordinator.root, { force: true, maxRetries: 2, recursive: true }),
  );
  assert.equal((await fs.stat(coordinator.root)).mode & 0o777, 0o700);
  for (const sourcePath of WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS) {
    const [expected, actual, stat] = await Promise.all([
      fs.readFile(path.join(REPOSITORY_ROOT, sourcePath)),
      fs.readFile(path.join(coordinator.root, sourcePath)),
      fs.stat(path.join(coordinator.root, sourcePath)),
    ]);
    assert.deepEqual(actual, expected);
    assert.equal(stat.mode & 0o777, 0o400);
  }
  const descriptor = {
    coordinator_path:
      "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs",
    entrypoint_path:
      "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition-materialized.mjs",
    files: coordinator.files,
    materialization_root: coordinator.root,
    schema: "wikijump_full_parity.wikidot_xmlrpc_materialized_launch.v1",
    wikijump_commit: commit,
    wikijump_tree: tree,
  };
  const identityArguments = [
    "--wikijump-commit",
    commit,
    "--wikijump-git-dir",
    path.join(checkout, ".git"),
    "--wikijump-tree",
    tree,
  ];
  await assert.doesNotReject(
    verifyMaterializedDescriptor(
      descriptor,
      identityArguments,
      coordinator.entrypoint,
    ),
  );
  const forged = {
    ...descriptor,
    files: descriptor.files.map((file, index, files) =>
      index === 0 ? { ...file, blob_oid: files[1].blob_oid } : file,
    ),
  };
  await assert.rejects(
    verifyMaterializedDescriptor(
      forged,
      identityArguments,
      coordinator.entrypoint,
    ),
    /descriptor_git_mismatch/u,
  );
  const mutableCoordinator = path.join(
    coordinator.root,
    "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs",
  );
  const mutableDependency = path.join(
    coordinator.root,
    "install/local/wikidot-verification/src/atomic-no-replace.mjs",
  );
  await fs.chmod(mutableDependency, 0o600);
  await fs.appendFile(mutableDependency, "// mutable omitted dependency\n");
  await fs.chmod(mutableDependency, 0o400);
  const incompleteDescriptor = {
    ...descriptor,
    files: descriptor.files.filter(
      (file) =>
        file.path !==
        "install/local/wikidot-verification/src/atomic-no-replace.mjs",
    ),
  };
  const omittedDependency = await runPrivateCoordinator(
    mutableCoordinator,
    runnerOptions({
      "wikijump-commit": commit,
      "wikijump-git-dir": path.join(checkout, ".git"),
      "wikijump-tree": tree,
    }),
    incompleteDescriptor,
  );
  assert.equal(omittedDependency.signal, null);
  assert.equal(omittedDependency.code, 7, omittedDependency.stderr);
  assert.match(
    omittedDependency.stdout,
    /materialized_launch_descriptor_invalid/u,
  );
  await fs.chmod(mutableCoordinator, 0o600);
  await fs.appendFile(mutableCoordinator, "// mutable private copy\n");
  await fs.chmod(mutableCoordinator, 0o400);
  const privateCopy = await runPrivateCoordinator(
    mutableCoordinator,
    runnerOptions({
      "wikijump-commit": commit,
      "wikijump-git-dir": path.join(checkout, ".git"),
      "wikijump-tree": tree,
    }),
    descriptor,
  );
  assert.equal(privateCopy.signal, null);
  assert.equal(privateCopy.code, 7, privateCopy.stderr);
  assert.match(privateCopy.stdout, /materialized_launch_private_file_invalid/u);
});
