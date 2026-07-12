import assert from "node:assert/strict";
import crypto from "node:crypto";
import test from "node:test";

import {ALLOWED_SITE_SLUG} from "../src/theme-localization-e2e.mjs";
import {DeepwellThemePageAdapter, validateLocalDeepwellRpcUrl} from "../src/theme-localization-deepwell-adapter.mjs";

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function resource() {
  const source = "日本語 theme source\n";
  const slug = "codex-l10n:20260713-adapter-yossistyle";
  return {resource: {resource_id: "yossistyle:wikijump", target: "wikijump", slug, url: `https://${ALLOWED_SITE_SLUG}.wikijump.localhost:18443/${slug}`, source_sha256: sha256(source), title: "Theme localization canary: yossistyle"}, source};
}

class FakeRpc {
  constructor() {
    this.calls = [];
    this.page = null;
    this.parserErrors = [];
  }

  async call(method, params, context = {}) {
    this.calls.push({method, params, context});
    if (method === "ping") return "Pong!";
    if (method === "site_get") return {site_id: 42};
    if (method === "login") return {session_token: "secret-session-token", needs_mfa: false};
    if (method === "page_get") return this.page;
    if (method === "page_create") {
      if (this.page) throw new Error("collision");
      this.page = {page_id: 100, revision_id: 200, wikitext: params.wikitext, title: params.title};
      return {parser_errors: this.parserErrors};
    }
    if (method === "page_delete") {
      this.page = null;
      return {};
    }
    throw new Error(`unexpected method ${method}`);
  }
}

async function connectedAdapter() {
  const rpc = new FakeRpc();
  const adapter = new DeepwellThemePageAdapter({rpcClient: rpc, adminEmail: "admin@wikijump", adminPassword: "password"});
  await adapter.connect();
  return {rpc, adapter};
}

test("RPC URL accepts only loopback HTTP JSON-RPC endpoints", () => {
  assert.equal(validateLocalDeepwellRpcUrl("http://127.0.0.1:12747/jsonrpc"), "http://127.0.0.1:12747/jsonrpc");
  for (const value of ["https://127.0.0.1/jsonrpc", "http://deepwell:2747/jsonrpc", "http://user:pass@127.0.0.1/jsonrpc", "http://127.0.0.1/admin"]) {
    assert.throws(() => validateLocalDeepwellRpcUrl(value), /loopback/);
  }
});

test("connect resolves only the allowlisted site and does not retain the password", async () => {
  const {rpc, adapter} = await connectedAdapter();
  assert.equal(adapter.siteId, 42);
  assert.equal(adapter.adminPassword, null);
  assert.deepEqual(rpc.calls.slice(0, 3).map((call) => call.method), ["ping", "site_get", "login"]);
  assert.equal(rpc.calls[1].params.site, ALLOWED_SITE_SLUG);
});

test("create is create-only, authenticated, and verifies the accepted source", async () => {
  const {rpc, adapter} = await connectedAdapter();
  const fixture = resource();
  assert.equal(await adapter.create(fixture.resource, {source: fixture.source}), 100);
  const create = rpc.calls.find((call) => call.method === "page_create");
  assert.equal(create.context.sessionToken, "secret-session-token");
  assert.equal(create.context.siteId, 42);
  assert.equal(create.params.slug, fixture.resource.slug);
  await assert.rejects(adapter.create(fixture.resource, {source: fixture.source}), /preexisting/);
  await assert.rejects(adapter.create({...fixture.resource, slug: "scp-173"}, {source: fixture.source}), /run-owned/);
  const legacySlug = "theme:codex-l10n-20260713-adapter-yossistyle";
  const legacy = {...fixture.resource, slug: legacySlug, url: `https://${ALLOWED_SITE_SLUG}.wikijump.localhost:18443/${legacySlug}`};
  rpc.page = null;
  assert.equal(await adapter.inspect(legacy), null);
  await assert.rejects(adapter.create(legacy, {source: fixture.source}), /run-owned/);
});

test("parser errors fail after creation so the outer intent ledger can clean up", async () => {
  const {rpc, adapter} = await connectedAdapter();
  const fixture = resource();
  rpc.parserErrors = [{message: "bad syntax"}];
  await assert.rejects(adapter.create(fixture.resource, {source: fixture.source}), /parser errors/);
  assert.notEqual(await adapter.inspect(fixture.resource), null);
});

test("remove refuses changed pages and deletes matching pages with revision fencing", async () => {
  const {rpc, adapter} = await connectedAdapter();
  const fixture = resource();
  const identity = await adapter.create(fixture.resource, {source: fixture.source});
  const expected = {source_sha256: fixture.resource.source_sha256, title: fixture.resource.title};
  rpc.page.title = "changed";
  await assert.rejects(adapter.remove(fixture.resource, {expected, identity}), /refused/);
  assert.notEqual(rpc.page, null);
  rpc.page.title = fixture.resource.title;
  await adapter.remove(fixture.resource, {expected, identity});
  const deletion = rpc.calls.find((call) => call.method === "page_delete");
  assert.equal(deletion.params.last_revision_id, 200);
  assert.equal(deletion.context.sessionToken, "secret-session-token");
  assert.equal(await adapter.inspect(fixture.resource), null);
});
