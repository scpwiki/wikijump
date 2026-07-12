import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";

import {
  ALLOWED_SITE_SLUG,
  THEME_LOCALIZATION_E2E_SCHEMA,
  assertRunOwnedSlug,
  validateTargetOrigin,
} from "./theme-localization-e2e.mjs";
import {targetRoundTripSourceSha256} from "./theme-source-roundtrip.mjs";

export const THEME_EXECUTION_LEDGER_SCHEMA = "wikijump_local_lab.theme_execution_ledger.v1";

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

async function syncParentDirectory(filePath) {
  const handle = await fs.open(path.dirname(filePath), "r");
  try {
    await handle.sync();
  } finally {
    await handle.close();
  }
}

function stableResources(plan) {
  const resources = [];
  for (const tier of plan.tiers) {
    assertRunOwnedSlug(tier.run_owned_slug, plan.run.id, tier.id);
    if (tier.preflight.status !== "pass" || !tier.preflight.source.sha256) {
      throw new Error(`tier is not executable: ${tier.id}`);
    }
    for (const target of tier.targets) {
      if (!new Set(["wikidot", "wikijump"]).has(target.id)) {
        throw new Error(`unknown execution target: ${target.id}`);
      }
      if (target.resource_id !== `${tier.id}:${target.id}`) throw new Error(`invalid execution resource id: ${target.resource_id}`);
      const url = new URL(target.url);
      const expectedOrigin = validateTargetOrigin(target.origin, target.id);
      if (url.origin !== expectedOrigin || url.pathname !== `/${tier.run_owned_slug}` || url.search || url.hash || url.username || url.password) {
        throw new Error(`execution target URL is outside the hard allowlist: ${target.resource_id}`);
      }
      resources.push({
        resource_id: target.resource_id,
        tier_id: tier.id,
        target: target.id,
        slug: tier.run_owned_slug,
        url: target.url,
        source_path: tier.preflight.source.absolute_path,
        source_sha256: tier.preflight.source.sha256,
        title: `Theme localization canary: ${tier.id}`,
      });
    }
  }
  return resources;
}

export function validateThemeExecutionPlan(plan) {
  if (!plan || plan.schema !== THEME_LOCALIZATION_E2E_SCHEMA) throw new Error("invalid theme localization plan schema");
  if (plan.preflight?.status !== "pass") throw new Error("theme localization plan preflight did not pass");
  if (plan.run?.site_slug !== ALLOWED_SITE_SLUG) throw new Error("theme localization plan site is outside the hard allowlist");
  const hardAllowlist = plan.safety?.hard_allowlist;
  if (hardAllowlist?.site_slug !== ALLOWED_SITE_SLUG || hardAllowlist.wikidot_hostname !== `${ALLOWED_SITE_SLUG}.wikidot.com` || hardAllowlist.wikijump_hostname !== `${ALLOWED_SITE_SLUG}.wikijump.localhost`) {
    throw new Error("theme localization plan hard allowlist is invalid");
  }
  const resources = stableResources(plan);
  if (resources.length === 0) throw new Error("theme localization plan has no resources");
  const ids = new Set(resources.map((resource) => resource.resource_id));
  if (ids.size !== resources.length) throw new Error("theme localization plan has duplicate resource ids");
  return resources;
}

export function themeExecutionFingerprint(plan) {
  const resources = validateThemeExecutionPlan(plan);
  return sha256(JSON.stringify({schema: plan.schema, execution: {mode: plan.mode ?? null, execute_supported: plan.safety?.execute_supported === true}, run: plan.run, resources}));
}

function parseEvents(text) {
  const complete = text.endsWith("\n") ? text : text.slice(0, text.lastIndexOf("\n") + 1);
  const events = complete.split("\n").filter(Boolean).map((line, index) => {
    try {
      return JSON.parse(line);
    } catch {
      throw new Error(`invalid ledger event JSON at line ${index + 1}`);
    }
  });
  if (events.length === 0) throw new Error("execution ledger has no complete header event");
  events.forEach((event, index) => {
    if (event.seq !== index) throw new Error(`execution ledger sequence mismatch at line ${index + 1}`);
  });
  return {events, complete};
}

