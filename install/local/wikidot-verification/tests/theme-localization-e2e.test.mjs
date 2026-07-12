import assert from "node:assert/strict";
import {execFile} from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {fileURLToPath} from "node:url";
import {promisify} from "node:util";

import {
  ALLOWED_SITE_SLUG,
  THEME_CAPTURE_VIEWPORTS,
  THEME_LOCALIZATION_TIERS,
  THEME_PERFORMANCE_GATES,
  assertRunOwnedSlug,
  buildThemeLocalizationE2EPlan,
  findSourceArtifactLeaks,
  inventoryThemeSource,
  runOwnedSlug,
  selectThemeTiers,
  validateTargetOrigin,
} from "../src/theme-localization-e2e.mjs";

const execFileAsync = promisify(execFile);
const here = path.dirname(fileURLToPath(import.meta.url));
const cli = path.resolve(here, "../scripts/theme-localization-e2e.mjs");

async function fixtureTranslationRoot() {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-theme-e2e-"));
  for (const tier of THEME_LOCALIZATION_TIERS) {
    const sourcePath = path.join(root, tier.accepted_source);
    await fs.mkdir(path.dirname(sourcePath), {recursive: true});
    await fs.writeFile(sourcePath, acceptedFixtureSource(tier), "utf8");
    for (const component of tier.dependencies.components) {
      const componentPath = path.join(root, "corpus", "jp", "pages", component, "source.wikidot.txt");
      await fs.mkdir(path.dirname(componentPath), {recursive: true});
      await fs.writeFile(componentPath, `日本語 component ${component}\n`, "utf8");
    }
    for (const asset of tier.dependencies.assets) {
      const assetPath = path.join(root, "corpus", "en", "pages", tier.article_slug, "files", asset);
      await fs.mkdir(path.dirname(assetPath), {recursive: true});
      await fs.writeFile(assetPath, `asset:${tier.id}:${asset}\n`, "utf8");
    }
  }
  return root;
}

function acceptedFixtureSource(tier) {
  const lines = ["日本語のテーマ記事です。", ...tier.required_markers];
  for (let index = 0; index < (tier.minimum_shape.css_modules ?? 0); index += 1) lines.push("[[module CSS]]", `.fixture-${index} { color: red; }`, "[[/module]]");
  for (let index = 0; index < (tier.minimum_shape.code_blocks ?? 0); index += 1) lines.push("[[code type=\"css\"]]", `.code-${index} { display: block; }`, "[[/code]]");
  for (let index = 0; index < (tier.minimum_shape.executable_includes ?? 0); index += 1) lines.push(`[[include component:fixture-${index}]]`);
  for (let index = 0; index < (tier.minimum_shape.local_resource_references ?? 0); index += 1) lines.push(`https://example.invalid/local--files/theme:fixture/asset-${index}.png`);
  while (lines.length < (tier.minimum_shape.logical_lines ?? 1) + 1) lines.push(`日本語フィラー ${lines.length}`);
  while (Buffer.byteLength(lines.join("\n")) < (tier.minimum_shape.bytes ?? 1) + 100) lines.push("日本語の決定的なフィラー行です。".repeat(8));
  return `${lines.join("\n")}\n`;
}

test("tier selection is deterministic and run-owned slugs cannot drift", () => {
  assert.deepEqual(selectThemeTiers(["basalt", "yossistyle", "basalt"]).map((tier) => tier.id), ["yossistyle", "basalt"]);
  assert.equal(runOwnedSlug("20260713-smoke", "basalt"), "theme:codex-l10n-20260713-smoke-basalt");
  assert.equal(assertRunOwnedSlug("theme:codex-l10n-20260713-smoke-basalt", "20260713-smoke", "basalt"), "theme:codex-l10n-20260713-smoke-basalt");
  assert.throws(() => assertRunOwnedSlug("theme:basalt", "20260713-smoke", "basalt"), /not owned by run/);
  assert.throws(() => runOwnedSlug("../../escape", "basalt"), /--run-id/);
  assert.throws(() => selectThemeTiers(["unknown"]), /unknown theme tier/);
});

