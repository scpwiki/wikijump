import assert from "node:assert/strict";
import {
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  createHttpFixtureResourceLoader,
  createLocalFixtureResourceLoader,
  materializeFixtureResourceManifest,
} from "../src/resource-materializer.mjs";
import {scanForFixtureLocalResources} from "../src/resource-scanner.mjs";

async function temporaryDirectory(t, prefix) {
  const directory = await mkdtemp(path.join(os.tmpdir(), prefix));
  t.after(() => rm(directory, {recursive: true, force: true}));
  return directory;
}

function scanManifest(sourceText, fixtureSlug = "materializer-fixture") {
  return scanForFixtureLocalResources({
    sourceText,
    fixtureSlug,
    sourcePath: `fixtures/${fixtureSlug}.wikidot.txt`,
  }).manifest;
}

function readableStreamFromChunks(...chunks) {
  return new ReadableStream({
    start(controller) {
      for (const chunk of chunks) {
        controller.enqueue(chunk);
      }
      controller.close();
    },
  });
}

test("materializes only manifest entries and hashes the bytes written", async (t) => {
  const outputRoot = await temporaryDirectory(t, "wikijump-materialize-");
  const manifest = scanManifest([
    "https://scp-wiki.wikidot.com/local--files/test/style.css",
    "https://scp-wiki.wikidot.com/local--files/test/pixel.png",
  ].join("\n"));
  const bytesByUrl = new Map([
    [
      "https://scp-wiki.wikidot.com/local--files/test/style.css",
      Buffer.from("body { color: #123; }\n"),
    ],
    [
      "https://scp-wiki.wikidot.com/local--files/test/pixel.png",
      Buffer.from([0, 1, 2, 255]),
    ],
  ]);
  const requestedUrls = [];

  const materialized = await materializeFixtureResourceManifest({
    manifest,
    outputRoot,
    loadResource: async (entry) => {
      requestedUrls.push(entry.original_url);
      return bytesByUrl.get(entry.original_url);
    },
  });

  assert.deepEqual(requestedUrls, manifest.map((entry) => entry.original_url));
  assert.ok(manifest.every((entry) => entry.sha256 === null));
  assert.equal(materialized.length, 2);

  const css = materialized.find((entry) => entry.filename === "style.css");
  const image = materialized.find((entry) => entry.filename === "pixel.png");
  assert.equal(
    css.sha256,
    "ba52bca62f079df261c85f75c8f60b512fe8f7624dd8eb81749232e1bedf24e7",
  );
  assert.equal(
    image.sha256,
    "3d1f57c984978ef98a18378c8166c1cb8ede02c03eeb6aee7e2f121dfeee3e56",
  );
  assert.equal(
    await readFile(path.join(outputRoot, css.local_target_path), "utf8"),
    "body { color: #123; }\n",
  );
  assert.deepEqual(
    await readFile(path.join(outputRoot, image.local_target_path)),
    Buffer.from([0, 1, 2, 255]),
  );
});

test("copies a binary resource from the same deterministic manifest layout", async (t) => {
  const sourceRoot = await temporaryDirectory(t, "wikijump-copy-source-");
  const outputRoot = await temporaryDirectory(t, "wikijump-copy-output-");
  const [entry] = scanManifest(
    "https://scp-wiki.wikidot.com/local--files/test/audio.mp3",
    "binary-fixture",
  );
  const sourcePath = path.join(sourceRoot, entry.local_target_path);
  await mkdir(path.dirname(sourcePath), {recursive: true});
  await writeFile(sourcePath, Buffer.from("local-copy-bytes\n"));

  const materialized = await materializeFixtureResourceManifest({
    manifest: [entry],
    outputRoot,
    loadResource: createLocalFixtureResourceLoader({sourceRoot}),
  });

  assert.equal(
    materialized[0].sha256,
    "d1186f9c6c10fdbfcc2b17a3c1001c94b37b10834cfb660f75b0911864880c4e",
  );
  assert.deepEqual(
    await readFile(path.join(outputRoot, entry.local_target_path)),
    Buffer.from("local-copy-bytes\n"),
  );
});

