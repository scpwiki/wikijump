import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import fs from "node:fs/promises";
import { createServer } from "node:http";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const scriptPath = path.resolve(__dirname, "../scripts/preview-source.mjs");

async function runPreview(args) {
  return execFileAsync(process.execPath, [scriptPath, ...args]);
}

async function assertPreviewFails(args, messagePattern) {
  await assert.rejects(runPreview(args), (error) => {
    assert.match(error.stderr, messagePattern);
    return true;
  });
}

test("preview-source writes deterministic failure diagnostics when RPC is unavailable", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-preview-source-"));
  const sourceDir = path.join(root, "pages", "scp-001");
  const outputDir = path.join(root, "out");
  const sourcePath = path.join(sourceDir, "source.wikidot.txt");

  await fs.mkdir(sourceDir, { recursive: true });
  await fs.writeFile(
    sourcePath,
    [
      "[[include fragment:card]]",
      "[[image local--files/scp-001/example.png]]",
      "Preview fixture body"
    ].join("\n")
  );

  await runPreview([
    "--source",
    sourcePath,
    "--output-dir",
    outputDir,
    "--rpc-url",
    "http://127.0.0.1:1/jsonrpc",
    "--json"
  ]);

  const result = JSON.parse(await fs.readFile(path.join(outputDir, "preview-result.json"), "utf8"));

  assert.equal(result.source.path, sourcePath);
  assert.equal(result.source.manifestMatched, false);
  assert.equal(result.request.previewSlug, "preview-scp-001");
  assert.equal(result.diagnostics.status, "failed-import");
  assert.equal(result.wikijump.action, "failed");
  assert.deepEqual(result.dependencies.includes, ["include:fragment:card"]);
  assert.deepEqual(result.assets.references, ["local--files/scp-001/example.png"]);
  assert.equal(result.html.bytes, 0);
});

test("preview-source persists tags when creating a page", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-preview-source-tags-"));
  const sourceDir = path.join(root, "pages", "scp-002");
  const outputDir = path.join(root, "out");
  const sourcePath = path.join(sourceDir, "source.wikidot.txt");
  const manifestPath = path.join(root, "manifest.tsv");
  const calls = [];

  await fs.mkdir(sourceDir, { recursive: true });
  await fs.writeFile(sourcePath, "Preview fixture body\n");
  await fs.writeFile(
    manifestPath,
    [
      "source_path\tslug\ttitle\ttags\tdependency_hints\tasset_paths",
      `${sourcePath}\tscp-002\tSCP-002\tscp|test|_hidden\t\t`
    ].join("\n")
  );

  const server = createServer((request, response) => {
    let body = "";
    request.setEncoding("utf8");
    request.on("data", (chunk) => {
      body += chunk;
    });
    request.on("end", () => {
      const payload = JSON.parse(body);
      calls.push(payload);
      const reply = (result) => {
        response.setHeader("content-type", "application/json");
        response.end(JSON.stringify({ jsonrpc: "2.0", id: payload.id, result }));
      };
      const fail = (message) => {
        response.setHeader("content-type", "application/json");
        response.end(JSON.stringify({ jsonrpc: "2.0", id: payload.id, error: { message } }));
      };

      if (payload.method === "ping") reply(true);
      else if (payload.method === "site_get") reply({ site_id: 123 });
      else if (payload.method === "login") reply({ session_token: "session" });
      else if (payload.method === "page_get" && !calls.some((call) => call.method === "page_create")) fail("PageMissing");
      else if (payload.method === "page_create") reply({ revision_id: 456, parser_errors: [] });
      else if (payload.method === "page_rerender") reply(true);
      else if (payload.method === "page_get") reply({
        page_id: 789,
        page_category_id: 101,
        revision_id: 456,
        title: "SCP-002",
        tags: ["scp", "test"],
        wikitext: "Preview fixture body\n",
        compiled_body_html: "<p>Preview fixture body</p>"
      });
      else fail(`unexpected method ${payload.method}`);
    });
  });

  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  try {
    const { port } = server.address();
    await runPreview([
      "--source",
      sourcePath,
      "--manifest",
      manifestPath,
      "--output-dir",
      outputDir,
      "--rpc-url",
      `http://127.0.0.1:${port}/jsonrpc`,
      "--json"
    ]);
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }

  const createCall = calls.find((call) => call.method === "page_create");
  assert.deepEqual(createCall.params.tags, ["scp", "test"]);
});

test("preview-source rejects missing required option values", async () => {
  await assertPreviewFails([], /--source is required/);
  await assertPreviewFails(["--source"], /--source requires a value/);
  await assertPreviewFails(["--source", "--output-dir"], /--source requires a value/);
  await assertPreviewFails(["--source", "fixture", "--rpc-url"], /--rpc-url requires a value/);
  await assertPreviewFails(["--source", "fixture", "--rpc-timeout-ms"], /--rpc-timeout-ms requires a value/);
  await assertPreviewFails(["--source", "fixture", "--rpc-timeout-ms", "--json"], /--rpc-timeout-ms requires a value/);
  await assertPreviewFails(["--source", "fixture", "--rpc-timeout-ms", "0"], /--rpc-timeout-ms must be a positive integer/);
  await assertPreviewFails(["--source", "fixture", "--rpc-timeout-ms", "10ms"], /--rpc-timeout-ms must be a positive integer/);
});
