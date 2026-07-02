import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "../../../..");
const seederRoot = path.join(repoRoot, "deepwell", "seeder");

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, relativePath), "utf8"));
}

test("seed data includes local sandbox-for-codex site required by parity probes", () => {
  const sites = readJson("deepwell/seeder/sites.json");
  const pagesBySite = readJson("deepwell/seeder/pages.json");
  const site = sites.find(
    (candidate) => candidate.slug === "sandbox-for-codex",
  );

  assert.ok(site, "sandbox-for-codex seed site should exist");
  assert.equal(site.name, "Sandbox for Codex");
  assert.equal(site["default-page"], "start");
  assert.equal(site.layout, "wikidot");

  const pages = pagesBySite["sandbox-for-codex"] ?? [];
  const pageSlugs = pages.map((page) => page.slug).sort();

  assert.deepEqual(pageSlugs, [
    "_admin",
    "nav:side",
    "nav:top",
    "start",
    "system:join",
    "system:members",
    "system:page-tags",
    "system:recent-changes",
  ]);

  for (const page of pages) {
    const wikitextPath = path.join(seederRoot, `${page.wikitext}.ftml`);
    assert.ok(
      fs.existsSync(wikitextPath),
      `${page.slug} references missing ${wikitextPath}`,
    );
  }
});

test("seed data names the EN mirror like live SCP Wiki chrome", () => {
  const sites = readJson("deepwell/seeder/sites.json");
  const site = sites.find((candidate) => candidate.slug === "scp-wiki");

  assert.ok(site, "scp-wiki seed site should exist");
  assert.equal(site.name, "SCP Foundation");
  assert.equal(site.tagline, "Secure, Contain, Protect");
  assert.equal(site["default-page"], "main");
});

test("seed data includes SCP-7243 page-local files used by browser parity canary", () => {
  const filesBySite = readJson("deepwell/seeder/files.json");
  const files = filesBySite["scp-wiki"]?.["scp-7243"] ?? [];
  const expectedNames = [
    "5y46584875-1.webp",
    "ARsterisk.png",
    "Anaximander.png",
    "DT.jpg",
    "DeeringsBW.jpg",
    "Digamma.png",
    "PhilStock.jpg",
    "Verne.jpg",
    "absentia-icon-3.svg",
    "admo-7243-abatement.jpg",
    "admo-7243-amy-abstract.jpg",
    "admo-7243-amy-cafeteria.jpg",
    "admo-7243-amy-ceiling.jpg",
    "admo-7243-amy-class.jpg",
    "admo-7243-amy-lights.jpg",
    "admo-7243-amy-sky.jpg",
    "admo-7243-amy-subway.jpg",
    "admo-7243-amy-truss.jpg",
    "admo-7243-amy-warehouse.jpg",
    "admo-7243-bubble-evenmoredone.jpg",
    "admo-7243-exactus.jpg",
    "admo-7243-oracle.png",
    "admo-7243-resurgence.jpg",
    "bluepint2.jpg",
    "chamber.jpg",
    "fucksplosion-4.webp",
    "metaamida-icon.svg",
    "paradoxysm-icon.svg",
    "timecrash.jpg",
  ];

  assert.deepEqual(files.map((file) => file.name).sort(), expectedNames);

  for (const file of files) {
    const seededPath = path.join(seederRoot, file.path);
    assert.ok(
      fs.existsSync(seededPath),
      `${file.name} references missing ${seededPath}`,
    );
    assert.ok(
      fs.statSync(seededPath).size > 0,
      `${file.name} should not be empty`,
    );
  }
});

test("seed data includes SCP-7243 cross-page icon dependencies", () => {
  const filesBySite = readJson("deepwell/seeder/files.json");
  const expectedByPage = new Map([
    ["component:anomaly-class-bar", ["danger-icon.svg", "ekhi-icon.svg"]],
    ["scp-5382", ["thaumiel-icon-2.svg"]],
    ["scp-7000", ["inimical-icon.svg"]],
  ]);

  for (const [page, expectedNames] of expectedByPage) {
    const files = filesBySite["scp-wiki"]?.[page] ?? [];
    assert.deepEqual(files.map((file) => file.name).sort(), expectedNames);

    for (const file of files) {
      const seededPath = path.join(seederRoot, file.path);
      assert.ok(
        fs.existsSync(seededPath),
        `${page}/${file.name} references missing ${seededPath}`,
      );
      assert.ok(
        fs.statSync(seededPath).size > 0,
        `${page}/${file.name} should not be empty`,
      );
    }
  }
});

test("seeded mirror authors are non-login identities", () => {
  const users = readJson("deepwell/seeder/users.json");
  const mirrorAuthorNames = new Set(["SeekGull", "daveyoufool"]);
  const mirrorAuthors = users.filter((user) =>
    mirrorAuthorNames.has(user.name),
  );

  assert.equal(mirrorAuthors.length, mirrorAuthorNames.size);
  for (const user of mirrorAuthors) {
    assert.equal(user.type, "system");
    assert.equal(user.password, null);
    assert.deepEqual(user.locales, []);
  }
});
