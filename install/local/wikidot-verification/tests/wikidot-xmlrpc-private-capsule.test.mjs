import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  isWikidotXmlrpcPrivateCapsule,
  materializeWikidotXmlrpcPrivateCapsule,
  prepareWikidotXmlrpcRuntime,
  prepareWikidotXmlrpcWorkerSource,
  WikidotXmlrpcPrivateCapsuleError,
  WIKIDOT_XMLRPC_WORKER_SOURCE_PATHS,
} from "../src/wikidot-xmlrpc-private-capsule.mjs";

const GIT_EXECUTABLE = "/usr/bin/git";
const GIT_ENVIRONMENT = Object.freeze({
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_OPTIONAL_LOCKS: "0",
  GIT_PAGER: "cat",
  GIT_TERMINAL_PROMPT: "0",
  LANG: "C",
  LC_ALL: "C",
  PATH: "/usr/bin:/bin",
});
const PRIVATE_DIRECTORY_MODE = 0o700;

function git(directory, args, options = {}) {
  return execFileSync(GIT_EXECUTABLE, args, {
    cwd: directory,
    encoding: "utf8",
    env: { ...GIT_ENVIRONMENT, ...options.env },
    stdio: ["ignore", "pipe", "pipe"],
  }).trim();
}

async function makePrivateDirectory(directory) {
  await fs.mkdir(directory, { mode: PRIVATE_DIRECTORY_MODE, recursive: true });
  await fs.chmod(directory, PRIVATE_DIRECTORY_MODE);
}

async function writeRuntimeFixture(root) {
  await makePrivateDirectory(root);
  await makePrivateDirectory(path.join(root, "bin"));
  await fs.writeFile(
    path.join(root, "bin", "python"),
    `#!/bin/sh
set -eu
if [ "$1" != "-I" ] || [ "$2" != "-B" ] || [ "$3" != "-c" ]; then
  exit 90
fi
case "$4" in
  *"importlib.import_module(name)"*) ;;
  *) exit 91 ;;
esac
worker="$5"
root=$(dirname "$(dirname "$(dirname "$worker")")")
printf '%s\\n' "{\\"base_prefix\\":\\"$root/runtime\\",\\"executable\\":\\"$root/runtime/bin/python\\",\\"modules\\":{\\"httpx\\":\\"$root/runtime/site-packages/httpx.py\\",\\"scp_wiki_wikidot.bounded_xmlrpc\\":\\"$root/application/scp_wiki_wikidot/bounded_xmlrpc.py\\",\\"scp_wiki_wikidot.client\\":\\"$root/application/scp_wiki_wikidot/client.py\\",\\"scp_wiki_wikidot.config\\":\\"$root/application/scp_wiki_wikidot/config.py\\",\\"scp_wiki_wikidot.transport\\":\\"$root/application/scp_wiki_wikidot/transport.py\\",\\"tenacity\\":\\"$root/runtime/site-packages/tenacity.py\\",\\"wikidot\\":\\"$root/runtime/site-packages/wikidot.py\\",\\"wikidot_paths\\":\\"$root/application/scripts/wikidot_paths.py\\"},\\"prefix\\":\\"$root/runtime\\",\\"stdlib\\":\\"$root/runtime/lib/python3.13\\",\\"sys_path\\":[\\"$root/runtime/lib/python3.13\\",\\"$root/application\\"],\\"version\\":[3,13,13]}"
`,
    { mode: 0o500 },
  );
  await fs.chmod(path.join(root, "bin", "python"), 0o500);
  await fs.writeFile(
    path.join(root, "pyvenv.cfg"),
    "include-system-site-packages = false\nversion = 3.13.13\n",
    { mode: 0o400 },
  );
  await fs.chmod(path.join(root, "pyvenv.cfg"), 0o400);
}

