import crypto from "node:crypto";
import {spawn} from "node:child_process";
import path from "node:path";
import readline from "node:readline";
import {fileURLToPath} from "node:url";

import {ALLOWED_SITE_SLUG} from "./theme-localization-e2e.mjs";
import {targetRoundTripSourceSha256} from "./theme-source-roundtrip.mjs";

const DEFAULT_TIMEOUT_MS = 30_000;
const RUN_OWNED_SLUG = /^theme:codex-l10n-[a-z0-9][a-z0-9-]+-(?:yossistyle|ashes-to-ashes|basalt)$/u;
const SECRET_KEY = /password|cookie|credential|session|token/iu;
const HELPER_PATH = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../scripts/wikidot-theme-page-helper.py");

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function validateResource(resource) {
  if (resource?.target !== "wikidot" || typeof resource.slug !== "string" || resource.slug.length > 100 || !RUN_OWNED_SLUG.test(resource.slug)) {
    throw new Error("Wikidot adapter accepts only run-owned theme pages");
  }
  const url = new URL(resource.url);
  if (url.protocol !== "http:" || url.hostname !== `${ALLOWED_SITE_SLUG}.wikidot.com` || url.port || url.pathname !== `/${resource.slug}` || url.search || url.hash || url.username || url.password) {
    throw new Error("Wikidot adapter resource URL is outside the hard allowlist");
  }
}

function containsSecretField(value) {
  if (Array.isArray(value)) return value.some(containsSecretField);
  if (!value || typeof value !== "object") return false;
  return Object.entries(value).some(([key, child]) => SECRET_KEY.test(key) || containsSecretField(child));
}

function minimalEnvironment(source) {
  const username = source.WIKIDOT_USERNAME;
  const password = source.WIKIDOT_PASSWORD;
  if (typeof username !== "string" || !username || typeof password !== "string" || !password) {
    throw new Error("Wikidot helper credentials must be supplied through WIKIDOT_USERNAME and WIKIDOT_PASSWORD");
  }
  return {
    HOME: source.HOME ?? "",
    LANG: source.LANG ?? "C.UTF-8",
    PATH: source.PATH ?? "/usr/bin:/bin",
    WIKIDOT_PASSWORD: password,
    WIKIDOT_PY_ROOT: source.WIKIDOT_PY_ROOT ?? "/home/roku/src/Rokurolize/wikidot.py/src",
    WIKIDOT_USERNAME: username,
  };
}