function reduceEvents(events) {
  const header = events[0];
  if (header.type !== "header" || header.schema !== THEME_EXECUTION_LEDGER_SCHEMA || !Array.isArray(header.resources)) {
    throw new Error("invalid execution ledger header");
  }
  const known = new Map(header.resources.map((resource) => [resource.resource_id, resource]));
  if (known.size !== header.resources.length) throw new Error("execution ledger has duplicate resource ids");
  const states = new Map();
  let completed = false;
  for (const event of events.slice(1)) {
    if (completed) throw new Error("execution ledger has events after completion");
    if (event.type === "completed") {
      if ([...states.values()].some((state) => state.phase !== "cleaned")) {
        throw new Error("execution ledger completed with residual resources");
      }
      completed = true;
      continue;
    }
    if (!known.has(event.resource_id)) throw new Error(`execution ledger references unknown resource: ${event.resource_id}`);
    const previous = states.get(event.resource_id);
    if (event.type === "intent") {
      if (previous) throw new Error(`duplicate execution intent: ${event.resource_id}`);
      states.set(event.resource_id, {phase: "intent", expected: event.expected});
    } else if (event.type === "created") {
      if (previous?.phase !== "intent") throw new Error(`created event without intent: ${event.resource_id}`);
      states.set(event.resource_id, {...previous, phase: "created", identity: event.identity});
    } else if (event.type === "cleaned") {
      if (!previous || previous.phase === "cleaned") throw new Error(`invalid cleaned event: ${event.resource_id}`);
      states.set(event.resource_id, {...previous, phase: "cleaned"});
    } else if (event.type === "residual") {
      if (!previous || previous.phase === "cleaned") throw new Error(`invalid residual event: ${event.resource_id}`);
      states.set(event.resource_id, {...previous, phase: "residual", reason: event.reason});
    } else {
      throw new Error(`unknown execution ledger event: ${event.type}`);
    }
  }
  return {header, known, states, completed};
}

export class ThemeExecutionLedger {
  constructor(filePath, events, now = () => new Date().toISOString()) {
    this.filePath = filePath;
    this.events = events;
    this.now = now;
    Object.assign(this, reduceEvents(events));
  }

  static async create(filePath, {runId, fingerprint, resources}, {now} = {}) {
    await fs.mkdir(path.dirname(filePath), {recursive: true});
    const header = {seq: 0, type: "header", schema: THEME_EXECUTION_LEDGER_SCHEMA, run_id: runId, fingerprint, resources, recorded_at: (now ?? (() => new Date().toISOString()))()};
    const handle = await fs.open(filePath, "wx", 0o600);
    try {
      await handle.writeFile(`${JSON.stringify(header)}\n`, "utf8");
      await handle.sync();
    } finally {
      await handle.close();
    }
    await syncParentDirectory(filePath);
    return new ThemeExecutionLedger(filePath, [header], now);
  }

  static async load(filePath, {now} = {}) {
    const [text, stat] = await Promise.all([fs.readFile(filePath, "utf8"), fs.stat(filePath)]);
    if ((stat.mode & 0o077) !== 0) throw new Error("execution ledger permissions must not allow group or other access");
    const {events, complete} = parseEvents(text);
    if (complete !== text) {
      const handle = await fs.open(filePath, "r+");
      try {
        await handle.truncate(Buffer.byteLength(complete));
        await handle.sync();
      } finally {
        await handle.close();
      }
    }
    return new ThemeExecutionLedger(filePath, events, now);
  }

  outstandingReverse() {
    return [...this.header.resources].reverse().filter((resource) => this.states.has(resource.resource_id) && this.states.get(resource.resource_id).phase !== "cleaned");
  }

  async append(event) {
    const full = {...event, seq: this.events.length, recorded_at: this.now()};
    const handle = await fs.open(this.filePath, "a", 0o600);
    try {
      await handle.writeFile(`${JSON.stringify(full)}\n`, "utf8");
      await handle.sync();
    } finally {
      await handle.close();
    }
    this.events.push(full);
    Object.assign(this, reduceEvents(this.events));
  }

  async intent(resource, expected) {
    await this.append({type: "intent", resource_id: resource.resource_id, expected});
  }

  async created(resource, identity) {
    await this.append({type: "created", resource_id: resource.resource_id, identity});
  }

  async cleaned(resource, reason) {
    await this.append({type: "cleaned", resource_id: resource.resource_id, reason});
  }

  async residual(resource, reason) {
    await this.append({type: "residual", resource_id: resource.resource_id, reason});
  }

  async complete() {
    if (this.completed) return;
    if (this.outstandingReverse().length) throw new Error("cannot complete execution ledger with residual resources");
    await this.append({type: "completed"});
  }
}

function adapterFor(adapters, resource) {
  const adapter = adapters[resource.target];
  if (!adapter || !["inspect", "create", "remove"].every((method) => typeof adapter[method] === "function")) {
    throw new Error(`missing execution adapter: ${resource.target}`);
  }
  return adapter;
}

function matchesExpected(actual, state, remoteSourceSha256) {
  if (actual.source_sha256 !== remoteSourceSha256) return false;
  if (actual.title !== state.expected.title) return false;
  return state.identity === undefined || actual.identity === state.identity;
}

