import crypto from "node:crypto";

import {ALLOWED_SITE_SLUG, isCurrentRunOwnedSlug, isRecoverableRunOwnedSlug} from "./theme-localization-e2e.mjs";

const DEFAULT_RPC_TIMEOUT_MS = 30_000;
const IP_ADDRESS = "127.0.0.1";
const MAX_REPORTED_PARSER_ERRORS = 12;
const MAX_PARSER_ERROR_FIELD_LENGTH = 80;
const MATERIALIZED_COMPONENT_TITLES = new Map([
  ["component:image-block-base", "Image Block Base"],
  ["component:image-block", "Image Block"],
]);

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function parserErrorSummary(errors) {
  const diagnostic = errors.slice(0, MAX_REPORTED_PARSER_ERRORS).map((error) => {
    const output = {};
    for (const field of ["token", "rule", "kind"]) {
      if (["string", "number", "boolean"].includes(typeof error?.[field])) {
        output[field] = String(error[field]).slice(0, MAX_PARSER_ERROR_FIELD_LENGTH);
      }
    }
    if (Array.isArray(error?.span) && error.span.length === 2 && error.span.every(Number.isSafeInteger)) output.span = error.span;
    return output;
  });
  return JSON.stringify({reported: diagnostic.length, omitted: Math.max(0, errors.length - diagnostic.length), errors: diagnostic});
}

export function validateLocalDeepwellRpcUrl(value) {
  const url = new URL(value);
  if (url.protocol !== "http:" || !new Set(["127.0.0.1", "localhost"]).has(url.hostname) || url.pathname !== "/jsonrpc" || url.username || url.password || url.search || url.hash) {
    throw new Error("Deepwell RPC URL must be an uncredentialed loopback HTTP /jsonrpc endpoint");
  }
  return url.href;
}

function validateResource(resource, {allowLegacy = false} = {}) {
  const kind = resource?.kind ?? "theme_page";
  const dependency = kind === "component_dependency" && MATERIALIZED_COMPONENT_TITLES.has(resource?.slug);
  const validSlug = dependency || (allowLegacy ? isRecoverableRunOwnedSlug(resource?.slug) : isCurrentRunOwnedSlug(resource?.slug));
  if (resource?.target !== "wikijump" || !validSlug) {
    throw new Error("Deepwell adapter accepts only validated Wikijump theme execution pages");
  }
  const url = new URL(resource.url);
  if (url.protocol !== "https:" || url.hostname !== `${ALLOWED_SITE_SLUG}.wikijump.localhost` || !new Set(["", "18443"]).has(url.port) || url.pathname !== `/${resource.slug}` || url.search || url.hash) {
    throw new Error("Deepwell adapter resource URL is outside the hard allowlist");
  }
  if (dependency && (resource.title !== MATERIALIZED_COMPONENT_TITLES.get(resource.slug) || resource.resource_id !== `dependency:${resource.slug}:wikijump` || !/^[0-9a-f]{32}$/u.test(resource.ownership_token))) {
    throw new Error("Deepwell adapter dependency resource is outside the materialized contract");
  }
  const expectedTags = dependency ? [`codex-l10n-owner-${resource.ownership_token}`, "component"] : resource.slug.endsWith("-yossistyle") ? ["テーマ"] : resource.slug.endsWith("-ashes-to-ashes") || resource.slug.endsWith("-basalt") ? ["theme"] : [];
  if (!allowLegacy && JSON.stringify(resource.tags ?? []) !== JSON.stringify(expectedTags)) throw new Error("Deepwell adapter resource tags are outside the run-owned contract");
}

export class DeepwellJsonRpcClient {
  constructor({rpcUrl = "http://127.0.0.1:2747/jsonrpc", timeoutMs = DEFAULT_RPC_TIMEOUT_MS, fetchImpl = globalThis.fetch} = {}) {
    if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0) throw new Error("Deepwell RPC timeout must be a positive integer");
    if (typeof fetchImpl !== "function") throw new Error("Deepwell RPC fetch implementation is required");
    this.rpcUrl = validateLocalDeepwellRpcUrl(rpcUrl);
    this.timeoutMs = timeoutMs;
    this.fetchImpl = fetchImpl;
    this.nextId = 1;
  }

  async call(method, params = {}, context = {}) {
    const headers = {"content-type": "application/json"};
    if (context.sessionToken) headers["X-Deepwell-Session-Token"] = context.sessionToken;
    if (context.siteId) headers["X-Deepwell-Site-Id"] = String(context.siteId);
    if (context.page) headers["X-Deepwell-Page"] = context.page;
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), this.timeoutMs);
    let response;
    try {
      response = await this.fetchImpl(this.rpcUrl, {method: "POST", headers, body: JSON.stringify({jsonrpc: "2.0", id: this.nextId++, method, params}), signal: controller.signal});
    } catch (error) {
      if (error.name === "AbortError") throw new Error(`Deepwell RPC ${method} timed out after ${this.timeoutMs}ms`);
      throw new Error(`Deepwell RPC ${method} transport failed: ${error.message}`);
    } finally {
      clearTimeout(timeout);
    }
    const text = await response.text();
    let body;
    try {
      body = JSON.parse(text);
    } catch {
      throw new Error(`Deepwell RPC ${method} returned invalid JSON with HTTP ${response.status}`);
    }
    if (!response.ok || body.error) throw new Error(`Deepwell RPC ${method} failed with HTTP ${response.status}: ${body.error?.message ?? "unknown error"}`);
    return body.result;
  }
}

