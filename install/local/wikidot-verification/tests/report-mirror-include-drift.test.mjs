import assert from "node:assert/strict";
import crypto from "node:crypto";
import test from "node:test";

import {
  buildDriftQuery,
  compareDriftRows,
  includeClosure,
  includeTargetSlug,
  parseArgs,
} from "../scripts/report-mirror-include-drift.mjs";

const sha = (text) => crypto.createHash("sha256").update(text).digest("hex");

test("normalizes only same-site, variable-free include targets", () => {
  assert.equal(includeTargetSlug(":scp-wiki:theme:basalt", "scp-wiki"), "theme:basalt");
  assert.equal(includeTargetSlug(":SCP-WIKI:Component:Preview", "scp-wiki"), "component:preview");
  assert.equal(includeTargetSlug("component:license-box", "scp-wiki"), "component:license-box");
  assert.equal(includeTargetSlug(":scp-jp:theme:x", "scp-wiki"), null);
  assert.equal(includeTargetSlug("{$theme}", "scp-wiki"), null);
  assert.equal(includeTargetSlug("component:license-box@@", "scp-wiki"), null);
  assert.equal(includeTargetSlug("@@[[span", "scp-wiki"), null);
});

test("walks the transitive include closure once per page, roots first", () => {
  const sources = {
    "scp-1": "[[include :scp-wiki:theme:basalt]]\n[[include component:a |x=1]]",
    "theme:basalt": "[[include component:interwiki-style]]\n@@[[include component:a@@]]",
    "component:a": "[[include :scp-wiki:component:b]]\n[[include theme:basalt]]",
    "component:b": "plain",
    "component:interwiki-style": "[[include :other-site:component:c]]",
  };
  const closure = includeClosure(
    ["scp-1"],
    (slug) => (slug in sources ? Buffer.from(sources[slug]) : null),
    "scp-wiki",
  );
  assert.deepEqual(closure.map((page) => page.slug), [
    "scp-1",
    "theme:basalt",
    "component:a",
    "component:interwiki-style",
    "component:b",
  ]);
});

test("reports current, drifted, and missing pages against the corpus", () => {
  const closure = [
    {slug: "scp-1", source: Buffer.from("current")},
    {slug: "theme:basalt", source: Buffer.from("july")},
    {slug: "component:gone", source: Buffer.from("x")},
    {slug: "component:unmirrored", source: null},
  ];
  const output = [
    `scp-1|${sha("current")}|3|2026-05-22T07:46:05Z`,
    `theme:basalt|${sha("may")}|0|2026-05-12T22:18:39Z`,
    "component:gone|||",
    "component:unmirrored|||",
  ].join("\n");
  const report = compareDriftRows(closure, output);

  assert.equal(report.status, "drift");
  assert.deepEqual(report.drifted, ["theme:basalt", "component:gone", "component:unmirrored"]);
  assert.deepEqual(
    report.pages.map((page) => [page.slug, page.status]),
    [
      ["scp-1", "current"],
      ["theme:basalt", "drift"],
      ["component:gone", "local_missing"],
      ["component:unmirrored", "corpus_missing"],
    ],
  );
  assert.equal(report.pages[1].local_revision_number, 0);
  assert.equal(report.pages[1].local_sha256, sha("may"));
  assert.equal(compareDriftRows(closure.slice(0, 1), output.split("\n")[0]).status, "current");
  assert.throws(() => compareDriftRows(closure, "scp-1|x"), /malformed row/u);
});

test("queries only the selected site and quotes slugs", () => {
  const query = buildDriftQuery({siteId: 6000005}, ["theme:basalt", "o'brien"]);
  assert.match(query, /page\.site_id = 6000005/u);
  assert.match(query, /\('theme:basalt'\)/u);
  assert.match(query, /\('o''brien'\)/u);
});

test("requires a corpus root and at least one root page", () => {
  assert.throws(() => parseArgs(["--root", "scp-1"]), /--corpus-root/u);
  assert.throws(() => parseArgs(["--corpus-root", "/c"]), /--root/u);
  assert.deepEqual(
    parseArgs(["--corpus-root", "/c", "--root", "scp-1", "--root", "scp-2", "--site-id", "7"]).roots,
    ["scp-1", "scp-2"],
  );
});
