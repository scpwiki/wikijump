import assert from "node:assert/strict";
import fs from "node:fs/promises";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  REDIRECT_VERDICT_SCHEMA,
  runRedirectRuntimeRepro,
  validateRedirectInputs,
} from "../src/redirect-runtime-repro.mjs";
import {parseArgs} from "../scripts/validate-redirect-runtime.mjs";

const HASH_A = "a".repeat(64);
const HASH_B = "b".repeat(64);

function inventory(slugs = ["about", "external"] ) {
  return {rows: slugs.map((slug) => ({fixture_id: `EN:${slug}`, family: "EN", slug}))};
}

function authority() {
  return [
    {fullname: "about", status: 301, location: "/target", expected_destination: "target", url: "https://scp-wiki.wikidot.com/about"},
    {fullname: "external", status: 301, location: "https://scp-wiki.wikidot.com/forum/start", expected_destination: "https://scp-wiki.wikidot.com/forum/start", url: "https://scp-wiki.wikidot.com/external"},
  ];
}

function corpus() {
  return [
    {fullname: "about", destination: "target", source_sha256: HASH_A, meta_sha256: HASH_B},
    {fullname: "external", destination: "https://scp-wiki.wikidot.com/forum/start", source_sha256: HASH_B, meta_sha256: HASH_A},
  ];
}

function runtimeIdentity() {
  return {
    schema: "wikijump_full_parity.local_browser_runtime_identity.v1",
    wikijump_sha: "a".repeat(40),
    ftml_sha: "b".repeat(40),
    deepwell_binary_or_image_sha256: HASH_A,
    framerail_assets_sha256: HASH_B,
    rustc_vv: "rustc 1.0\nbinary: rustc\ncommit-hash: x\ncommit-date: x\nhost: x\nrelease: x\nLLVM version: x",
    profile: "release",
    features: [],
    render_run_id: "1",
  };
}

async function writeInputs(root) {
  const paths = {
    inventoryPath: path.join(root, "inventory.json"),
    authorityPath: path.join(root, "authority.json"),
    corpusRedirectsPath: path.join(root, "corpus.json"),
    runtimeIdentityPath: path.join(root, "runtime.json"),
    outputPath: path.join(root, "verdict.json"),
  };
  await Promise.all([
    fs.writeFile(paths.inventoryPath, JSON.stringify(inventory())),
    fs.writeFile(paths.authorityPath, JSON.stringify(authority())),
    fs.writeFile(paths.corpusRedirectsPath, JSON.stringify(corpus())),
    fs.writeFile(paths.runtimeIdentityPath, JSON.stringify(runtimeIdentity())),
  ]);
  return paths;
}

async function fixtureServer() {
  let followed = 0;
  const server = http.createServer((request, response) => {
    if (request.url === "/about") {
      response.writeHead(301, {location: "/target", "content-type": "text/html; charset=utf-8"});
      response.end("redirect");
    } else if (request.url === "/external") {
      response.writeHead(301, {location: "https://scp-wiki.wikidot.com/forum/start"});
      response.end();
    } else {
      followed += 1;
      response.writeHead(200);
      response.end("followed");
    }
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  return {
    server,
    port: server.address().port,
    followed: () => followed,
    close: () => new Promise((resolve, reject) => server.close((error) => error ? reject(error) : resolve())),
  };
}

test("input closure requires exact authority, corpus, and inventory agreement", () => {
  const rows = validateRedirectInputs({inventoryDocument: inventory(), authorityDocument: authority(), corpusDocument: corpus()});
  assert.deepEqual(rows.map((row) => row.fixture_id), ["EN:about", "EN:external"]);
  assert.throws(() => validateRedirectInputs({inventoryDocument: inventory(), authorityDocument: [...authority(), authority()[0]], corpusDocument: corpus()}), /duplicate redirect authority/);
  assert.throws(() => validateRedirectInputs({inventoryDocument: inventory(), authorityDocument: authority(), corpusDocument: corpus().slice(1)}), /sets are not exactly equal/);
});

test("CLI binds every authority and candidate input", () => {
  const args = parseArgs(["--inventory", "i", "--authority", "a", "--corpus-redirects", "c", "--runtime-identity", "r", "--local-base", "https://scp-wiki.wikijump.localhost", "--resolved-address", "127.0.0.2", "--output", "o", "--site-id", "6000006", "--workers", "2", "--ignore-https-errors"]);
  assert.equal(args.workers, 2);
  assert.equal(args.siteId, "6000006");
  assert.equal(args.ignoreHttpsErrors, true);
  assert.throws(() => parseArgs(["--inventory", "i"]), /--authority is required/);
});

test("two-pass runtime verification does not follow redirects", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "redirect-repro-"));
  const fixture = await fixtureServer();
  try {
    const paths = await writeInputs(root);
    const verdict = await runRedirectRuntimeRepro({
      ...paths,
      localBase: `http://scp-wiki.wikijump.localhost:${fixture.port}`,
      resolvedAddress: "127.0.0.1",
      timeoutMs: 2_000,
      workers: 2,
      ignoreHttpsErrors: false,
    });
    assert.equal(verdict.schema, REDIRECT_VERDICT_SCHEMA);
    assert.equal(verdict.status, "pass");
    assert.equal(verdict.expected_count, 2);
    assert.equal(verdict.rows.every((row) => row.observations.length === 2 && row.reproducible), true);
    assert.equal(fixture.followed(), 0);
    assert.equal(JSON.parse(await fs.readFile(paths.outputPath, "utf8")).status, "pass");
  } finally {
    await fixture.close();
  }
});

test("runtime mismatches and non-loopback resolution fail closed", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "redirect-repro-fail-"));
  const paths = await writeInputs(root);
  await assert.rejects(() => runRedirectRuntimeRepro({...paths, localBase: "https://scp-wiki.wikijump.localhost", resolvedAddress: "192.0.2.1", timeoutMs: 100, workers: 1, ignoreHttpsErrors: true}), /loopback/);
  await assert.rejects(() => runRedirectRuntimeRepro({...paths, localBase: "https://scp-wiki.wikijump.localhost", resolvedAddress: "127.0.0.1", siteId: "0", timeoutMs: 100, workers: 1, ignoreHttpsErrors: true}), /site ID/);
  await assert.rejects(() => runRedirectRuntimeRepro({...paths, outputPath: paths.authorityPath, localBase: "https://scp-wiki.wikijump.localhost", resolvedAddress: "127.0.0.1", timeoutMs: 100, workers: 1, ignoreHttpsErrors: true}), /must not overwrite an input/);
  let calls = 0;
  const verdict = await runRedirectRuntimeRepro({...paths, localBase: "https://scp-wiki.wikijump.localhost", resolvedAddress: "127.0.0.1", timeoutMs: 100, workers: 1, ignoreHttpsErrors: true, requester: async ({row}) => {
    calls += 1;
    return {status: row.slug === "about" ? 302 : 301, location: row.location, location_count: 1, content_type: null, body_bytes: 0, body_sha256: HASH_A};
  }});
  assert.equal(calls, 4);
  assert.equal(verdict.status, "fail");
  assert.deepEqual(verdict.failed_fixtures, ["EN:about"]);
});