export class DeepwellThemePageAdapter {
  constructor({rpcClient, rpcUrl, timeoutMs, adminEmail, adminPassword, actorUserId = -1, siteSlug = ALLOWED_SITE_SLUG} = {}) {
    if (siteSlug !== ALLOWED_SITE_SLUG) throw new Error("Deepwell adapter site is outside the hard allowlist");
    if (typeof adminEmail !== "string" || !adminEmail || typeof adminPassword !== "string" || !adminPassword) throw new Error("Deepwell adapter credentials are required");
    if (!Number.isSafeInteger(actorUserId)) throw new Error("Deepwell adapter actor user id must be an integer");
    this.rpc = rpcClient ?? new DeepwellJsonRpcClient({rpcUrl, timeoutMs});
    this.siteSlug = siteSlug;
    this.adminEmail = adminEmail;
    this.adminPassword = adminPassword;
    this.actorUserId = actorUserId;
    this.siteId = null;
    this.sessionToken = null;
  }

  async connect() {
    await this.rpc.call("ping", {});
    const site = await this.rpc.call("site_get", {site: this.siteSlug});
    if (!Number.isSafeInteger(site?.site_id)) throw new Error("Deepwell site lookup did not return an integer site id");
    const login = await this.rpc.call("login", {name_or_email: this.adminEmail, password: this.adminPassword, ip_address: IP_ADDRESS, user_agent: "wikijump-theme-localization-e2e/0.1"});
    if (login?.needs_mfa !== false) throw new Error("Deepwell adapter does not accept an incomplete MFA login");
    if (typeof login?.session_token !== "string" || !login.session_token) throw new Error("Deepwell login did not return a session token");
    this.siteId = site.site_id;
    this.sessionToken = login.session_token;
    this.adminPassword = null;
    return this;
  }

  context(resource) {
    if (!this.sessionToken || !Number.isSafeInteger(this.siteId)) throw new Error("Deepwell adapter is not connected");
    return {sessionToken: this.sessionToken, siteId: this.siteId, page: resource.slug};
  }

  async inspect(resource) {
    validateResource(resource, {allowLegacy: true});
    const page = await this.rpc.call("page_get", {site_id: this.siteId, page: resource.slug, details: {wikitext: true, compiled: false}}, this.context(resource));
    if (page === null) return null;
    if (!Number.isSafeInteger(page.page_id) || !Number.isSafeInteger(page.revision_id) || typeof page.wikitext !== "string" || typeof page.title !== "string" || !Array.isArray(page.tags) || page.tags.some((tag) => typeof tag !== "string")) {
      throw new Error("Deepwell page inspection returned an incomplete page");
    }
    return {identity: page.page_id, source_sha256: sha256(page.wikitext), title: page.title, revision_id: page.revision_id, tags: page.tags};
  }

  async create(resource, payload) {
    validateResource(resource);
    if (typeof payload?.source !== "string" || sha256(payload.source) !== resource.source_sha256) throw new Error("Deepwell create source does not match the accepted source hash");
    if (await this.inspect(resource) !== null) throw new Error("Deepwell create-only guard found a preexisting page");
    const result = await this.rpc.call("page_create", {
      site_id: this.siteId,
      wikitext: payload.source,
      title: resource.title,
      alt_title: null,
      slug: resource.slug,
      layout: "wikidot",
      revision_comments: "run-owned theme localization E2E create",
      user_id: this.actorUserId,
      ip_address: IP_ADDRESS,
      tags: resource.tags ?? [],
    }, this.context(resource));
    if (result?.parser_errors?.length) throw new Error(`Deepwell page_create reported ${result.parser_errors.length} parser errors: ${parserErrorSummary(result.parser_errors)}`);
    const actual = await this.inspect(resource);
    if (actual === null || actual.source_sha256 !== resource.source_sha256 || actual.title !== resource.title || JSON.stringify(actual.tags) !== JSON.stringify(resource.tags ?? [])) throw new Error("Deepwell page did not round-trip after create");
    return actual.identity;
  }

  async remove(resource, {expected, identity} = {}) {
    validateResource(resource, {allowLegacy: true});
    const actual = await this.inspect(resource);
    if (actual === null) return;
    if (actual.source_sha256 !== expected?.source_sha256 || actual.title !== expected?.title || JSON.stringify(actual.tags) !== JSON.stringify(expected?.tags) || (identity !== undefined && actual.identity !== identity)) {
      throw new Error("Deepwell delete refused a page whose identity or content changed");
    }
    await this.rpc.call("page_delete", {
      site_id: this.siteId,
      page: actual.identity,
      last_revision_id: actual.revision_id,
      revision_comments: "run-owned theme localization E2E cleanup",
      user_id: this.actorUserId,
      ip_address: IP_ADDRESS,
    }, this.context(resource));
    if (await this.inspect(resource) !== null) throw new Error("Deepwell page remains after delete");
  }

  close() {
    this.sessionToken = null;
    this.adminPassword = null;
    this.siteId = null;
  }
}
