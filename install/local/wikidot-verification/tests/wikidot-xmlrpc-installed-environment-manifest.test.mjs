import assert from "node:assert/strict";
import fs from "node:fs/promises";
import test from "node:test";

import {stableStringify} from "../src/corpus-import-manifest.mjs";
import {
  buildWikidotXmlrpcInstalledEnvironmentManifest,
  hashWikidotXmlrpcInstalledEnvironmentManifest,
  openWikidotXmlrpcInstalledEnvironmentManifest,
  parseWikidotXmlrpcInstalledEnvironmentManifest,
  putWikidotXmlrpcInstalledEnvironmentManifest,
  serializeWikidotXmlrpcInstalledEnvironmentManifest,
  WIKIDOT_XMLRPC_INSTALLED_ENVIRONMENT_MANIFEST_SCHEMA,
} from "../src/wikidot-xmlrpc-installed-environment-manifest.mjs";
import {assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest} from "../src/wikidot-xmlrpc-python-environment.mjs";
import {openReferenceObjectStore} from "../src/reference-object-store.mjs";
import {
  installedEnvironmentFile,
  installedEnvironmentManifest,
  installedEnvironmentStoreFixture,
  pythonEnvironmentForManifest,
} from "./support/wikidot-xmlrpc-installed-environment-fixture.mjs";

test("schema and canonical bytes bind the manifest identity", async () => {
  const schema = JSON.parse(
    await fs.readFile(
      new URL(
        "../schemas/wikidot-xmlrpc-installed-environment-manifest-v1.schema.json",
        import.meta.url,
      ),
    ),
  );
  const value = installedEnvironmentManifest();
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
  const value = installedEnvironmentManifest();
  const state = await installedEnvironmentStoreFixture(t);
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
      pythonEnvironmentForManifest(value),
      value,
    );
  assert.equal(Object.isFrozen(matched), true);
  assert.deepEqual(matched.descriptor, pythonEnvironmentForManifest(value));
  assert.deepEqual(matched.manifest, value);
});

