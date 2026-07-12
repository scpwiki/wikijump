import assert from "node:assert/strict";
import crypto from "node:crypto";
import {EventEmitter} from "node:events";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {parseArgs} from "../scripts/theme-localization-e2e.mjs";
import {ALLOWED_SITE_SLUG, THEME_LOCALIZATION_E2E_SCHEMA, runOwnedSlug} from "../src/theme-localization-e2e.mjs";
import {ThemeExecutionLedger, themeExecutionFingerprint, validateRecoverableThemeExecutionPlan, validateThemeExecutionPlan} from "../src/theme-localization-execution.mjs";
import {GUARDED_THEME_WIKIJUMP_RPC_URL, THEME_RUN_RESULT_SCHEMA, createLiveThemeDependencies, runGuardedThemeAction, validateGuardedThemeRpcUrl, validateStorageState, validateThemeCdpEndpoint, writeExecutableThemePlan} from "../src/theme-localization-runner.mjs";
import {targetRoundTripSourceSha256} from "../src/theme-source-roundtrip.mjs";

const digest = (value) => crypto.createHash("sha256").update(value).digest("hex");

class MemoryAdapter {
  constructor(target, onCreate = null) {
    this.target = target;
    this.onCreate = onCreate;
    this.pages = new Map();
    this.nextId = 1;
  }

  async inspect(resource) { return this.pages.get(resource.slug) ?? null; }
  async create(resource, payload) {
    const page = {identity: this.nextId++, title: resource.title, source_sha256: targetRoundTripSourceSha256(resource.target, payload.source)};
    this.pages.set(resource.slug, page);
    this.onCreate?.();
    return page.identity;
  }
  async remove(resource) { this.pages.delete(resource.slug); }
}

async function fixture({onCreate, tierIds = ["yossistyle"]} = {}) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "theme-runner-"));
  const runId = "runner-test";
  const tiers = [];
  for (const [index, tierId] of tierIds.entries()) {
    const source = `日本語のテーマ本文 ${tierId}\n`;
    const sourcePath = path.join(root, `${tierId}.txt`);
    const slug = runOwnedSlug(runId, tierId);
    await fs.writeFile(sourcePath, source);
    tiers.push({
      id: tierId, order: index + 1, run_owned_slug: slug,
      preflight: {status: "pass", source: {absolute_path: sourcePath, sha256: digest(source)}},
      targets: [
        {id: "wikidot", resource_id: `${tierId}:wikidot`, origin: `http://${ALLOWED_SITE_SLUG}.wikidot.com`, url: `http://${ALLOWED_SITE_SLUG}.wikidot.com/${slug}`},
        {id: "wikijump", resource_id: `${tierId}:wikijump`, origin: `https://${ALLOWED_SITE_SLUG}.wikijump.localhost:18443`, url: `https://${ALLOWED_SITE_SLUG}.wikijump.localhost:18443/${slug}`},
      ],
      capture: {
        viewports: [{id: "desktop", width: 100, height: 100}],
        computed_styles: {properties: ["display"], probes: [{id: "header", selector: "#header", expectation: "required"}]},
      },
    });
  }
  const plan = {
    schema: THEME_LOCALIZATION_E2E_SCHEMA,
    mode: "execute",
    run: {id: runId, site_slug: ALLOWED_SITE_SLUG, owned_slug_prefix: `codex-l10n:${runId}-`},
    safety: {execute_supported: true, hard_allowlist: {site_slug: ALLOWED_SITE_SLUG, wikidot_hostname: `${ALLOWED_SITE_SLUG}.wikidot.com`, wikijump_hostname: `${ALLOWED_SITE_SLUG}.wikijump.localhost`}},
    preflight: {status: "pass"},
    tiers,
  };
  const adapters = {wikidot: new MemoryAdapter("wikidot", onCreate), wikijump: new MemoryAdapter("wikijump")};
  let closed = false;
  let closedAfterCleanup = false;
  const browserSession = {id: "shared-browser"};
  const dependencyFactory = async ({needsBrowser}) => ({adapters, secrets: ["swordfish-pass"], storageStates: {}, chromium: {}, browserSession: needsBrowser ? browserSession : null, async close() { closedAfterCleanup = adapters.wikidot.pages.size + adapters.wikijump.pages.size === 0; closed = true; }});
  return {root, plan, adapters, browserSession, dependencyFactory, closed: () => closed, closedAfterCleanup: () => closedAfterCleanup, ledgerPath: path.join(root, "ledger.jsonl"), resultPath: path.join(root, "result.json"), artifactDir: path.join(root, "artifacts")};
}

