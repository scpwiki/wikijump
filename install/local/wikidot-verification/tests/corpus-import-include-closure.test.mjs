import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  buildCorpusImportManifest,
  buildManifestSummary,
  formatJsonl,
  summarizeIncludeClosure,
} from "../src/corpus-import-manifest.mjs";
import {
  expandIncludeClosure,
  includeTargetSlug,
  parseIncludeTargets,
} from "../src/corpus-include-closure.mjs";
import { parseArgs } from "../scripts/build-corpus-import-manifest.mjs";
import { writePage } from "./support/corpus-import-manifest-fixture.mjs";

const sourceReader = (sources) => ({
  readSource: (slug) => (slug in sources ? sources[slug] : null),
  pageExists: (slug) => slug in sources,
});

test("includeTargetSlug matches the drift reporter contract", () => {
  assert.equal(includeTargetSlug(":scp-wiki:theme:basalt", "scp-wiki"), "theme:basalt");
  assert.equal(includeTargetSlug(":SCP-WIKI:Component:Preview", "scp-wiki"), "component:preview");
  assert.equal(includeTargetSlug("component:license-box", "scp-wiki"), "component:license-box");
  assert.equal(includeTargetSlug(":scp-jp:theme:x", "scp-wiki"), null);
  assert.equal(includeTargetSlug("{$theme}", "scp-wiki"), null);
  assert.equal(includeTargetSlug("component:license-box@@", "scp-wiki"), null);
});

test("parseIncludeTargets separates same-site, external, and dynamic targets", () => {
  const targets = parseIncludeTargets(
    [
      "[[include info:start]]",
      "[[include :scp-wiki:info:more]]",
      "[[include :scpsandbox2:hippo]]",
      "[[include :SCP-WIKI:Info:End]]",
      "[[include {$theme}]]",
      "@@[[include component:documented]]@@",
      "[!-- [[include component:hidden]] --]",
    ].join("\n"),
    "scp-wiki",
  );
  assert.deepEqual(targets.sameSite, ["info:start", "info:more", "info:end"]);
  assert.deepEqual(targets.external, ["scpsandbox2:hippo"]);
  assert.equal(targets.dynamic, 1);
});

test("expandIncludeClosure walks transitively, roots first, once per page", () => {
  const sources = {
    "scp-1": "[[include :scp-wiki:theme:basalt]]\n[[include component:a |x=1]]",
    "theme:basalt": "[[include component:interwiki-style]]\n@@[[include component:a@@]]",
    "component:a": "[[include :scp-wiki:component:b]]\n[[include theme:basalt]]",
    "component:b": "plain",
    "component:interwiki-style": "[[include :other-site:component:c]]",
  };
  const closure = expandIncludeClosure({ roots: ["scp-1"], ...sourceReader(sources), sourceSite: "scp-wiki" });
  assert.deepEqual(closure.included.map((page) => page.slug), [
    "scp-1",
    "theme:basalt",
    "component:a",
    "component:interwiki-style",
    "component:b",
  ]);
  assert.deepEqual(closure.included.map((page) => page.depth), [0, 1, 1, 2, 2]);
  assert.equal(closure.included[1].via, "scp-1");
  assert.deepEqual(closure.unresolved, []);
  assert.deepEqual(closure.external, ["other-site:component:c"]);
  assert.equal(closure.dynamic, 0);
  assert.deepEqual(closure.truncated, []);
});

test("expandIncludeClosure reports unresolved and dynamic targets without failing", () => {
  const closure = expandIncludeClosure({
    roots: ["scp-1"],
    ...sourceReader({
      "scp-1": "[[include component:gone]]\n[[include {$name}]]\n[[include :scp-wiki:component:gone]]",
    }),
    sourceSite: "scp-wiki",
  });
  assert.deepEqual(closure.included.map((page) => page.slug), ["scp-1"]);
  assert.deepEqual(closure.unresolved, ["component:gone"]);
  assert.equal(closure.dynamic, 1);
});

test("expandIncludeClosure truncates past maxDepth and bounds total pages", () => {
  const sources = {
    "scp-1": "[[include component:a]]",
    "component:a": "[[include component:b]]",
    "component:b": "plain",
  };
  const shallow = expandIncludeClosure({
    roots: ["scp-1"],
    ...sourceReader(sources),
    sourceSite: "scp-wiki",
    maxDepth: 1,
  });
  assert.deepEqual(shallow.included.map((page) => page.slug), ["scp-1", "component:a"]);
  assert.deepEqual(shallow.truncated, ["component:b"]);

  assert.throws(
    () => expandIncludeClosure({ roots: ["scp-1"], ...sourceReader(sources), sourceSite: "scp-wiki", maxPages: 1 }),
    /maxPages/u,
  );
  assert.throws(
    () => expandIncludeClosure({ roots: [], ...sourceReader(sources), sourceSite: "scp-wiki" }),
    /at least one root/u,
  );
});

