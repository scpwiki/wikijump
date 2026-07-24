import assert from "node:assert/strict";
import crypto from "node:crypto";
import {spawnSync} from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {fileURLToPath} from "node:url";

import {ThemeExecutionLedger, cleanupThemeExecution} from "../src/theme-localization-execution.mjs";
import {WIKIDOT_HELPER_PYTHON, WikidotJsonlHelperClient, WikidotThemePageAdapter} from "../src/theme-localization-wikidot-adapter.mjs";
import {targetRoundTripSourceSha256} from "../src/theme-source-roundtrip.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const HELPER_PATH = path.resolve(HERE, "../scripts/wikidot_theme_page_helper.py");
const SITE = "scpaiueouiuiuiui";

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function fixtureResource() {
  const source = "日本語 theme source\n";
  const slug = "codex-l10n:20260713-adapter-yossistyle";
  return {source, resource: {resource_id: "yossistyle:wikidot", tier_id: "yossistyle", target: "wikidot", slug, url: `https://${SITE}.wikidot.com/${slug}`, source_sha256: sha256(source), title: "Theme localization canary: yossistyle", tags: ["テーマ"]}};
}

function prerequisiteResource() {
  const source = "[[include component:image-block-base]]";
  const slug = "component:image-block";
  return {source, resource: {resource_id: `prerequisite:${slug}:wikidot`, kind: "reference_prerequisite", target: "wikidot", slug, url: `https://${SITE}.wikidot.com/${slug}`, source_sha256: sha256(source), title: "Image Block", tags: ["codex-source-parity-redo", "component"]}};
}

class FakeHelper {
  constructor({failAfterCreate = false} = {}) {
    this.pages = new Map();
    this.calls = [];
    this.failAfterCreate = failAfterCreate;
  }

  async start() {}
  async close() {}

  async request(action, fields) {
    this.calls.push({action, fields});
    if (action === "inspect") return {page: this.pages.get(fields.slug) ?? null};
    if (action === "create") {
      const page = {identity: 1234, title: fields.title, source_sha256: targetRoundTripSourceSha256("wikidot", fields.source), tags: fields.tags};
      this.pages.set(fields.slug, page);
      if (this.failAfterCreate) throw new Error("simulated post-save transport failure");
      return {page};
    }
    if (action === "remove") {
      assert.deepEqual(this.pages.get(fields.slug), fields.expected);
      this.pages.delete(fields.slug);
      return {removed: true};
    }
    throw new Error(`unexpected action ${action}`);
  }
}

test("private-site adapter uses the execution interface without ListPages lookup", async () => {
  const helper = new FakeHelper();
  const adapter = new WikidotThemePageAdapter({helperClient: helper});
  const {resource, source} = fixtureResource();
  await adapter.connect();
  assert.equal(await adapter.inspect(resource), null);
  const identity = await adapter.create(resource, {source});
  assert.equal(identity, 1234);
  await adapter.remove(resource, {expected: {title: resource.title, source_sha256: targetRoundTripSourceSha256("wikidot", source), tags: resource.tags}, identity});
  assert.equal(await adapter.inspect(resource), null);
  assert.deepEqual(helper.calls.map(({action}) => action), ["inspect", "inspect", "create", "inspect", "remove", "inspect", "inspect"]);
  assert.deepEqual(helper.calls.find(({action}) => action === "create").fields.tags, ["テーマ"]);
  await assert.rejects(adapter.inspect({...resource, url: `https://scp-wiki.wikidot.com/${resource.slug}`}), /hard allowlist/);
  await assert.rejects(adapter.inspect({...resource, url: `http://${SITE}.wikidot.com/${resource.slug}`}), /hard allowlist/);
  await assert.rejects(adapter.inspect({...resource, slug: "theme:yossistyle"}), /validated/);
  const legacySlug = "theme:codex-l10n-20260713-adapter-yossistyle";
  const legacy = {...resource, slug: legacySlug, url: `https://${SITE}.wikidot.com/${legacySlug}`};
  assert.equal(await adapter.inspect(legacy), null);
  await assert.rejects(adapter.create(legacy, {source}), /validated/);
});