async function expectedRemoteSourceSha256(resource, expected) {
  if (typeof expected.remote_source_sha256 === "string") return expected.remote_source_sha256;
  if (resource.target !== "wikidot") return expected.source_sha256;
  const stat = await fs.lstat(resource.source_path);
  if (!stat.isFile() || stat.isSymbolicLink()) throw new Error("accepted source is not a regular file during recovery");
  const source = await fs.readFile(resource.source_path, "utf8");
  if (sha256(source) !== expected.source_sha256) throw new Error("accepted source changed before recovery");
  return targetRoundTripSourceSha256(resource.target, source);
}

function throwIfAborted(signal) {
  if (signal?.aborted) throw signal.reason instanceof Error ? signal.reason : new Error("theme execution interrupted");
}

export async function cleanupThemeExecution({ledger, adapters}) {
  if (ledger.completed) return;
  const failures = [];
  for (const resource of ledger.outstandingReverse()) {
    const state = ledger.states.get(resource.resource_id);
    const adapter = adapterFor(adapters, resource);
    try {
      const actual = await adapter.inspect(resource);
      if (actual === null) {
        await ledger.cleaned(resource, "already_absent");
        continue;
      }
      const remoteSourceSha256 = await expectedRemoteSourceSha256(resource, state.expected);
      if (!matchesExpected(actual, state, remoteSourceSha256)) throw new Error("remote identity or source hash changed");
      const remoteExpected = {...state.expected, source_sha256: remoteSourceSha256};
      delete remoteExpected.remote_source_sha256;
      await adapter.remove(resource, {expected: remoteExpected, identity: state.identity});
      if (await adapter.inspect(resource) !== null) throw new Error("page remains after delete");
      await ledger.cleaned(resource, "deleted_and_verified_absent");
    } catch (error) {
      const reason = error instanceof Error ? error.message : String(error);
      await ledger.residual(resource, reason).catch(() => {});
      failures.push(new Error(`${resource.resource_id}: ${reason}`));
    }
  }
  if (failures.length) throw new AggregateError(failures, "theme execution cleanup left residual resources");
  await ledger.complete();
}

export async function executeThemeRunOwnedPages({plan, ledgerPath, adapters, materialize, capture, now, signal}) {
  const resources = validateThemeExecutionPlan(plan);
  if (typeof materialize !== "function" || typeof capture !== "function") throw new Error("materialize and capture callbacks are required");
  throwIfAborted(signal);
  for (const resource of resources) {
    if (await adapterFor(adapters, resource).inspect(resource) !== null) throw new Error(`preexisting page blocks execution: ${resource.resource_id}`);
    throwIfAborted(signal);
  }
  const ledger = await ThemeExecutionLedger.create(ledgerPath, {runId: plan.run.id, fingerprint: themeExecutionFingerprint(plan), resources}, {now});
  let primaryError = null;
  try {
    for (const tier of plan.tiers) {
      const tierResources = resources.filter((resource) => resource.tier_id === tier.id);
      for (const resource of tierResources) {
        throwIfAborted(signal);
        const payload = await materialize(resource);
        throwIfAborted(signal);
        if (typeof payload?.source !== "string" || sha256(payload.source) !== resource.source_sha256) throw new Error(`accepted source changed after preflight: ${resource.resource_id}`);
        const expected = {source_sha256: resource.source_sha256, remote_source_sha256: targetRoundTripSourceSha256(resource.target, payload.source), title: resource.title};
        await ledger.intent(resource, expected);
        const identity = await adapterFor(adapters, resource).create(resource, payload);
        throwIfAborted(signal);
        await ledger.created(resource, identity);
      }
      throwIfAborted(signal);
      await capture(tier, tierResources);
      throwIfAborted(signal);
    }
  } catch (error) {
    primaryError = error;
  }
  try {
    await cleanupThemeExecution({ledger, adapters});
  } catch (cleanupError) {
    if (primaryError) throw new AggregateError([primaryError, cleanupError], "theme execution and cleanup both failed");
    throw cleanupError;
  }
  if (primaryError) throw primaryError;
  return {status: "pass", resources_created: resources.length, resources_residual: 0, ledger_path: ledgerPath};
}

export async function recoverThemeExecution({ledgerPath, plan, adapters, now}) {
  const resources = validateThemeExecutionPlan(plan);
  const ledger = await ThemeExecutionLedger.load(ledgerPath, {now});
  if (ledger.header.run_id !== plan.run.id || ledger.header.fingerprint !== themeExecutionFingerprint(plan) || JSON.stringify(ledger.header.resources) !== JSON.stringify(resources)) {
    throw new Error("execution ledger does not match the requested plan");
  }
  await cleanupThemeExecution({ledger, adapters});
  return {status: "clean", resources_residual: 0, ledger_path: ledgerPath};
}