test("target allowlist rejects mirror sites, paths, credentials, and wrong protocols", () => {
  assert.equal(validateTargetOrigin("http://scpaiueouiuiuiui.wikidot.com", "wikidot"), "http://scpaiueouiuiuiui.wikidot.com");
  assert.equal(validateTargetOrigin("https://scpaiueouiuiuiui.wikijump.localhost:18443", "wikijump"), "https://scpaiueouiuiuiui.wikijump.localhost:18443");
  assert.throws(() => validateTargetOrigin("http://scpaiueouiuiui.wikidot.com", "wikidot"), /hard allowlist/);
  assert.throws(() => validateTargetOrigin("https://scpaiueouiuiui.wikijump.localhost:18443", "wikijump"), /hard allowlist/);
  assert.throws(() => validateTargetOrigin("https://scp-wiki.wikijump.localhost", "wikijump"), /hard allowlist/);
  assert.throws(() => validateTargetOrigin("https://scp-jp.wikijump.localhost", "wikijump"), /hard allowlist/);
  assert.throws(() => validateTargetOrigin("http://scpaiueouiuiuiui.wikidot.com/admin", "wikidot"), /hard allowlist/);
  assert.throws(() => validateTargetOrigin("https://scpaiueouiuiuiui.wikidot.com", "wikidot"), /hard allowlist/);
  assert.throws(() => validateTargetOrigin("https://user:pass@scpaiueouiuiuiui.wikijump.localhost", "wikijump"), /hard allowlist/);
  assert.throws(() => validateTargetOrigin("http://scpaiueouiuiuiui.wikidot.com:9999", "wikidot"), /hard allowlist/);
  assert.throws(() => validateTargetOrigin("https://scpaiueouiuiuiui.wikijump.localhost:2747", "wikijump"), /hard allowlist/);
  assert.throws(() => validateTargetOrigin("https://scpaiueouiuiuiui.wikijump.localhost", "other"), /unknown target/);
});

test("artifact leakage reports locations without copying identifiers", () => {
  const leakedId = "019de38f-db2e-76d1-b375-145c28c0ea8b";
  const source = `正常な行\ntext thread_id=${leakedId}\n[/ thread_id=${leakedId}\n/home/roku/secret/file\nrun_id=worker-20260713`;
  const findings = findSourceArtifactLeaks(source);
  assert.deepEqual(findings.map((finding) => finding.id), ["thread_id", "thread_id", "local_absolute_path", "run_id"]);
  assert.deepEqual(findings.map((finding) => finding.line), [2, 3, 4, 5]);
  assert.ok(!JSON.stringify(findings).includes(leakedId));
  assert.deepEqual(findSourceArtifactLeaks("https://scp-wiki.wikidot.com/local--files/theme:x/a.png\n[[include]]"), []);
});

test("source inventory distinguishes executable includes from prose", () => {
  const inventory = inventoryThemeSource("日本語\n[[include]]\n[[include component:x]]\n[[module CSS]]\ncontent: 'x';\n[[/module]]\n[[code]]x[[/code]]\n/local--files/theme:x/a.png\n");
  assert.deepEqual(inventory, {
    bytes: 134,
    logical_lines: 8,
    css_modules: 1,
    code_blocks: 1,
    executable_includes: 1,
    local_resource_references: 1,
    css_content_declarations: 1,
  });
});