test("private-site adapter exposes exact read-only reference prerequisites", async () => {
  const helper = new FakeHelper();
  const adapter = new WikidotThemePageAdapter({helperClient: helper});
  const {resource, source} = prerequisiteResource();
  helper.pages.set(resource.slug, {identity: 77, title: resource.title, source_sha256: resource.source_sha256, tags: resource.tags});
  assert.equal((await adapter.inspect(resource)).identity, 77);
  await assert.rejects(adapter.create(resource, {source}), /read-only/);
  await assert.rejects(adapter.inspect({...resource, slug: "component:other", url: `http://${SITE}.wikidot.com/component:other`}), /validated/);
  await assert.rejects(adapter.inspect({...resource, title: "changed"}), /read-only contract/);
});

test("durable create intent retains a page after a post-save error without a recorded identity", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "theme-wikidot-intent-"));
  const helper = new FakeHelper({failAfterCreate: true});
  const adapter = new WikidotThemePageAdapter({helperClient: helper});
  const {resource, source} = fixtureResource();
  const ledger = await ThemeExecutionLedger.create(path.join(root, "ledger.jsonl"), {runId: "20260713-adapter", fingerprint: "fixture", prerequisites: [], resources: [resource]});
  const expected = {title: resource.title, source_sha256: resource.source_sha256, remote_source_sha256: targetRoundTripSourceSha256("wikidot", source), tags: resource.tags};
  await ledger.intent(resource, expected);
  await assert.rejects(adapter.create(resource, {source}), /post-save/);
  assert.notEqual(await adapter.inspect(resource), null);
  await assert.rejects(cleanupThemeExecution({ledger, adapters: {wikidot: adapter}}), /cleanup left residual resources/);
  assert.equal(helper.pages.size, 1);
  const recovered = await ThemeExecutionLedger.load(ledger.filePath);
  assert.equal(recovered.completed, false);
  assert.equal(recovered.states.get(resource.resource_id).phase, "residual");
});

test("cleanup refuses content or identity changed after creation", async () => {
  const helper = new FakeHelper();
  const adapter = new WikidotThemePageAdapter({helperClient: helper});
  const {resource, source} = fixtureResource();
  const identity = await adapter.create(resource, {source});
  helper.pages.set(resource.slug, {...helper.pages.get(resource.slug), title: "changed"});
  await assert.rejects(adapter.remove(resource, {expected: {title: resource.title, source_sha256: resource.source_sha256, tags: resource.tags}, identity}), /identity, title, or source changed/);
  assert.equal(helper.calls.some(({action}) => action === "remove"), false);
  assert.equal(helper.pages.size, 1);
});

test("Python helper contains only direct authenticated page primitives", async () => {
  const source = await fs.readFile(HELPER_PATH, "utf8");
  assert.match(source, /viewsource\/ViewSourceModule/);
  assert.match(source, /edit\/PageEditModule/);
  assert.match(source, /"event": "deletePage"/);
  assert.match(source, /"event": "saveTags"/);
  assert.match(source, /ALLOWED_ORIGIN}\/ajax-module-connector\.php/);
  assert.match(source, /page_revision_id/);
  assert.doesNotMatch(source, /ListPagesModule/);
  assert.doesNotMatch(source, /site\.page\.get/);
  assert.doesNotMatch(source, /site\.amc_request/);
  assert.doesNotMatch(source, /WIKIDOT_PY_ROOT|sys\.path\.insert/);
  assert.doesNotMatch(source, /str\(exc\)|repr\(exc\)/);
});

test("production helper uses the component-owned virtual environment", () => {
  const client = new WikidotJsonlHelperClient({env: helperEnvironment()});
  assert.equal(client.command, path.resolve(HERE, "../.venv/bin/python"));
  assert.equal(client.command, WIKIDOT_HELPER_PYTHON);
  assert.equal("WIKIDOT_PY_ROOT" in client.env, false);
});