test("each role and file identity field changes the manifest digest", () => {
  const baseline = installedEnvironmentManifest();
  const baselineHash = hashWikidotXmlrpcInstalledEnvironmentManifest(baseline);
  const changed = [
    installedEnvironmentManifest({
      files: [
        installedEnvironmentFile("bin/python", { executable: true, sha256: "e".repeat(64) }),
        installedEnvironmentFile("lib/site-packages/example.py", { sha256: "c".repeat(64) }),
        installedEnvironmentFile("pyvenv.cfg", { sha256: "d".repeat(64) }),
      ],
    }),
    installedEnvironmentManifest({
      files: [
        installedEnvironmentFile("bin/python", { executable: true, sha256: "b".repeat(64) }),
        installedEnvironmentFile("lib/site-packages/example.py", {
          bytes: 5,
          sha256: "c".repeat(64),
        }),
        installedEnvironmentFile("pyvenv.cfg", { sha256: "d".repeat(64) }),
      ],
    }),
    installedEnvironmentManifest({
      files: [
        installedEnvironmentFile("bin/python", { executable: true, sha256: "b".repeat(64) }),
        installedEnvironmentFile("lib/site-packages/example.py", {
          executable: true,
          sha256: "c".repeat(64),
        }),
        installedEnvironmentFile("pyvenv.cfg", { sha256: "d".repeat(64) }),
      ],
    }),
    installedEnvironmentManifest({ additionalFiles: [installedEnvironmentFile("lib/site-packages/extra.py")] }),
    installedEnvironmentManifest({
      files: [
        installedEnvironmentFile("runtime/python", { executable: true, sha256: "b".repeat(64) }),
        installedEnvironmentFile("lib/site-packages/example.py", { sha256: "c".repeat(64) }),
        installedEnvironmentFile("pyvenv.cfg", { sha256: "d".repeat(64) }),
      ],
      pythonExecutablePath: "runtime/python",
    }),
    installedEnvironmentManifest({
      files: [
        installedEnvironmentFile("bin/python", { executable: true, sha256: "b".repeat(64) }),
        installedEnvironmentFile("lib/site-packages/example.py", { sha256: "c".repeat(64) }),
        installedEnvironmentFile("pyvenv.cfg", { sha256: "e".repeat(64) }),
      ],
    }),
    installedEnvironmentManifest({ pythonVersion: "3.13.14" }),
  ];
  for (const value of changed) {
    assert.notEqual(
      hashWikidotXmlrpcInstalledEnvironmentManifest(value),
      baselineHash,
    );
  }

  const alternateRoles = [
    installedEnvironmentFile("bin/python", { executable: true, sha256: "b".repeat(64) }),
    installedEnvironmentFile("lib/site-packages/example.py", { sha256: "c".repeat(64) }),
    installedEnvironmentFile("pyvenv-alt.cfg", { sha256: "d".repeat(64) }),
    installedEnvironmentFile("pyvenv.cfg", { sha256: "d".repeat(64) }),
    installedEnvironmentFile("runtime/python", { executable: true, sha256: "b".repeat(64) }),
  ];
  const firstRoles = installedEnvironmentManifest({ files: alternateRoles });
  const otherPythonRole = installedEnvironmentManifest({
    files: alternateRoles,
    pythonExecutablePath: "runtime/python",
  });
  const otherConfigRole = installedEnvironmentManifest({
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
  const value = installedEnvironmentManifest();
  const environment = pythonEnvironmentForManifest(value);
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
  const baseline = installedEnvironmentManifest();
  const expanded = installedEnvironmentManifest({
    additionalFiles: [
      installedEnvironmentFile("lib/site-packages/another.py", { sha256: "e".repeat(64) }),
    ],
  });
  assert.throws(
    () =>
      assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest(
        pythonEnvironmentForManifest(baseline),
        expanded,
      ),
    /does not match/u,
  );

  const alternateFiles = [
    installedEnvironmentFile("bin/python", { executable: true, sha256: "b".repeat(64) }),
    installedEnvironmentFile("config/pyvenv.cfg", { sha256: "f".repeat(64) }),
    installedEnvironmentFile("lib/site-packages/example.py", { sha256: "c".repeat(64) }),
    installedEnvironmentFile("pyvenv.cfg", { sha256: "d".repeat(64) }),
    installedEnvironmentFile("runtime/python", { executable: true, sha256: "e".repeat(64) }),
  ];
  const initialRoles = installedEnvironmentManifest({ files: alternateFiles });
  const alternateExecutable = installedEnvironmentManifest({
    files: alternateFiles,
    pythonExecutablePath: "runtime/python",
  });
  const alternateConfig = installedEnvironmentManifest({
    files: alternateFiles,
    venvConfigPath: "config/pyvenv.cfg",
  });
  assert.throws(
    () =>
      assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest(
        pythonEnvironmentForManifest(alternateExecutable, {
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
        pythonEnvironmentForManifest(alternateConfig, {
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
  const value = installedEnvironmentManifest();
  const marker = "sentinel-python-environment-marker";
  const hostile = new Proxy(pythonEnvironmentForManifest(value), {
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
  const valid = installedEnvironmentManifest();
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
        files: [installedEnvironmentFile("bin/python", { executable: false }), installedEnvironmentFile("pyvenv.cfg")],
        pythonExecutablePath: "bin/python",
        pythonImplementation: "cpython",
        pythonVersion: "3.13.13",
        venvConfigPath: "pyvenv.cfg",
      }),
    () =>
      buildWikidotXmlrpcInstalledEnvironmentManifest({
        files: [
          installedEnvironmentFile("bin/python", { executable: true }),
          installedEnvironmentFile("pyvenv.cfg", { executable: true }),
        ],
        pythonExecutablePath: "bin/python",
        pythonImplementation: "cpython",
        pythonVersion: "3.13.13",
        venvConfigPath: "pyvenv.cfg",
      }),
    () =>
      buildWikidotXmlrpcInstalledEnvironmentManifest({
        files: [installedEnvironmentFile("bin/python", { executable: true }), installedEnvironmentFile("pyvenv.cfg")],
        pythonExecutablePath: "bin/python",
        pythonImplementation: "cpython",
        pythonVersion: "3.13.13",
        venvConfigPath: "bin/python",
      }),
    () =>
      installedEnvironmentManifest({
        additionalFiles: [installedEnvironmentFile("lib/\tbad")],
      }),
    () =>
      installedEnvironmentManifest({
        additionalFiles: [installedEnvironmentFile("😀".repeat(1025))],
      }),
    () =>
      buildWikidotXmlrpcInstalledEnvironmentManifest({
        files: [
          installedEnvironmentFile("bin/python", { executable: true }),
          installedEnvironmentFile("pyvenv.cfg", { bytes: -0 }),
        ],
        pythonExecutablePath: "bin/python",
        pythonImplementation: "cpython",
        pythonVersion: "3.13.13",
        venvConfigPath: "pyvenv.cfg",
      }),
    () =>
      buildWikidotXmlrpcInstalledEnvironmentManifest({
        files: [installedEnvironmentFile("../escape", { executable: true }), installedEnvironmentFile("pyvenv.cfg")],
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