function legacyPlan(plan) {
  const legacy = structuredClone(plan);
  legacy.run.owned_slug_prefix = `theme:codex-l10n-${legacy.run.id}-`;
  for (const tier of legacy.tiers) {
    const slug = `theme:codex-l10n-${legacy.run.id}-${tier.id}`;
    tier.run_owned_slug = slug;
    for (const target of tier.targets) target.url = `${target.origin}/${slug}`;
  }
  return legacy;
}

function capture(status = "pass") {
  return async ({tier, source}) => ({tier_id: tier.id, status, targets: tier.targets.map((target) => ({id: target.id, verdict: {status, failed_viewports: status === "pass" ? [] : ["desktop"]}})), source_seen: source.length});
}

test("guarded execution captures the tier, cleans both targets, and persists a private aggregate", async () => {
  const fx = await fixture();
  const result = await runGuardedThemeAction({...fx, mode: "execute", captureTierImpl: capture()});
  assert.equal(result.schema, THEME_RUN_RESULT_SCHEMA);
  assert.equal(result.status, "pass");
  assert.deepEqual(result.captures[0].targets.map((target) => target.id), ["wikidot", "wikijump"]);
  assert.equal(fx.adapters.wikidot.pages.size + fx.adapters.wikijump.pages.size, 0);
  assert.equal((await ThemeExecutionLedger.load(fx.ledgerPath)).completed, true);
  assert.equal((await fs.stat(fx.resultPath)).mode & 0o077, 0);
  assert.equal(fx.closed(), true);
  assert.equal(fx.closedAfterCleanup(), true);
});

test("one browser session is reused across tiers and closed only after cleanup", async () => {
  const fx = await fixture({tierIds: ["yossistyle", "ashes-to-ashes"]});
  const seen = [];
  const result = await runGuardedThemeAction({...fx, mode: "execute", captureTierImpl: async (options) => {
    seen.push({tier: options.tier.id, session: options.browserSession});
    return capture()(options);
  }});
  assert.deepEqual(seen.map((item) => item.tier), ["yossistyle", "ashes-to-ashes"]);
  assert.ok(seen.every((item) => item.session === fx.browserSession));
  assert.equal(result.captures.length, 2);
  assert.equal(fx.closedAfterCleanup(), true);
});

test("live execution and recovery reject a dry-run plan before connecting adapters", async () => {
  const fx = await fixture();
  const executableFingerprint = themeExecutionFingerprint(fx.plan);
  fx.plan.mode = "dry-run";
  fx.plan.safety.execute_supported = false;
  assert.notEqual(themeExecutionFingerprint(fx.plan), executableFingerprint);
  let connected = false;
  fx.dependencyFactory = async () => { connected = true; };
  await assert.rejects(runGuardedThemeAction({...fx, mode: "execute"}), /not explicitly executable/);
  await assert.rejects(runGuardedThemeAction({...fx, mode: "recover"}), /not explicitly executable/);
  assert.equal(connected, false);
  await assert.rejects(fs.stat(fx.resultPath), /ENOENT/);
});

test("executable plan persistence is exclusive, durable, and private", async () => {
  const fx = await fixture();
  const planPath = path.join(fx.root, "plan.json");
  await writeExecutableThemePlan(planPath, fx.plan);
  assert.equal((await fs.stat(planPath)).mode & 0o077, 0);
  assert.equal(JSON.parse(await fs.readFile(planPath, "utf8")).mode, "execute");
  await assert.rejects(writeExecutableThemePlan(planPath, fx.plan), /EEXIST/);
  assert.equal(JSON.parse(await fs.readFile(planPath, "utf8")).run.id, fx.plan.run.id);
});

