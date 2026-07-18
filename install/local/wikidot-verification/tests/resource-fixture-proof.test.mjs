import assert from "node:assert/strict";
import {createServer} from "node:http";
import {createHash} from "node:crypto";
import {mkdtemp, readFile, rm} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {materializeFixtureResourceManifest} from "../src/resource-materializer.mjs";
import {scanForFixtureLocalResources} from "../src/resource-scanner.mjs";
import {substituteFixtureResourceUrls} from "../src/resource-substituter.mjs";

const FIXTURE_SOURCE = new URL(
  "../../../../deepwell/seeder/fragment-scp-8980-1.ftml",
  import.meta.url,
);

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function temporaryDirectory(t, prefix) {
  const directory = await mkdtemp(path.join(os.tmpdir(), prefix));
  t.after(() => rm(directory, {recursive: true, force: true}));
  return directory;
}

function selectedSCP8980Manifest(manifest) {
  const selectedUrls = new Set([
    "https://scp-wiki.wikidot.com/local--files/scp-8980/departuremono.css",
    "https://scp-wiki.wikidot.com/local--files/scp-8980/femalescientist.png",
  ]);
  return manifest.filter((entry) => selectedUrls.has(entry.original_url));
}

async function serveDirectory(t, {root, prefix}) {
  const resolvedRoot = path.resolve(root);
  const server = createServer(async (request, response) => {
    try {
      const requestUrl = new URL(request.url ?? "/", "http://127.0.0.1");
      if (!requestUrl.pathname.startsWith(prefix)) {
        response.writeHead(404).end("not found");
        return;
      }

      const relativePath = decodeURIComponent(requestUrl.pathname.slice(prefix.length));
      const candidate = path.resolve(resolvedRoot, relativePath.replace(/^\/+/, ""));
      const relative = path.relative(resolvedRoot, candidate);
      // A resolved candidate escapes the static root when path.relative() returns
      // exactly "..", starts with "../" on this platform, or is still absolute.
      // Treat those cases as directory traversal attempts before reading bytes.
      if (relative === ".." || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
        response.writeHead(403).end("forbidden");
        return;
      }

      const bytes = await readFile(candidate);
      response.writeHead(200, {"content-length": String(bytes.byteLength)});
      response.end(bytes);
    } catch (error) {
      response.writeHead(error?.code === "ENOENT" ? 404 : 500).end(String(error?.message ?? error));
    }
  });

  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  t.after(() => new Promise((resolve) => server.close(resolve)));

  const address = server.address();
  return `http://127.0.0.1:${address.port}${prefix}`;
}

test("SCP-8980 selected fixture resources materialize, substitute, and resolve locally", async (t) => {
  const sourceText = await readFile(FIXTURE_SOURCE, "utf8");
  const {manifest, out_of_scope: outOfScope} = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "fragment-scp-8980-1",
    sourcePath: "deepwell/seeder/fragment-scp-8980-1.ftml",
  });

  assert.equal(outOfScope.length, 0);
  assert.ok(manifest.length >= 2);

  const selected = selectedSCP8980Manifest(manifest);
  assert.deepEqual(
    selected.map((entry) => entry.kind_guess).sort(),
    ["css", "image"],
  );

  const bytesByUrl = new Map([
    [
      "https://scp-wiki.wikidot.com/local--files/scp-8980/departuremono.css",
      Buffer.from("@font-face { font-family: DepartureMono; src: local('Departure Mono'); }\n"),
    ],
    [
      "https://scp-wiki.wikidot.com/local--files/scp-8980/femalescientist.png",
      Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x66, 0x69, 0x78, 0x74, 0x75, 0x72, 0x65]),
    ],
  ]);

  const outputRoot = await temporaryDirectory(t, "wikijump-scp8980-resource-proof-");
  const materialized = await materializeFixtureResourceManifest({
    manifest: selected,
    outputRoot,
    loadResource: async (entry) => bytesByUrl.get(entry.original_url),
  });

  assert.equal(materialized.length, 2);
  for (const entry of materialized) {
    const expectedBytes = bytesByUrl.get(entry.original_url);
    assert.equal(entry.sha256, sha256(expectedBytes));
    assert.deepEqual(
      await readFile(path.join(outputRoot, entry.local_target_path)),
      expectedBytes,
    );
  }

  const substituted = substituteFixtureResourceUrls({
    sourceText,
    manifest: materialized,
    localUrlPrefix: "/fixture-assets",
  });

  assert.equal(substituted.substitutions, 2);
  for (const entry of materialized) {
    assert.ok(!substituted.text.includes(entry.original_url));
    assert.ok(substituted.text.includes(`/fixture-assets/${entry.local_target_path}`));
  }

  const localServerPrefix = await serveDirectory(t, {
    root: outputRoot,
    prefix: "/fixture-assets/",
  });

  for (const entry of materialized) {
    const response = await fetch(`${localServerPrefix}${entry.local_target_path}`);
    assert.equal(response.status, 200);
    const bytes = Buffer.from(await response.arrayBuffer());
    assert.equal(sha256(bytes), entry.sha256);
  }

  const skipped = manifest.filter(
    (entry) => !materialized.some((selectedEntry) => selectedEntry.original_url === entry.original_url),
  );
  assert.ok(skipped.length > 0);
  assert.ok(
    skipped.some((entry) => substituted.text.includes(entry.original_url)),
    "non-materialized SCP-8980 resources should remain remote and visible for the gap ledger",
  );
});