test("HTTP loader requests exactly the validated manifest URL without redirects", async () => {
  const [entry] = scanManifest(
    "http://scptestwiki.wikidot.com/local--files/sigma:lol/ecw.png",
    "http-fixture",
  );
  const calls = [];
  const expectedBytes = Buffer.from([5, 4, 3, 2, 1]);
  const loadResource = createHttpFixtureResourceLoader({
    fetchImpl: async (url, options) => {
      calls.push({url, options});
      return {
        ok: true,
        status: 200,
        headers: {get: () => String(expectedBytes.byteLength)},
        body: readableStreamFromChunks(expectedBytes),
      };
    },
    maxResourceBytes: 1024,
    timeoutMs: 1000,
  });

  assert.deepEqual(await loadResource(entry), expectedBytes);
  assert.equal(calls.length, 1);
  assert.equal(calls[0].url, entry.original_url);
  assert.equal(calls[0].options.method, "GET");
  assert.equal(calls[0].options.redirect, "error");
  assert.ok(calls[0].options.signal instanceof AbortSignal);
});

test("HTTP loader rejects non-streaming responses before reading arrayBuffer", async () => {
  const [entry] = scanManifest(
    "https://scp-wiki.wikidot.com/local--files/test/non-streaming.bin",
    "http-non-streaming-fixture",
  );
  let arrayBufferCalls = 0;
  const loadResource = createHttpFixtureResourceLoader({
    fetchImpl: async () => ({
      ok: true,
      status: 200,
      headers: {get: () => null},
      arrayBuffer: async () => {
        arrayBufferCalls += 1;
        return Buffer.alloc(1024 * 1024);
      },
    }),
    maxResourceBytes: 16,
    timeoutMs: 1000,
  });

  await assert.rejects(loadResource(entry), /streaming body/);
  assert.equal(arrayBufferCalls, 0);
});

test("rejects a tampered target path before invoking the loader", async (t) => {
  const outputRoot = await temporaryDirectory(t, "wikijump-traversal-");
  const [entry] = scanManifest(
    "https://scp-wiki.wikidot.com/local--files/test/asset.bin",
  );
  const tampered = {...entry, local_target_path: "../outside.bin"};
  let loaderCalls = 0;

  await assert.rejects(
    materializeFixtureResourceManifest({
      manifest: [tampered],
      outputRoot,
      loadResource: async () => {
        loaderCalls += 1;
        return Buffer.from("must not be loaded");
      },
    }),
    /local_target_path/,
  );
  assert.equal(loaderCalls, 0);
});

test("rejects distinct URL paths that collapse to one filesystem target", async (t) => {
  const outputRoot = await temporaryDirectory(t, "wikijump-collision-");
  const manifest = scanManifest([
    "https://scp-wiki.wikidot.com/local--files/test//asset.png",
    "https://scp-wiki.wikidot.com/local--files/test/asset.png",
  ].join("\n"));
  assert.equal(manifest.length, 2);
  let loaderCalls = 0;

  await assert.rejects(
    materializeFixtureResourceManifest({
      manifest,
      outputRoot,
      loadResource: async () => {
        loaderCalls += 1;
        return Buffer.from("must not be loaded");
      },
    }),
    /resolve to the same file/,
  );
  assert.equal(loaderCalls, 0);
});


test("rejects a symlinked output directory before writing outside the root", async (t) => {
  const outputRoot = await temporaryDirectory(t, "wikijump-symlink-output-");
  const outsideRoot = await temporaryDirectory(t, "wikijump-symlink-outside-");
  await symlink(outsideRoot, path.join(outputRoot, "resources"));
  const [entry] = scanManifest(
    "https://scp-wiki.wikidot.com/local--files/test/asset.bin",
  );

  await assert.rejects(
    materializeFixtureResourceManifest({
      manifest: [entry],
      outputRoot,
      loadResource: async () => Buffer.from("must stay inside root"),
    }),
    /not a real directory/,
  );
  assert.deepEqual(await readdir(outsideRoot), []);
});


test("HTTP loader aborts a streamed body that exceeds the configured limit", async () => {
  const [entry] = scanManifest(
    "https://scp-wiki.wikidot.com/local--files/test/oversized.bin",
    "http-limit-fixture",
  );
  let requestSignal;
  const loadResource = createHttpFixtureResourceLoader({
    fetchImpl: async (_url, options) => {
      requestSignal = options.signal;
      return {
        ok: true,
        status: 200,
        headers: {get: () => null},
        body: readableStreamFromChunks(
          Uint8Array.from([1, 2, 3]),
          Uint8Array.from([4, 5, 6]),
        ),
      };
    },
    maxResourceBytes: 4,
    timeoutMs: 1000,
  });

  await assert.rejects(loadResource(entry), /body exceeds maxResourceBytes/);
  assert.equal(requestSignal.aborted, true);
});