test("browser storage state denies group and other access", async () => {
  const fx = await fixture();
  const storageState = path.join(fx.root, "storage.json");
  await fs.writeFile(storageState, "{}", {mode: 0o600});
  assert.equal(await validateStorageState(storageState), storageState);
  await fs.chmod(storageState, 0o640);
  await assert.rejects(validateStorageState(storageState), /deny group and other access/);
});

test("CDP endpoint accepts only an uncredentialed loopback HTTP origin", () => {
  assert.equal(validateThemeCdpEndpoint("http://127.0.0.1:9222"), "http://127.0.0.1:9222");
  assert.equal(validateThemeCdpEndpoint("http://localhost:9333/"), "http://localhost:9333");
  for (const endpoint of ["https://127.0.0.1:9222", "http://192.168.1.2:9222", "http://user:pass@127.0.0.1:9222", "http://127.0.0.1:9222/json", "http://127.0.0.1"]) assert.throws(() => validateThemeCdpEndpoint(endpoint), /loopback HTTP origin/);
});

test("guarded runner requires the exact runtime50x Deepwell RPC binding", () => {
  assert.equal(validateGuardedThemeRpcUrl(GUARDED_THEME_WIKIJUMP_RPC_URL), GUARDED_THEME_WIKIJUMP_RPC_URL);
  for (const endpoint of [undefined, "", "http://127.0.0.1:2747/jsonrpc", "http://localhost:12747/jsonrpc", "http://127.0.0.1:12747/jsonrpc/"]) {
    assert.throws(() => validateGuardedThemeRpcUrl(endpoint), /must explicitly equal/);
  }
});

test("live dependency construction cannot fall back to another Deepwell stack", async () => {
  await assert.rejects(createLiveThemeDependencies({env: {}}), /WIKIJUMP_THEME_RPC_URL must explicitly equal/);
  await assert.rejects(createLiveThemeDependencies({env: {WIKIJUMP_THEME_RPC_URL: "http://127.0.0.1:2747/jsonrpc"}}), /WIKIJUMP_THEME_RPC_URL must explicitly equal/);
});

test("insecure artifact root is rejected before adapters connect", async () => {
  const fx = await fixture();
  await fs.mkdir(fx.artifactDir, {mode: 0o755});
  let connected = false;
  fx.dependencyFactory = async () => { connected = true; };
  await assert.rejects(runGuardedThemeAction({...fx, mode: "execute"}), /artifact directory permissions/);
  assert.equal(connected, false);
});

test("a strict capture failure remains primary after verified cleanup", async () => {
  const fx = await fixture();
  await assert.rejects(runGuardedThemeAction({...fx, mode: "execute", captureTierImpl: capture("fail")}), /strict browser verdict failed/);
  assert.equal(fx.adapters.wikidot.pages.size + fx.adapters.wikijump.pages.size, 0);
  const result = JSON.parse(await fs.readFile(fx.resultPath, "utf8"));
  assert.equal(result.status, "fail");
  assert.equal(result.captures[0].status, "fail");
});

test("recovery accepts only the matching fingerprint and removes an intent-fenced page", async () => {
  const fx = await fixture();
  let browserRequested = null;
  const dependencyFactory = fx.dependencyFactory;
  fx.dependencyFactory = async (options) => { browserRequested = options.needsBrowser; return dependencyFactory(options); };
  const resources = validateThemeExecutionPlan(fx.plan);
  const ledger = await ThemeExecutionLedger.create(fx.ledgerPath, {runId: fx.plan.run.id, fingerprint: themeExecutionFingerprint(fx.plan), resources});
  const resource = resources[0];
  const expected = {source_sha256: resource.source_sha256, remote_source_sha256: targetRoundTripSourceSha256(resource.target, await fs.readFile(resource.source_path, "utf8")), title: resource.title};
  await ledger.intent(resource, expected);
  fx.adapters.wikidot.pages.set(resource.slug, {identity: 42, title: expected.title, source_sha256: expected.remote_source_sha256});
  const result = await runGuardedThemeAction({...fx, mode: "recover"});
  assert.equal(result.operation.status, "clean");
  assert.equal(fx.adapters.wikidot.pages.size, 0);
  assert.equal(browserRequested, false);
});