export class WikidotJsonlHelperClient {
  constructor({command = "python3", commandArgs = [HELPER_PATH], env = process.env, timeoutMs = DEFAULT_TIMEOUT_MS, spawnImpl = spawn} = {}) {
    if (typeof command !== "string" || !command || !Array.isArray(commandArgs) || commandArgs.some((arg) => typeof arg !== "string")) throw new Error("Wikidot helper command is invalid");
    if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0) throw new Error("Wikidot helper timeout must be a positive integer");
    const childEnvironment = minimalEnvironment(env);
    if (commandArgs.some((arg) => arg.includes(childEnvironment.WIKIDOT_USERNAME) || arg.includes(childEnvironment.WIKIDOT_PASSWORD))) throw new Error("Wikidot credentials must not appear in helper command arguments");
    this.command = command;
    this.commandArgs = commandArgs;
    this.env = childEnvironment;
    this.timeoutMs = timeoutMs;
    this.spawnImpl = spawnImpl;
    this.child = null;
    this.pending = new Map();
    this.nextId = 1;
    this.exited = false;
    this.closing = false;
  }

  async start() {
    if (this.child) return this;
    const childEnvironment = this.env;
    this.env = null;
    this.child = this.spawnImpl(this.command, this.commandArgs, {env: childEnvironment, stdio: ["pipe", "pipe", "ignore"]});
    this.exitPromise = new Promise((resolve) => this.child.once("exit", resolve));
    const lines = readline.createInterface({input: this.child.stdout, crlfDelay: Infinity});
    lines.on("line", (line) => this.handleLine(line));
    this.child.on("error", () => this.failAll(new Error("Wikidot helper process could not start")));
    this.child.on("exit", (code, signal) => {
      this.exited = true;
      this.failAll(this.closing ? new Error("Wikidot helper closed") : new Error(`Wikidot helper exited unexpectedly (code=${code ?? "none"}, signal=${signal ?? "none"})`));
    });
    let ping;
    try {
      ping = await this.request("ping");
    } catch (error) {
      this.terminate(error);
      throw error;
    }
    if (ping?.protocol !== "wikijump.theme_wikidot_helper.v1" || ping.site !== ALLOWED_SITE_SLUG) {
      this.terminate(new Error("Wikidot helper handshake failed"));
      throw new Error("Wikidot helper handshake failed");
    }
    return this;
  }

  handleLine(line) {
    let response;
    try {
      if (Buffer.byteLength(line) > 65_536) throw new Error();
      response = JSON.parse(line);
    } catch {
      this.terminate(new Error("Wikidot helper returned invalid JSONL"));
      return;
    }
    if (!response || !Number.isSafeInteger(response.id) || typeof response.ok !== "boolean" || containsSecretField(response)) {
      this.terminate(new Error("Wikidot helper returned an invalid or secret-bearing response"));
      return;
    }
    const pending = this.pending.get(response.id);
    if (!pending) {
      this.terminate(new Error("Wikidot helper returned an unexpected response id"));
      return;
    }
    clearTimeout(pending.timeout);
    this.pending.delete(response.id);
    if (response.ok) {
      pending.resolve(response.result);
      return;
    }
    const code = typeof response.error?.code === "string" && /^[a-z][a-z0-9_]{0,63}$/u.test(response.error.code) ? response.error.code : "operation_failed";
    pending.reject(new Error(`Wikidot helper ${pending.action} failed (${code})`));
  }

  failAll(error) {
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timeout);
      pending.reject(error);
    }
    this.pending.clear();
  }

  terminate(error) {
    this.failAll(error);
    this.closing = true;
    this.child?.kill("SIGTERM");
  }

  request(action, fields = {}) {
    if (!this.child || this.exited || this.closing) return Promise.reject(new Error("Wikidot helper is not running"));
    const request = {id: this.nextId++, action, ...fields};
    if (containsSecretField(request)) return Promise.reject(new Error("Wikidot helper request contains a forbidden secret field"));
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.terminate(new Error(`Wikidot helper ${action} timed out after ${this.timeoutMs}ms`));
      }, this.timeoutMs);
      this.pending.set(request.id, {action, resolve, reject, timeout});
      this.child.stdin.write(`${JSON.stringify(request)}\n`, (error) => {
        if (error && this.pending.has(request.id)) {
          clearTimeout(timeout);
          this.pending.delete(request.id);
          reject(new Error("Wikidot helper request transport failed"));
        }
      });
    });
  }

  async close() {
    if (!this.child || this.exited) return;
    this.closing = true;
    let timer;
    try {
      const request = {id: this.nextId++, action: "shutdown"};
      const shutdown = new Promise((resolve, reject) => {
        const timeout = setTimeout(resolve, Math.min(this.timeoutMs, 2_000));
        this.pending.set(request.id, {action: "shutdown", resolve, reject, timeout});
        this.child.stdin.write(`${JSON.stringify(request)}\n`);
      });
      await Promise.race([shutdown.catch(() => {}), this.exitPromise, new Promise((resolve) => { timer = setTimeout(resolve, Math.min(this.timeoutMs, 2_000)); })]);
    } finally {
      clearTimeout(timer);
      this.child.stdin.end();
      if (!this.exited) {
        await Promise.race([this.exitPromise, new Promise((resolve) => { timer = setTimeout(resolve, Math.min(this.timeoutMs, 2_000)); })]);
        clearTimeout(timer);
      }
      if (!this.exited) this.child.kill("SIGTERM");
      this.failAll(new Error("Wikidot helper closed"));
    }
  }
}

export class WikidotThemePageAdapter {
  constructor({helperClient, helperOptions} = {}) {
    this.helper = helperClient ?? new WikidotJsonlHelperClient(helperOptions);
  }

  async connect() {
    await this.helper.start();
    return this;
  }

  async inspect(resource) {
    validateResource(resource);
    const result = await this.helper.request("inspect", {slug: resource.slug});
    const page = result?.page;
    if (page === null) return null;
    if (!Number.isSafeInteger(page?.identity) || typeof page.title !== "string" || !/^[0-9a-f]{64}$/u.test(page.source_sha256)) {
      throw new Error("Wikidot helper returned an incomplete page identity");
    }
    return page;
  }

  async create(resource, payload) {
    validateResource(resource);
    if (typeof payload?.source !== "string" || sha256(payload.source) !== resource.source_sha256) throw new Error("Wikidot create source does not match the accepted source hash");
    if (await this.inspect(resource) !== null) throw new Error("Wikidot create-only guard found a preexisting page");
    const result = await this.helper.request("create", {slug: resource.slug, title: resource.title, source: payload.source, source_sha256: resource.source_sha256});
    const page = result?.page;
    if (!Number.isSafeInteger(page?.identity) || page.title !== resource.title || page.source_sha256 !== targetRoundTripSourceSha256("wikidot", payload.source)) {
      throw new Error("Wikidot page did not round-trip after create");
    }
    return page.identity;
  }

  async remove(resource, {expected, identity} = {}) {
    validateResource(resource);
    const actual = await this.inspect(resource);
    if (actual === null) return;
    if (actual.source_sha256 !== expected?.source_sha256 || actual.title !== expected?.title || (identity !== undefined && actual.identity !== identity)) {
      throw new Error("Wikidot delete refused a page whose identity, title, or source changed");
    }
    await this.helper.request("remove", {slug: resource.slug, expected: {identity: identity ?? actual.identity, title: expected.title, source_sha256: expected.source_sha256}});
    if (await this.inspect(resource) !== null) throw new Error("Wikidot page remains after delete");
  }

  async close() {
    await this.helper.close();
  }
}