async function writeSourceFixture(repository) {
  await fs.mkdir(repository);
  git("/", ["init", "--initial-branch=main", repository]);
  for (const sourcePath of WIKIDOT_XMLRPC_WORKER_SOURCE_PATHS) {
    const filePath = path.join(repository, ...sourcePath.split("/"));
    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, `${sourcePath}\n`);
  }
  git(repository, ["add", "."]);
  git(repository, ["commit", "-m", "fixture"], {
    env: {
      GIT_AUTHOR_DATE: "2000-01-01T00:00:00Z",
      GIT_AUTHOR_EMAIL: "oracle@example.invalid",
      GIT_AUTHOR_NAME: "Oracle",
      GIT_COMMITTER_DATE: "2000-01-01T00:00:00Z",
      GIT_COMMITTER_EMAIL: "oracle@example.invalid",
      GIT_COMMITTER_NAME: "Oracle",
    },
  });
  return Object.freeze({
    commitOid: git(repository, ["rev-parse", "HEAD"]),
    gitDirectory: path.join(repository, ".git"),
    treeOid: git(repository, ["rev-parse", "HEAD^{tree}"]),
  });
}

async function fixture(t) {
  const parent = await fs.mkdtemp(path.join(os.tmpdir(), "xmlrpc-capsule-"));
  const runtimeRoot = path.join(parent, "runtime-source");
  const capsuleParent = path.join(parent, "capsules");
  const repository = path.join(parent, "source");
  await writeRuntimeFixture(runtimeRoot);
  await makePrivateDirectory(capsuleParent);
  const source = await writeSourceFixture(repository);
  t.after(() => fs.rm(parent, { force: true, recursive: true }));
  return Object.freeze({ capsuleParent, runtimeRoot, source });
}

function capsuleError(code) {
  return (error) =>
    error instanceof WikidotXmlrpcPrivateCapsuleError && error.code === code;
}

test("private capsule materializes the fixed source closure and validates its import probe", async (t) => {
  const state = await fixture(t);
  const runtime = await prepareWikidotXmlrpcRuntime({
    pythonExecutablePath: "bin/python",
    pythonVersion: "3.13.13",
    root: state.runtimeRoot,
    venvConfigPath: "pyvenv.cfg",
  });
  const source = await prepareWikidotXmlrpcWorkerSource(state.source);
  assert.deepEqual(
    source.files.map((file) => file.path),
    WIKIDOT_XMLRPC_WORKER_SOURCE_PATHS,
  );
  await assert.rejects(
    materializeWikidotXmlrpcPrivateCapsule({
      capsuleParent: state.capsuleParent,
      runtime,
      source: Object.freeze({ ...source }),
    }),
    capsuleError("source_capability_invalid"),
  );

  const capsule = await materializeWikidotXmlrpcPrivateCapsule({
    capsuleParent: state.capsuleParent,
    runtime,
    source,
  });
  assert.equal(isWikidotXmlrpcPrivateCapsule(capsule), true);
  await capsule.dispose();
  assert.deepEqual(await fs.readdir(state.capsuleParent), []);
});

test("private capsule rejects symlinked runtime and capsule roots", async (t) => {
  const state = await fixture(t);
  const runtimeLink = `${state.runtimeRoot}-link`;
  const capsuleLink = `${state.capsuleParent}-link`;
  await fs.symlink(state.runtimeRoot, runtimeLink);
  await fs.symlink(state.capsuleParent, capsuleLink);
  await assert.rejects(
    prepareWikidotXmlrpcRuntime({
      pythonExecutablePath: "bin/python",
      pythonVersion: "3.13.13",
      root: runtimeLink,
      venvConfigPath: "pyvenv.cfg",
    }),
    capsuleError("runtime_root_invalid"),
  );

  const runtime = await prepareWikidotXmlrpcRuntime({
    pythonExecutablePath: "bin/python",
    pythonVersion: "3.13.13",
    root: state.runtimeRoot,
    venvConfigPath: "pyvenv.cfg",
  });
  const source = await prepareWikidotXmlrpcWorkerSource(state.source);
  await assert.rejects(
    materializeWikidotXmlrpcPrivateCapsule({
      capsuleParent: capsuleLink,
      runtime,
      source,
    }),
    capsuleError("capsule_parent_invalid"),
  );
});