test("runner recovery preserves sealed legacy theme-category ledgers without enabling legacy execution", async () => {
  const fx = await fixture();
  fx.plan = legacyPlan(fx.plan);
  const resources = validateRecoverableThemeExecutionPlan(fx.plan);
  const ledger = await ThemeExecutionLedger.create(fx.ledgerPath, {runId: fx.plan.run.id, fingerprint: themeExecutionFingerprint(fx.plan, {allowLegacy: true}), resources});
  await ledger.complete();

  const result = await runGuardedThemeAction({...fx, mode: "recover"});
  assert.equal(result.status, "pass");
  assert.equal(result.operation.status, "clean");
  assert.equal(result.plan_fingerprint, themeExecutionFingerprint(fx.plan, {allowLegacy: true}));

  const executeFx = await fixture();
  executeFx.plan = legacyPlan(executeFx.plan);
  let connected = false;
  executeFx.dependencyFactory = async () => { connected = true; };
  await assert.rejects(runGuardedThemeAction({...executeFx, mode: "execute"}), /slug prefix is invalid/);
  assert.equal(connected, false);
});

test("SIGINT aborts at an operation boundary without bypassing cleanup", async () => {
  const signals = new EventEmitter();
  const fx = await fixture({onCreate: () => signals.emit("SIGINT")});
  await assert.rejects(runGuardedThemeAction({...fx, mode: "execute", signalSource: signals, captureTierImpl: capture()}), (error) => error.signal === "SIGINT");
  assert.equal(fx.adapters.wikidot.pages.size + fx.adapters.wikijump.pages.size, 0);
  assert.equal(signals.listenerCount("SIGINT") + signals.listenerCount("SIGTERM"), 0);
  assert.equal(JSON.parse(await fs.readFile(fx.resultPath, "utf8")).signal, "SIGINT");
});

test("credential values are redacted from thrown and persisted errors", async () => {
  const fx = await fixture();
  fx.adapters.wikidot.inspect = async () => { throw new Error("remote rejected swordfish-pass"); };
  await assert.rejects(runGuardedThemeAction({...fx, mode: "execute", captureTierImpl: capture()}), (error) => !error.message.includes("swordfish-pass") && error.message.includes("[REDACTED]"));
  assert.doesNotMatch(await fs.readFile(fx.resultPath, "utf8"), /swordfish-pass/);
});

test("CLI requires one explicit action and never accepts credential flags", () => {
  const base = ["node", "theme", "--translation-root", "/tmp/translations", "--run-id", "runner-test", "--output", "/tmp/plan.json"];
  assert.throws(() => parseArgs(base), /exactly one/);
  assert.throws(() => parseArgs([...base, "--dry-run", "--execute"]), /exactly one/);
  assert.throws(() => parseArgs([...base, "--execute", "--password", "secret"]), /Unknown argument: --password/);
  assert.throws(() => parseArgs([...base, "--execute", "--browser-executable", "/chrome", "--cdp-endpoint", "http://127.0.0.1:9222"]), /cannot be combined/);
  assert.throws(() => parseArgs([...base, "--execute", "--cdp-endpoint", "http://example.com:9222"]), /loopback HTTP origin/);
  assert.throws(() => parseArgs(["node", "theme", "--recover", "--plan", "/tmp/plan", "--ledger", "/tmp/ledger"]), /--result/);
  assert.equal(parseArgs([...base, "--dry-run"]).mode, "dry-run");
});