test("create-only backend rejects an existing PageEditModule revision", () => {
  const program = String.raw`
import importlib.util, sys
spec = importlib.util.spec_from_file_location("theme_helper", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
backend = object.__new__(module.WikidotBackend)
backend.inspect = lambda slug, kind="theme_page": None
backend._request_ajax_module_connector = lambda body: {"status": "ok", "lock_id": "lock", "lock_secret": "secret", "page_revision_id": 99}
source = "fixture source"
try:
    backend.create("codex-l10n:20260713-adapter-yossistyle", title="fixture", source=source, expected_source_sha256=module.sha256(source), tags=["テーマ"])
except module.PublicError as error:
    print(error.code)
`;
  const result = spawnSync("python3", ["-c", program, HELPER_PATH], {encoding: "utf8"});
  assert.equal(result.status, 0);
  assert.equal(result.stdout.trim(), "page_exists");
  assert.equal(result.stderr, "");
});

test("create-only backend saves and verifies the deterministic run-owned tag", () => {
  const program = String.raw`
import importlib.util, sys
spec = importlib.util.spec_from_file_location("theme_helper", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
backend = object.__new__(module.WikidotBackend)
source = "fixture source"
actual = {"identity": 7, "title": "fixture", "source_sha256": module.sha256(source)}
inspections = iter([None, actual])
backend.inspect = lambda slug, kind="theme_page": next(inspections)
events = []
def amc(body):
    events.append(body.get("event", body.get("moduleName")))
    if body.get("moduleName") == "edit/PageEditModule": return {"status": "ok", "lock_id": "lock", "lock_secret": "secret"}
    return {"status": "ok"}
backend._request_ajax_module_connector = amc
backend.page_tags = lambda slug, kind="theme_page": ["テーマ"]
created = backend.create("codex-l10n:20260713-adapter-yossistyle", title="fixture", source=source, expected_source_sha256=module.sha256(source), tags=["テーマ"])
print(created["identity"])
print(",".join(created["tags"]))
print(",".join(events))
`;
  const result = spawnSync("python3", ["-c", program, HELPER_PATH], {encoding: "utf8"});
  assert.equal(result.status, 0);
  assert.equal(result.stderr, "");
  assert.equal(result.stdout.trim(), "7\nテーマ\nedit/PageEditModule,savePage,saveTags");
});

test("Wikidot round-trip hash removes only one observed terminal LF", () => {
  const program = String.raw`
import importlib.util, sys
spec = importlib.util.spec_from_file_location("theme_helper", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
print(module.wikidot_round_trip_sha256("fixture\n"))
print(module.wikidot_round_trip_sha256("fixture"))
print(module.wikidot_round_trip_sha256("fixture\n\n"))
`;
  const result = spawnSync("python3", ["-c", program, HELPER_PATH], {encoding: "utf8"});
  assert.equal(result.status, 0);
  assert.equal(result.stderr, "");
  assert.deepEqual(result.stdout.trim().split("\n"), [sha256("fixture"), sha256("fixture"), sha256("fixture\n")]);
});

test("Python helper reserves new slugs for creates but retains exact legacy cleanup access", () => {
  const program = String.raw`
import importlib.util, sys
spec = importlib.util.spec_from_file_location("theme_helper", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
current = "codex-l10n:20260713-adapter-yossistyle"
legacy = "theme:codex-l10n-20260713-adapter-yossistyle"
print(module.validate_slug(current))
print(module.validate_slug(legacy, allow_legacy=True))
print(module.validate_slug("component:image-block", kind="reference_prerequisite"))
try: module.validate_slug(legacy)
except module.PublicError as error: print(error.code)
try: module.validate_slug("component:other", kind="reference_prerequisite")
except module.PublicError as error: print(error.code)
`;
  const result = spawnSync("python3", ["-c", program, HELPER_PATH], {encoding: "utf8"});
  assert.equal(result.status, 0);
  assert.equal(result.stderr, "");
  assert.equal(result.stdout.trim(), "codex-l10n:20260713-adapter-yossistyle\ntheme:codex-l10n-20260713-adapter-yossistyle\ncomponent:image-block\nresource_not_allowed\nresource_not_allowed");
});

