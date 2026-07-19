import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { stableStringify } from "../src/corpus-import-manifest.mjs";
import {
  buildWikidotXmlrpcInstalledEnvironmentManifest,
  hashWikidotXmlrpcInstalledEnvironmentManifest,
  openWikidotXmlrpcInstalledEnvironmentManifest,
  parseWikidotXmlrpcInstalledEnvironmentManifest,
  putWikidotXmlrpcInstalledEnvironmentManifest,
  serializeWikidotXmlrpcInstalledEnvironmentManifest,
  WIKIDOT_XMLRPC_INSTALLED_ENVIRONMENT_MANIFEST_SCHEMA,
} from "../src/wikidot-xmlrpc-installed-environment-manifest.mjs";
import {
  assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest,
  buildWikidotXmlrpcPythonEnvironment,
} from "../src/wikidot-xmlrpc-python-environment.mjs";
import {
  initializeReferenceObjectStore,
  openReferenceObjectStore,
  referenceObjectRelativePath,
} from "../src/reference-object-store.mjs";

function file(pathname, overrides = {}) {
  return {
    bytes: 4,
    executable: false,
    path: pathname,
    sha256: "a".repeat(64),
    ...overrides,
  };
}

function manifest({ additionalFiles = [], ...overrides } = {}) {
  return buildWikidotXmlrpcInstalledEnvironmentManifest({
    files: [
      file("bin/python", { executable: true, sha256: "b".repeat(64) }),
      file("lib/site-packages/example.py", { sha256: "c".repeat(64) }),
      file("pyvenv.cfg", { sha256: "d".repeat(64) }),
      ...additionalFiles,
    ],
    pythonExecutablePath: "bin/python",
    pythonImplementation: "cpython",
    pythonVersion: "3.13.13",
    venvConfigPath: "pyvenv.cfg",
    ...overrides,
  });
}

