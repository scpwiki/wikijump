import assert from "node:assert/strict";
import fs from "node:fs/promises";
import test from "node:test";

import { stableStringify } from "../src/corpus-import-manifest.mjs";
import {
  buildWikidotXmlrpcInstalledEnvironmentManifest,
  hashWikidotXmlrpcInstalledEnvironmentManifest,
  parseWikidotXmlrpcInstalledEnvironmentManifest,
  serializeWikidotXmlrpcInstalledEnvironmentManifest,
  WIKIDOT_XMLRPC_INSTALLED_ENVIRONMENT_MANIFEST_SCHEMA,
} from "../src/wikidot-xmlrpc-installed-environment-manifest.mjs";

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