test("authenticated GET treats only HTTP 404 as page absence", () => {
  const program = String.raw`
import importlib.util, sys
spec = importlib.util.spec_from_file_location("theme_helper", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
class Response:
    is_redirect = False
    def __init__(self, status, text): self.status_code, self.text = status, text
class Client:
    def __init__(self, response, **kwargs): self.response = response
    def __enter__(self): return self
    def __exit__(self, *args): pass
    def get(self, *args, **kwargs): return self.response
class Httpx:
    def __init__(self, response): self.response = response
    def Client(self, **kwargs): return Client(self.response, **kwargs)
backend = object.__new__(module.WikidotBackend)
backend.headers = {}
backend.httpx = Httpx(Response(404, ""))
print(backend._get("codex-l10n:20260713-adapter-yossistyle") is None)
backend.httpx = Httpx(Response(200, ""))
try: backend._get("codex-l10n:20260713-adapter-yossistyle")
except module.PublicError as error: print(error.code)
`;
  const result = spawnSync("python3", ["-c", program, HELPER_PATH], {encoding: "utf8"});
  assert.equal(result.status, 0);
  assert.equal(result.stdout.trim(), "True\nauthenticated_get_failed");
});

function helperEnvironment(password = "unit-test-password") {
  return {HOME: process.env.HOME, LANG: "C.UTF-8", PATH: process.env.PATH, WIKIDOT_USERNAME: "unit-test-user", WIKIDOT_PASSWORD: password};
}

test("persistent JSONL client drives the production dispatcher", async () => {
  const program = String.raw`
import importlib.util, sys
spec = importlib.util.spec_from_file_location("theme_helper", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
class Backend:
    def __init__(self): self.count = 0
    def inspect(self, slug, kind="theme_page"):
        self.count += 1
        return {"identity": self.count, "title": "fixture", "source_sha256": "0" * 64, "tags": []}
    def close(self): pass
module.serve(sys.stdin, sys.stdout, Backend())
`;
  const client = new WikidotJsonlHelperClient({command: "python3", commandArgs: ["-c", program, HELPER_PATH], env: helperEnvironment(), timeoutMs: 5_000});
  await client.start();
  const slug = fixtureResource().resource.slug;
  assert.equal((await client.request("inspect", {slug})).page.identity, 1);
  assert.equal((await client.request("inspect", {slug})).page.identity, 2);
  await assert.rejects(client.request("inspect", {slug, session_token: "forbidden"}), /forbidden secret field/);
  await client.close();
});

test("Python helper closes its backend when response delivery fails", () => {
  const program = String.raw`
import importlib.util, io, sys
spec = importlib.util.spec_from_file_location("theme_helper", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
class Backend:
    closed = False
    def close(self): self.closed = True
class BrokenOutput:
    def write(self, value): raise OSError("closed pipe")
    def flush(self): pass
backend = Backend()
try:
    module.serve(io.StringIO('{"id":1,"action":"ping"}\n'), BrokenOutput(), backend)
except OSError:
    pass
print(backend.closed)
`;
  const result = spawnSync("python3", ["-c", program, HELPER_PATH], {encoding: "utf8"});
  assert.equal(result.status, 0);
  assert.equal(result.stderr, "");
  assert.equal(result.stdout.trim(), "True");
});

test("helper errors and process exits never expose credentials", async () => {
  const secret = "never-print-this-password";
  const errorProgram = String.raw`
import json, os, sys
request = json.loads(sys.stdin.readline())
print(json.dumps({"id": request["id"], "ok": False, "error": {"code": "authentication_failed", "message": os.environ["WIKIDOT_PASSWORD"]}}), flush=True)
`;
  const rejected = new WikidotJsonlHelperClient({command: "python3", commandArgs: ["-c", errorProgram], env: helperEnvironment(secret), timeoutMs: 5_000});
  await assert.rejects(rejected.start(), (error) => !error.message.includes(secret) && /authentication_failed/.test(error.message));
  await rejected.close();

  const exited = new WikidotJsonlHelperClient({command: "python3", commandArgs: ["-c", "import sys; sys.stdin.readline()"], env: helperEnvironment(secret), timeoutMs: 5_000});
  await assert.rejects(exited.start(), (error) => !error.message.includes(secret) && /exited unexpectedly/.test(error.message));
});