function writeClosureCorpus() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "corpus-closure-"));
  const pages = {
    "scp-3352": "[[include :scp-wiki:info:start]]\n[[include :scpsandbox2:hippo]]\nbody",
    "info:start": "[[include info:more]]\nstart",
    "info:more": "more",
  };
  let counter = 0;
  for (const [fullname, source] of Object.entries(pages)) {
    counter += 1;
    writePage(root, "en", fullname, {
      entityId: `00000000-0000-4000-8000-${String(counter).padStart(12, "0")}`,
      source,
    });
  }
  return root;
}

test("buildCorpusImportManifest expands the include closure and annotates rows", () => {
  const root = writeClosureCorpus();
  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: "en",
    sourceSite: "scp-wiki",
    sourceBranch: "en",
    fullnames: ["scp-3352"],
    includeClosure: {},
  });
  assert.deepEqual(rows.map((row) => row.fullname), ["info:more", "info:start", "scp-3352"]);
  const byName = new Map(rows.map((row) => [row.fullname, row]));
  assert.equal(byName.get("scp-3352").closure_depth, 0);
  assert.equal(byName.get("scp-3352").closure_via, null);
  assert.equal(byName.get("info:start").closure_depth, 1);
  assert.equal(byName.get("info:start").closure_via, "scp-3352");
  assert.equal(byName.get("info:more").closure_depth, 2);
  assert.equal(byName.get("info:more").closure_via, "info:start");
  const summary = summarizeIncludeClosure(
    expandIncludeClosure({
      roots: ["scp-3352"],
      readSource: (slug) => {
        const sourcePath = path.join(root, "en", "pages", slug, "source.wikidot.txt");
        return fs.existsSync(sourcePath) ? fs.readFileSync(sourcePath, "utf8") : null;
      },
      pageExists: (slug) => fs.existsSync(path.join(root, "en", "pages", slug, "source.wikidot.txt")),
      sourceSite: "scp-wiki",
    }),
  );
  assert.equal(summary.included_count, 3);
  assert.equal(summary.added_count, 2);
  assert.equal(summary.max_depth, 2);
  assert.deepEqual(summary.unresolved, []);
  assert.deepEqual(summary.external, ["scpsandbox2:hippo"]);
  const manifestSummary = buildManifestSummary(rows, formatJsonl(rows));
  assert.equal(manifestSummary.row_count, 3);
});

test("buildCorpusImportManifest fails closed on unresolved closure targets", () => {
  const root = writeClosureCorpus();
  writePage(root, "en", "scp-1", {
    entityId: "00000000-0000-4000-8000-000000000009",
    source: "[[include component:gone]]",
  });
  assert.throws(
    () => buildCorpusImportManifest({
      corpusRoot: root,
      branch: "en",
      sourceSite: "scp-wiki",
      fullnames: ["scp-1"],
      includeClosure: {},
    }),
    /component:gone/u,
  );
  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: "en",
    sourceSite: "scp-wiki",
    fullnames: ["scp-1"],
    includeClosure: { allowUnresolved: true },
  });
  assert.deepEqual(rows.map((row) => row.fullname), ["scp-1"]);
});

test("buildCorpusImportManifest rejects closure without explicit roots", () => {
  const root = writeClosureCorpus();
  assert.throws(
    () => buildCorpusImportManifest({ corpusRoot: root, branch: "en", includeClosure: {} }),
    /explicit fullnames/u,
  );
});

test("manifest CLI parses closure flags and guards their scope", () => {
  const args = parseArgs([
    "--corpus-root",
    "/corpus",
    "--fullname",
    "scp-3352",
    "--include-closure",
    "--closure-max-depth",
    "4",
    "--output",
    "/tmp/manifest.jsonl",
    "--summary",
    "/tmp/summary.json",
  ]);
  assert.equal(args.includeClosure, true);
  assert.equal(args.closureMaxDepth, 4);
  assert.throws(
    () => parseArgs(["--source-bundle", "/bundle", "--include-closure", "--output", "o", "--summary", "s"]),
    /--corpus-root/u,
  );
  assert.throws(
    () => parseArgs(["--corpus-root", "/corpus", "--include-closure", "--output", "o", "--summary", "s"]),
    /--fullname/u,
  );
  assert.throws(
    () => parseArgs(["--corpus-root", "/corpus", "--allow-unresolved-closure", "--output", "o", "--summary", "s"]),
    /require --include-closure/u,
  );
});
