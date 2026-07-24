import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import {
  buildWikidotXmlrpcInstalledEnvironmentManifest,
  hashWikidotXmlrpcInstalledEnvironmentManifest,
} from "../../src/wikidot-xmlrpc-installed-environment-manifest.mjs";
import {buildWikidotXmlrpcPythonEnvironment} from "../../src/wikidot-xmlrpc-python-environment.mjs";
import {initializeReferenceObjectStore} from "../../src/reference-object-store.mjs";

export function installedEnvironmentFile(pathname, overrides = {}) {
  return {
    bytes: 4,
    executable: false,
    path: pathname,
    sha256: "a".repeat(64),
    ...overrides,
  };
}

export function installedEnvironmentManifest({additionalFiles = [], ...overrides} = {}) {
  return buildWikidotXmlrpcInstalledEnvironmentManifest({
    files: [
      installedEnvironmentFile("bin/python", {executable: true, sha256: "b".repeat(64)}),
      installedEnvironmentFile("lib/site-packages/example.py", {sha256: "c".repeat(64)}),
      installedEnvironmentFile("pyvenv.cfg", {sha256: "d".repeat(64)}),
      ...additionalFiles,
    ],
    pythonExecutablePath: "bin/python",
    pythonImplementation: "cpython",
    pythonVersion: "3.13.13",
    venvConfigPath: "pyvenv.cfg",
    ...overrides,
  });
}

export function pythonEnvironmentForManifest(value, overrides = {}) {
  const files = new Map(value.files.map((entry) => [entry.path, entry]));
  return buildWikidotXmlrpcPythonEnvironment({
    dependencyEnvironmentSha256: hashWikidotXmlrpcInstalledEnvironmentManifest(value),
    dependencyLockBlobOid: "1".repeat(40),
    dependencyLockFileSha256: "2".repeat(64),
    dependencyRecipeBlobOid: "3".repeat(40),
    dependencyRecipeSha256: "4".repeat(64),
    pythonExecutableSha256: files.get(value.python_executable_path).sha256,
    pythonImplementation: value.python_implementation,
    pythonVersion: value.python_version,
    venvConfigSha256: files.get(value.venv_config_path).sha256,
    workerBlobOid: "5".repeat(40),
    workerFileSha256: "6".repeat(64),
    workerRepositoryCommit: "7".repeat(40),
    workerRepositoryTree: "8".repeat(40),
    ...overrides,
  });
}

export async function installedEnvironmentStoreFixture(t) {
  const parent = await fs.mkdtemp(path.join(os.tmpdir(), "xmlrpc-installed-environment-"));
  const state = {
    root: path.join(parent, "store"),
    store: undefined,
  };
  state.store = await initializeReferenceObjectStore(state.root);
  t.after(async () => {
    await state.store.close();
    await fs.rm(parent, {force: true, recursive: true});
  });
  return state;
}
