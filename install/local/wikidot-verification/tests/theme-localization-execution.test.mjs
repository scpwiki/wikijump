import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  ALLOWED_SITE_SLUG,
  THEME_LOCALIZATION_E2E_SCHEMA,
  runOwnedSlug,
} from "../src/theme-localization-e2e.mjs";
import {
  ThemeExecutionLedger,
  executeThemeRunOwnedPages,
  recoverThemeExecution,
  themeExecutionFingerprint,
  validateRecoverableThemeExecutionPlan,
  validateThemeExecutionPlan,
} from "../src/theme-localization-execution.mjs";
import {targetRoundTripSourceSha256} from "../src/theme-source-roundtrip.mjs";

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function fixturePlan({runId = "20260713-core", tiers = ["yossistyle", "ashes-to-ashes"]} = {}) {
  return {
    schema: THEME_LOCALIZATION_E2E_SCHEMA,
    run: {id: runId, site_slug: ALLOWED_SITE_SLUG, owned_slug_prefix: `codex-l10n:${runId}-`},
    safety: {
      hard_allowlist: {
        site_slug: ALLOWED_SITE_SLUG,
        wikidot_hostname: `${ALLOWED_SITE_SLUG}.wikidot.com`,
        wikijump_hostname: `${ALLOWED_SITE_SLUG}.wikijump.localhost`,
      },
    },
    preflight: {status: "pass"},
    tiers: tiers.map((id, index) => {
      const source = `日本語 source ${id}\n`;
      const slug = runOwnedSlug(runId, id);
      return {
        id,
        order: index + 1,
        run_owned_slug: slug,
        preflight: {status: "pass", source: {absolute_path: `/accepted/${id}.txt`, sha256: sha256(source)}},
        capture: {computed_styles: {properties: ["display"], probes: [{id: "header", selector: "#header", expectation: "required"}]}},
        targets: [
          {id: "wikidot", resource_id: `${id}:wikidot`, origin: `http://${ALLOWED_SITE_SLUG}.wikidot.com`, url: `http://${ALLOWED_SITE_SLUG}.wikidot.com/${slug}`},
          {id: "wikijump", resource_id: `${id}:wikijump`, origin: `https://${ALLOWED_SITE_SLUG}.wikijump.localhost:18443`, url: `https://${ALLOWED_SITE_SLUG}.wikijump.localhost:18443/${slug}`},
        ],
      };
    }),
  };
}

function legacyFixturePlan(options = {}) {
  const plan = fixturePlan(options);
  plan.run.owned_slug_prefix = `theme:codex-l10n-${plan.run.id}-`;
  for (const tier of plan.tiers) {
    const slug = `theme:codex-l10n-${plan.run.id}-${tier.id}`;
    tier.run_owned_slug = slug;
    for (const target of tier.targets) target.url = `${target.origin}/${slug}`;
  }
  return plan;
}

class FakeAdapter {
  constructor(target, events) {
    this.target = target;
    this.events = events;
    this.pages = new Map();
    this.nextId = 1;
  }

  async inspect(resource) {
    this.events.push(`inspect:${resource.resource_id}`);
    return this.pages.get(resource.slug) ?? null;
  }

  async create(resource, payload) {
    this.events.push(`create:${resource.resource_id}`);
    if (this.pages.has(resource.slug)) throw new Error("create-only collision");
    const identity = `${this.target}-${this.nextId++}`;
    this.pages.set(resource.slug, {identity, source_sha256: targetRoundTripSourceSha256(resource.target, payload.source), title: resource.title});
    return identity;
  }

  async remove(resource) {
    this.events.push(`remove:${resource.resource_id}`);
    this.pages.delete(resource.slug);
  }
}

async function fixture() {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "theme-execution-core-"));
  const plan = fixturePlan();
  const events = [];
  const adapters = {wikidot: new FakeAdapter("wikidot", events), wikijump: new FakeAdapter("wikijump", events)};
  const sources = new Map(plan.tiers.map((tier) => [tier.id, `日本語 source ${tier.id}\n`]));
  return {
    root,
    plan,
    events,
    adapters,
    ledgerPath: path.join(root, "creation-ledger.jsonl"),
    materialize: async (resource) => ({source: sources.get(resource.tier_id)}),
  };
}

