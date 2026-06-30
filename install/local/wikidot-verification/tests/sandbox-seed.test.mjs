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
  const site = sites.find((candidate) => candidate.slug === "sandbox-for-codex");

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
    "system:recent-changes"
  ]);

  for (const page of pages) {
    const wikitextPath = path.join(seederRoot, `${page.wikitext}.ftml`);
    assert.ok(fs.existsSync(wikitextPath), `${page.slug} references missing ${wikitextPath}`);
  }
});

test("seeded mirror authors are non-login identities", () => {
  const users = readJson("deepwell/seeder/users.json");
  const mirrorAuthorNames = new Set(["SeekGull", "daveyoufool"]);
  const mirrorAuthors = users.filter((user) => mirrorAuthorNames.has(user.name));

  assert.equal(mirrorAuthors.length, mirrorAuthorNames.size);
  for (const user of mirrorAuthors) {
    assert.equal(user.type, "system");
    assert.equal(user.password, null);
    assert.deepEqual(user.locales, []);
  }
});
