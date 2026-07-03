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

test("imported page-local attachments are not repository seed fixtures", () => {
  const filesBySite = readJson("deepwell/seeder/files.json");
  for (const page of [
    "scp-3922",
    "scp-7243",
    "scp-8382",
    "scp-9506",
    "theme:basalt",
  ]) {
    assert.equal(filesBySite["scp-wiki"]?.[page], undefined);
  }
});

test("seed data includes SCP-7243 cross-page icon dependencies", () => {
  const filesBySite = readJson("deepwell/seeder/files.json");
  const expectedByPage = new Map([
    [
      "component:anomaly-class-bar",
      [
        "amida-icon.svg",
        "critical-icon.svg",
        "danger-icon.svg",
        "ekhi-icon.svg",
        "keter-icon.svg",
      ],
    ],
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

test("seed data includes SCP-7243 nav side chrome image dependencies", () => {
  const filesBySite = readJson("deepwell/seeder/files.json");
  const files = filesBySite["scp-wiki"]?.["nav:side"] ?? [];
  const expectedNames = [
    "black.png",
    "icon-Discord-2023.png",
    "social-bluesky.png",
    "social-facebook.png",
    "social-instagram.png",
    "social-reddit.png",
    "social-tiktok.png",
    "social-twitter.png",
  ];

  assert.deepEqual(files.map((file) => file.name).sort(), expectedNames);

  for (const file of files) {
    const seededPath = path.join(seederRoot, file.path);
    assert.ok(
      fs.existsSync(seededPath),
      `nav:side/${file.name} references missing ${seededPath}`,
    );
    assert.ok(
      fs.statSync(seededPath).size > 0,
      `nav:side/${file.name} should not be empty`,
    );
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
