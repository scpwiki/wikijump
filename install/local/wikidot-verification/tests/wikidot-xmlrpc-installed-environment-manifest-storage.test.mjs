import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import {
  buildWikidotXmlrpcInstalledEnvironmentManifest,
  openWikidotXmlrpcInstalledEnvironmentManifest,
  putWikidotXmlrpcInstalledEnvironmentManifest,
} from "../src/wikidot-xmlrpc-installed-environment-manifest.mjs";
import {referenceObjectRelativePath} from "../src/reference-object-store.mjs";
import {
  installedEnvironmentFile,
  installedEnvironmentManifest,
  installedEnvironmentStoreFixture,
} from "./support/wikidot-xmlrpc-installed-environment-fixture.mjs";

test("manifest rejects a file that would replace a required role directory", () => {
  assert.throws(
    () => installedEnvironmentManifest({additionalFiles: [installedEnvironmentFile("bin")]}),
    /invalid file tree/u,
  );
});

test("builder rejects an unencodable manifest before allocating its full serialization", () => {
  const suffix = "a".repeat(4088);
  const files = [
    installedEnvironmentFile("bin/python", {executable: true}),
    installedEnvironmentFile("pyvenv.cfg"),
    ...Array.from({length: 4200}, (_, index) =>
      installedEnvironmentFile(`${String(index).padStart(5, "0")}-${suffix}`),
    ),
  ];
  assert.throws(
    () => buildWikidotXmlrpcInstalledEnvironmentManifest({
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
  const state = await installedEnvironmentStoreFixture(t);
  const stored = await putWikidotXmlrpcInstalledEnvironmentManifest(
    state.store,
    installedEnvironmentManifest(),
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
      error.message === "installed environment manifest object cannot be read" &&
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
    openWikidotXmlrpcInstalledEnvironmentManifest(state.store, malformed.object),
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