test("plan is deterministic, mutation-free, and carries cleanup and capture contracts", async () => {
  const translationRoot = await fixtureTranslationRoot();
  const options = {translationRoot, runId: "20260713-contract", tiers: ["all"]};
  const first = await buildThemeLocalizationE2EPlan(options);
  const second = await buildThemeLocalizationE2EPlan(options);

  assert.deepEqual(first, second);
  assert.equal(first.preflight.status, "pass");
  assert.equal(first.safety.page_mutations_performed, 0);
  assert.equal(first.safety.execute_supported, false);
  assert.equal(first.run.site_slug, ALLOWED_SITE_SLUG);
  assert.deepEqual(first.tiers.map((tier) => tier.id), ["yossistyle", "ashes-to-ashes", "basalt"]);
  assert.equal(first.cleanup.finally_required, true);
  assert.equal(first.cleanup.creation_ledger_required, true);
  assert.equal(first.cleanup.resources.length, 6);
  assert.equal(first.cleanup.resources[0].resource_id, "basalt:wikijump");
  assert.ok(first.cleanup.resources.every((resource) => resource.preexisting_policy === "abort_before_write"));
  assert.deepEqual(first.tiers[0].capture.viewports, THEME_CAPTURE_VIEWPORTS);
  assert.deepEqual(first.tiers[0].capture.web_vitals.gates, THEME_PERFORMANCE_GATES);
  assert.ok(first.tiers[0].capture.computed_styles.probes.some((probe) => probe.pseudo === "::after"));
  assert.ok(first.tiers[2].capture.interactions.some((interaction) => interaction.id === "tab_switch"));
  assert.ok(first.tiers.flatMap((tier) => tier.targets).every((target) => target.url.includes("scpaiueouiuiuiui")));
  assert.ok(first.tiers.flatMap((tier) => tier.preflight.dependency_files.assets).every((asset) => asset.status === "pass" && asset.sha256));

  const executable = await buildThemeLocalizationE2EPlan({...options, mode: "execute"});
  assert.equal(executable.mode, "execute");
  assert.equal(executable.safety.execute_supported, true);
  assert.equal(first.mode, "dry-run");
});

test("dependency preflight fails closed when an attachment is absent", async () => {
  const translationRoot = await fixtureTranslationRoot();
  const tier = THEME_LOCALIZATION_TIERS.find((candidate) => candidate.id === "ashes-to-ashes");
  await fs.unlink(path.join(translationRoot, "corpus", "en", "pages", tier.article_slug, "files", tier.dependencies.assets[0]));
  const plan = await buildThemeLocalizationE2EPlan({translationRoot, runId: "20260713-missing", tiers: [tier.id]});
  assert.equal(plan.preflight.status, "fail");
  assert.equal(plan.tiers[0].preflight.dependency_files.assets[0].status, "fail");
});

test("accepted source preflight fails closed on artifact leakage", async () => {
  const translationRoot = await fixtureTranslationRoot();
  const tier = THEME_LOCALIZATION_TIERS[0];
  const sourcePath = path.join(translationRoot, tier.accepted_source);
  await fs.appendFile(sourcePath, "thread_id=019de38f-db2e-76d1-b375-145c28c0ea8b\n", "utf8");
  const plan = await buildThemeLocalizationE2EPlan({translationRoot, runId: "20260713-leak", tiers: [tier.id]});
  assert.equal(plan.preflight.status, "fail");
  const leakage = plan.tiers[0].preflight.checks.find((check) => check.id === "artifact_leakage");
  assert.equal(leakage.status, "fail");
  assert.deepEqual(leakage.findings.map((finding) => finding.id), ["thread_id"]);
});

test("CLI requires dry-run and writes a passing deterministic plan", async () => {
  const translationRoot = await fixtureTranslationRoot();
  const output = path.join(translationRoot, "artifacts", "theme-plan.json");
  const args = [cli, "--dry-run", "--translation-root", translationRoot, "--run-id", "20260713-cli", "--output", output, "--json"];
  const result = await execFileAsync(process.execPath, args);
  const summary = JSON.parse(result.stdout);
  const plan = JSON.parse(await fs.readFile(output, "utf8"));
  assert.equal(summary.preflight.status, "pass");
  assert.equal(summary.page_mutations_performed, 0);
  assert.equal(plan.mode, "dry-run");

  await assert.rejects(execFileAsync(process.execPath, [cli, "--translation-root", translationRoot, "--run-id", "20260713-cli", "--output", output]), /exactly one of/);
  await assert.rejects(execFileAsync(process.execPath, [cli, "--execute", "--translation-root", translationRoot, "--run-id", "20260713-cli", "--output", output]), /--execute requires/);
  await assert.rejects(execFileAsync(process.execPath, [cli, "--dry-run", "--translation-root", translationRoot, "--run-id", "20260713-cli", "--output", output, "--site", "scp-wiki"]), /site is not allowlisted/);
});