function environmentFor(value, overrides = {}) {
  const files = new Map(value.files.map((entry) => [entry.path, entry]));
  return buildWikidotXmlrpcPythonEnvironment({
    dependencyEnvironmentSha256:
      hashWikidotXmlrpcInstalledEnvironmentManifest(value),
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

async function fixture(t) {
  const parent = await fs.mkdtemp(
    path.join(os.tmpdir(), "xmlrpc-installed-environment-"),
  );
  const state = {
    root: path.join(parent, "store"),
    store: undefined,
  };
  state.store = await initializeReferenceObjectStore(state.root);
  t.after(async () => {
    await state.store.close();
    await fs.rm(parent, { force: true, recursive: true });
  });
  return state;
}

test("schema and canonical bytes bind the manifest identity", async () => {
  const schema = JSON.parse(
    await fs.readFile(
      new URL(
        "../schemas/wikidot-xmlrpc-installed-environment-manifest-v1.schema.json",
        import.meta.url,
      ),
    ),
  );
  const value = manifest();
  const bytes = serializeWikidotXmlrpcInstalledEnvironmentManifest(value);

  assert.deepEqual(Object.keys(value).sort(), schema.required);
  assert.deepEqual(
    Object.keys(value).sort(),
    Object.keys(schema.properties).sort(),
  );
  assert.equal(
    value.schema,
    WIKIDOT_XMLRPC_INSTALLED_ENVIRONMENT_MANIFEST_SCHEMA,
  );
  assert.equal(schema.properties.scope.const, value.scope);
  assert.equal(value.scope, "declared_application_dependencies");
  assert.equal(
    new RegExp(schema.$defs.relativePath.pattern, "u").test("lib/\tbad"),
    false,
  );
  assert.equal(
    new RegExp(schema.$defs.relativePath.pattern, "u").test("lib//bad"),
    false,
  );
  assert.equal(
    new RegExp(schema.$defs.relativePath.pattern, "u").test("lib/bad/"),
    false,
  );
  assert.match(schema.$defs.relativePath.description, /UTF-8-byte limit/u);
  assert.match(
    schema["x-runtime-validation"].join("\n"),
    /ancestor of another file path/u,
  );
  assert.match(schema.properties.files.description, /regular files/u);
  assert.equal(Object.isFrozen(value), true);
  assert.equal(Object.isFrozen(value.files), true);
  assert.deepEqual(
    value.files.map((entry) => entry.path),
    ["bin/python", "lib/site-packages/example.py", "pyvenv.cfg"],
  );
  assert.deepEqual(
    parseWikidotXmlrpcInstalledEnvironmentManifest(bytes),
    value,
  );
  assert.equal(bytes.includes(Buffer.from("/host/path", "utf8")), false);
});

test("CAS identity and Python environment matching bind the manifest", async (t) => {
  const value = manifest();
  const state = await fixture(t);
  const first = await putWikidotXmlrpcInstalledEnvironmentManifest(
    state.store,
    value,
  );
  const repeated = await putWikidotXmlrpcInstalledEnvironmentManifest(
    state.store,
    value,
  );
  assert.equal(first.disposition, "created");
  assert.equal(repeated.disposition, "exists");
  assert.deepEqual(repeated.object, first.object);
  assert.equal(
    first.object.sha256,
    hashWikidotXmlrpcInstalledEnvironmentManifest(value),
  );
  assert.equal(
    first.object.bytes,
    serializeWikidotXmlrpcInstalledEnvironmentManifest(value).byteLength,
  );
  await state.store.close();
  state.store = await openReferenceObjectStore(state.root);
  assert.deepEqual(
    (
      await openWikidotXmlrpcInstalledEnvironmentManifest(
        state.store,
        first.object,
      )
    ).descriptor,
    value,
  );
  const matched =
    assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest(
      environmentFor(value),
      value,
    );
  assert.equal(Object.isFrozen(matched), true);
  assert.deepEqual(matched.descriptor, environmentFor(value));
  assert.deepEqual(matched.manifest, value);
});

test("each role and file identity field changes the manifest digest", () => {
  const baseline = manifest();
  const baselineHash = hashWikidotXmlrpcInstalledEnvironmentManifest(baseline);
  const changed = [
    manifest({
      files: [
        file("bin/python", { executable: true, sha256: "e".repeat(64) }),
        file("lib/site-packages/example.py", { sha256: "c".repeat(64) }),
        file("pyvenv.cfg", { sha256: "d".repeat(64) }),
      ],
    }),
    manifest({
      files: [
        file("bin/python", { executable: true, sha256: "b".repeat(64) }),
        file("lib/site-packages/example.py", {
          bytes: 5,
          sha256: "c".repeat(64),
        }),
        file("pyvenv.cfg", { sha256: "d".repeat(64) }),
      ],
    }),
    manifest({
      files: [
        file("bin/python", { executable: true, sha256: "b".repeat(64) }),
        file("lib/site-packages/example.py", {
          executable: true,
          sha256: "c".repeat(64),
        }),
        file("pyvenv.cfg", { sha256: "d".repeat(64) }),
      ],
    }),
    manifest({ additionalFiles: [file("lib/site-packages/extra.py")] }),
    manifest({
      files: [
        file("runtime/python", { executable: true, sha256: "b".repeat(64) }),
        file("lib/site-packages/example.py", { sha256: "c".repeat(64) }),
        file("pyvenv.cfg", { sha256: "d".repeat(64) }),
      ],
      pythonExecutablePath: "runtime/python",
    }),
    manifest({
      files: [
        file("bin/python", { executable: true, sha256: "b".repeat(64) }),
        file("lib/site-packages/example.py", { sha256: "c".repeat(64) }),
        file("pyvenv.cfg", { sha256: "e".repeat(64) }),
      ],
    }),
    manifest({ pythonVersion: "3.13.14" }),
  ];
  for (const value of changed) {
    assert.notEqual(
      hashWikidotXmlrpcInstalledEnvironmentManifest(value),
      baselineHash,
    );
  }

  const alternateRoles = [
    file("bin/python", { executable: true, sha256: "b".repeat(64) }),
    file("lib/site-packages/example.py", { sha256: "c".repeat(64) }),
    file("pyvenv-alt.cfg", { sha256: "d".repeat(64) }),
    file("pyvenv.cfg", { sha256: "d".repeat(64) }),
    file("runtime/python", { executable: true, sha256: "b".repeat(64) }),
  ];
  const firstRoles = manifest({ files: alternateRoles });
  const otherPythonRole = manifest({
    files: alternateRoles,
    pythonExecutablePath: "runtime/python",
  });
  const otherConfigRole = manifest({
    files: alternateRoles,
    venvConfigPath: "pyvenv-alt.cfg",
  });
  assert.notEqual(
    hashWikidotXmlrpcInstalledEnvironmentManifest(firstRoles),
    hashWikidotXmlrpcInstalledEnvironmentManifest(otherPythonRole),
  );
  assert.notEqual(
    hashWikidotXmlrpcInstalledEnvironmentManifest(firstRoles),
    hashWikidotXmlrpcInstalledEnvironmentManifest(otherConfigRole),
  );
});

test("Python environment matcher rejects every environment identity disagreement", () => {
  const value = manifest();
  const environment = environmentFor(value);
  for (const changed of [
    { dependency_environment_sha256: "0".repeat(64) },
    { python_executable_sha256: "1".repeat(64) },
    { python_version: "3.13.14" },
    { venv_config_sha256: "2".repeat(64) },
  ]) {
    assert.throws(
      () =>
        assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest(
          { ...environment, ...changed },
          value,
        ),
      /does not match/u,
    );
  }
});

test("Python environment matcher binds the full manifest and selected role files", () => {
  const baseline = manifest();
  const expanded = manifest({
    additionalFiles: [
      file("lib/site-packages/another.py", { sha256: "e".repeat(64) }),
    ],
  });
  assert.throws(
    () =>
      assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest(
        environmentFor(baseline),
        expanded,
      ),
    /does not match/u,
  );

  const alternateFiles = [
    file("bin/python", { executable: true, sha256: "b".repeat(64) }),
    file("config/pyvenv.cfg", { sha256: "f".repeat(64) }),
    file("lib/site-packages/example.py", { sha256: "c".repeat(64) }),
    file("pyvenv.cfg", { sha256: "d".repeat(64) }),
    file("runtime/python", { executable: true, sha256: "e".repeat(64) }),
  ];
  const initialRoles = manifest({ files: alternateFiles });
  const alternateExecutable = manifest({
    files: alternateFiles,
    pythonExecutablePath: "runtime/python",
  });
  const alternateConfig = manifest({
    files: alternateFiles,
    venvConfigPath: "config/pyvenv.cfg",
  });
  assert.throws(
    () =>
      assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest(
        environmentFor(alternateExecutable, {
          pythonExecutableSha256: initialRoles.files.find(
            (entry) => entry.path === initialRoles.python_executable_path,
          ).sha256,
        }),
        alternateExecutable,
      ),
    /does not match/u,
  );
  assert.throws(
    () =>
      assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest(
        environmentFor(alternateConfig, {
          venvConfigSha256: initialRoles.files.find(
            (entry) => entry.path === initialRoles.venv_config_path,
          ).sha256,
        }),
        alternateConfig,
      ),
    /does not match/u,
  );
});

test("Python environment matcher fails closed for hostile operands", () => {
  const value = manifest();
  const marker = "sentinel-python-environment-marker";
  const hostile = new Proxy(environmentFor(value), {
    ownKeys() {
      throw new Error(marker);
    },
  });
  assert.throws(
    () =>
      assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest(
        hostile,
        value,
      ),
    (error) =>
      error.message === "XML-RPC Python environment is invalid" &&
      !error.message.includes(marker),
  );
});

test("manifest rejects invalid roles, hostile values, and noncanonical bytes", () => {
  const marker = "sentinel-installed-environment-marker";
  const valid = manifest();
  const accessor = {
    files: [],
    pythonExecutablePath: "bin/python",
    pythonImplementation: "cpython",
    pythonVersion: "3.13.13",
    venvConfigPath: "pyvenv.cfg",
  };
  Object.defineProperty(accessor, "files", {
    enumerable: true,
    get() {
      throw new Error(marker);
    },
  });
  const reversed = {
    ...valid,
    files: [...valid.files].reverse(),
  };

  for (const call of [
    () => buildWikidotXmlrpcInstalledEnvironmentManifest(accessor),
    () =>
      buildWikidotXmlrpcInstalledEnvironmentManifest({
        files: [file("bin/python", { executable: false }), file("pyvenv.cfg")],
        pythonExecutablePath: "bin/python",
        pythonImplementation: "cpython",
        pythonVersion: "3.13.13",
        venvConfigPath: "pyvenv.cfg",
      }),
    () =>
      buildWikidotXmlrpcInstalledEnvironmentManifest({
        files: [
          file("bin/python", { executable: true }),
          file("pyvenv.cfg", { executable: true }),
        ],
        pythonExecutablePath: "bin/python",
        pythonImplementation: "cpython",
        pythonVersion: "3.13.13",
        venvConfigPath: "pyvenv.cfg",
      }),
    () =>
      buildWikidotXmlrpcInstalledEnvironmentManifest({
        files: [file("bin/python", { executable: true }), file("pyvenv.cfg")],
        pythonExecutablePath: "bin/python",
        pythonImplementation: "cpython",
        pythonVersion: "3.13.13",
        venvConfigPath: "bin/python",
      }),
    () =>
      manifest({
        additionalFiles: [file("lib/\tbad")],
      }),
    () =>
      manifest({
        additionalFiles: [file("😀".repeat(1025))],
      }),
    () =>
      buildWikidotXmlrpcInstalledEnvironmentManifest({
        files: [
          file("bin/python", { executable: true }),
          file("pyvenv.cfg", { bytes: -0 }),
        ],
        pythonExecutablePath: "bin/python",
        pythonImplementation: "cpython",
        pythonVersion: "3.13.13",
        venvConfigPath: "pyvenv.cfg",
      }),
    () =>
      buildWikidotXmlrpcInstalledEnvironmentManifest({
        files: [file("../escape", { executable: true }), file("pyvenv.cfg")],
        pythonExecutablePath: "../escape",
        pythonImplementation: "cpython",
        pythonVersion: "3.13.13",
        venvConfigPath: "pyvenv.cfg",
      }),
    () =>
      parseWikidotXmlrpcInstalledEnvironmentManifest(
        Buffer.from(`${stableStringify(reversed)}\n`),
      ),
    () => parseWikidotXmlrpcInstalledEnvironmentManifest(Buffer.from("{}\r\n")),
    () =>
      parseWikidotXmlrpcInstalledEnvironmentManifest(Buffer.from([0xff, 0x0a])),
  ]) {
    assert.throws(call, (error) => !error.message.includes(marker));
  }
});

test("manifest rejects a file that would replace a required role directory", () => {
  assert.throws(
    () => manifest({ additionalFiles: [file("bin")] }),
    /invalid file tree/u,
  );
});

test("builder rejects an unencodable manifest before allocating its full serialization", () => {
  const suffix = "a".repeat(4088);
  const files = [
    file("bin/python", { executable: true }),
    file("pyvenv.cfg"),
    ...Array.from({ length: 4200 }, (_, index) =>
      file(`${String(index).padStart(5, "0")}-${suffix}`),
    ),
  ];
  assert.throws(
    () =>
      buildWikidotXmlrpcInstalledEnvironmentManifest({
        files,
        pythonExecutablePath: "bin/python",
        pythonImplementation: "cpython",
        pythonVersion: "3.13.13",
        venvConfigPath: "pyvenv.cfg",
      }),
    /byte limit/u,
  );
});

test("opening fails closed on invalid immutable CAS content and references", async (t) => {
  const state = await fixture(t);
  const stored = await putWikidotXmlrpcInstalledEnvironmentManifest(
    state.store,
    manifest(),
  );
  const objectPath = path.join(
    state.root,
    ...referenceObjectRelativePath(stored.object.sha256).split("/"),
  );
  await assert.rejects(
    openWikidotXmlrpcInstalledEnvironmentManifest(state.store, {
      algorithm: "sha256",
      bytes: 1,
      sha256: "0".repeat(64),
    }),
    (error) =>
      error.message ===
        "installed environment manifest object cannot be read" &&
      !error.message.includes("/proc/"),
  );
  await fs.chmod(objectPath, 0o600);
  await fs.writeFile(objectPath, Buffer.alloc(stored.object.bytes, 0x20));
  await fs.chmod(objectPath, 0o400);
  await assert.rejects(
    openWikidotXmlrpcInstalledEnvironmentManifest(state.store, stored.object),
    /object cannot be read/u,
  );
  const malformed = await state.store.putBytes(Buffer.from("{}\n"));
  await assert.rejects(
    openWikidotXmlrpcInstalledEnvironmentManifest(
      state.store,
      malformed.object,
    ),
    /object is not canonical/u,
  );
  const marker = "sentinel-reference-marker";
  const proxy = new Proxy(stored.object, {
    ownKeys() {
      throw new Error(marker);
    },
  });
  await assert.rejects(
    openWikidotXmlrpcInstalledEnvironmentManifest(state.store, proxy),
    (error) => !error.message.includes(marker),
  );
});