test("execution plan accepts only run-owned resources on the corrected sandbox", () => {
  const plan = fixturePlan();
  assert.equal(validateThemeExecutionPlan(plan).length, 4);
  assert.equal(themeExecutionFingerprint(plan).length, 64);

  const wrongHost = structuredClone(plan);
  wrongHost.safety.hard_allowlist.wikidot_hostname = "scpaiueouiuiui.wikidot.com";
  assert.throws(() => validateThemeExecutionPlan(wrongHost), /hard allowlist/);

  const wrongSlug = structuredClone(plan);
  wrongSlug.tiers[0].run_owned_slug = "theme:yossistyle";
  assert.throws(() => validateThemeExecutionPlan(wrongSlug), /not owned by run/);

  const legacy = legacyFixturePlan({tiers: ["yossistyle"]});
  assert.throws(() => validateThemeExecutionPlan(legacy), /slug prefix is invalid/);
  assert.equal(validateRecoverableThemeExecutionPlan(legacy).length, 2);

  const mirrorUrl = structuredClone(plan);
  mirrorUrl.tiers[0].targets[1].url = `https://scp-wiki.wikijump.localhost/${mirrorUrl.tiers[0].run_owned_slug}`;
  assert.throws(() => validateThemeExecutionPlan(mirrorUrl), /outside the hard allowlist/);

  const invalidExpectation = structuredClone(plan);
  invalidExpectation.tiers[0].capture.computed_styles.probes[0].expectation = "sometimes";
  assert.throws(() => validateThemeExecutionPlan(invalidExpectation), /invalid expectation: sometimes/);
});

test("successful execution records intents before creates and cleans in reverse order", async () => {
  const fx = await fixture();
  const captured = [];
  const result = await executeThemeRunOwnedPages({...fx, capture: async (tier) => captured.push(tier.id), now: () => "2026-07-13T00:00:00.000Z"});

  assert.deepEqual(captured, ["yossistyle", "ashes-to-ashes"]);
  assert.equal(result.resources_created, 4);
  assert.equal([...fx.adapters.wikidot.pages, ...fx.adapters.wikijump.pages].length, 0);
  const ledger = await ThemeExecutionLedger.load(fx.ledgerPath);
  assert.equal(ledger.completed, true);
  assert.equal(ledger.outstandingReverse().length, 0);
  assert.deepEqual(fx.events.filter((event) => event.startsWith("remove:")), ["remove:ashes-to-ashes:wikijump", "remove:ashes-to-ashes:wikidot", "remove:yossistyle:wikijump", "remove:yossistyle:wikidot"]);
  assert.equal((await fs.stat(fx.ledgerPath)).mode & 0o077, 0);
});

test("global preexisting-page guard performs no creates and writes no ledger", async () => {
  const fx = await fixture();
  const blocked = validateThemeExecutionPlan(fx.plan).at(-1);
  fx.adapters.wikijump.pages.set(blocked.slug, {identity: "foreign", source_sha256: "foreign", title: "foreign"});

  await assert.rejects(executeThemeRunOwnedPages({...fx, capture: async () => {}}), /preexisting page blocks execution/);
  assert.equal(fx.events.some((event) => event.startsWith("create:")), false);
  await assert.rejects(fs.stat(fx.ledgerPath), /ENOENT/);
});

test("capture failure still removes every page and preserves the primary error", async () => {
  const fx = await fixture();
  await assert.rejects(
    executeThemeRunOwnedPages({...fx, capture: async () => { throw new Error("capture failed"); }}),
    /capture failed/,
  );
  assert.equal([...fx.adapters.wikidot.pages, ...fx.adapters.wikijump.pages].length, 0);
  assert.equal((await ThemeExecutionLedger.load(fx.ledgerPath)).completed, true);
});

test("accepted source hash is rechecked before the first remote create", async () => {
  const fx = await fixture();
  await assert.rejects(executeThemeRunOwnedPages({...fx, materialize: async () => ({source: "changed after preflight"}), capture: async () => {}}), /accepted source changed after preflight/);
  assert.equal(fx.events.some((event) => event.startsWith("create:")), false);
  assert.equal((await ThemeExecutionLedger.load(fx.ledgerPath)).completed, true);
});

test("changed remote content fails closed while cleanup still attempts other resources", async () => {
  const fx = await fixture();
  await assert.rejects(
    executeThemeRunOwnedPages({
      ...fx,
      capture: async (tier, resources) => {
        if (tier.id === "ashes-to-ashes") {
          const resource = resources[0];
          const current = fx.adapters.wikidot.pages.get(resource.slug);
          fx.adapters.wikidot.pages.set(resource.slug, {...current, source_sha256: "changed"});
        }
      },
    }),
    /cleanup left residual resources/,
  );
  const ledger = await ThemeExecutionLedger.load(fx.ledgerPath);
  assert.deepEqual(ledger.outstandingReverse().map((resource) => resource.resource_id), ["ashes-to-ashes:wikidot"]);
  assert.equal(fx.adapters.wikidot.pages.size, 1);
  assert.equal(fx.adapters.wikijump.pages.size, 0);
  assert.ok(fx.events.includes("remove:yossistyle:wikidot"));
});

test("recovery cleans a page created after a durable intent even with a partial last event", async () => {
  const fx = await fixture();
  fx.plan = fixturePlan({tiers: ["yossistyle"]});
  const resources = validateThemeExecutionPlan(fx.plan);
  const ledger = await ThemeExecutionLedger.create(fx.ledgerPath, {runId: fx.plan.run.id, fingerprint: themeExecutionFingerprint(fx.plan), resources});
  const resource = resources[0];
  const expected = {source_sha256: resource.source_sha256, remote_source_sha256: targetRoundTripSourceSha256(resource.target, `日本語 source ${resource.tier_id}\n`), title: resource.title};
  await ledger.intent(resource, expected);
  fx.adapters.wikidot.pages.set(resource.slug, {identity: "created-before-crash", title: expected.title, source_sha256: expected.remote_source_sha256});
  await fs.appendFile(fx.ledgerPath, "{partial", "utf8");

  const result = await recoverThemeExecution({ledgerPath: fx.ledgerPath, plan: fx.plan, adapters: fx.adapters});
  assert.equal(result.status, "clean");
  assert.equal(fx.adapters.wikidot.pages.size, 0);
  assert.equal((await ThemeExecutionLedger.load(fx.ledgerPath)).completed, true);
  assert.equal((await recoverThemeExecution({ledgerPath: fx.ledgerPath, plan: fx.plan, adapters: fx.adapters})).status, "clean");
});

test("recovery alone accepts an exact legacy theme-category plan and remains idempotent after sealing", async () => {
  const fx = await fixture();
  fx.plan = legacyFixturePlan({tiers: ["yossistyle"]});
  const resources = validateRecoverableThemeExecutionPlan(fx.plan);
  const ledger = await ThemeExecutionLedger.create(fx.ledgerPath, {runId: fx.plan.run.id, fingerprint: themeExecutionFingerprint(fx.plan, {allowLegacy: true}), resources});
  const resource = resources[0];
  const source = `日本語 source ${resource.tier_id}\n`;
  const expected = {source_sha256: resource.source_sha256, remote_source_sha256: targetRoundTripSourceSha256(resource.target, source), title: resource.title};
  await ledger.intent(resource, expected);
  fx.adapters.wikidot.pages.set(resource.slug, {identity: "legacy-created-before-crash", title: expected.title, source_sha256: expected.remote_source_sha256});

  assert.equal((await recoverThemeExecution({ledgerPath: fx.ledgerPath, plan: fx.plan, adapters: fx.adapters})).status, "clean");
  assert.equal(fx.adapters.wikidot.pages.size, 0);
  assert.equal((await ThemeExecutionLedger.load(fx.ledgerPath)).completed, true);
  assert.equal((await recoverThemeExecution({ledgerPath: fx.ledgerPath, plan: fx.plan, adapters: fx.adapters})).status, "clean");
});

test("legacy intent derives the narrow Wikidot terminal-LF round-trip hash from an unchanged accepted source", async () => {
  const fx = await fixture();
  fx.plan = fixturePlan({tiers: ["yossistyle"]});
  const source = "日本語 source yossistyle\n";
  const sourcePath = path.join(fx.root, "accepted.wikidot.txt");
  await fs.writeFile(sourcePath, source);
  fx.plan.tiers[0].preflight.source.absolute_path = sourcePath;
  const resources = validateThemeExecutionPlan(fx.plan);
  const ledger = await ThemeExecutionLedger.create(fx.ledgerPath, {runId: fx.plan.run.id, fingerprint: themeExecutionFingerprint(fx.plan), resources});
  const resource = resources[0];
  const expected = {source_sha256: resource.source_sha256, title: resource.title};
  await ledger.intent(resource, expected);
  fx.adapters.wikidot.pages.set(resource.slug, {identity: "saved-before-crash", title: expected.title, source_sha256: sha256(source.slice(0, -1))});

  await recoverThemeExecution({ledgerPath: fx.ledgerPath, plan: fx.plan, adapters: fx.adapters});
  assert.equal(fx.adapters.wikidot.pages.size, 0);
  assert.equal((await ThemeExecutionLedger.load(fx.ledgerPath)).completed, true);
});

test("recovery refuses a ledger from another plan", async () => {
  const fx = await fixture();
  const resources = validateThemeExecutionPlan(fx.plan);
  await ThemeExecutionLedger.create(fx.ledgerPath, {runId: fx.plan.run.id, fingerprint: themeExecutionFingerprint(fx.plan), resources});
  const otherPlan = fixturePlan({runId: "20260713-other"});
  await assert.rejects(recoverThemeExecution({ledgerPath: fx.ledgerPath, plan: otherPlan, adapters: fx.adapters}), /does not match the requested plan/);
});
