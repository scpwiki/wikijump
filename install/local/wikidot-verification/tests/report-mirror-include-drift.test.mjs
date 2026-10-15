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

test("source and transitive include mutations change the drift proof, and reverting restores it", () => {
  // Keep the originally imported database state while the corpus/source
  // changes. This tests the *drift detector's* invalidation inputs, not the
  // Deepwell renderer or propagation of compiled dependent pages.
  const source = new Map([
    ["scp-test", "[[include :scp-wiki:theme:basalt]]"],
    ["theme:basalt", "[[include :scp-wiki:component:preview]]"],
    ["component:preview", "initial preview"],
    ["component:acs-animation", "initial animation"],
  ]);
  const imported = new Map(source);
  const read = (slug) => source.has(slug) ? Buffer.from(source.get(slug)) : null;
  const observe = () => {
    const closure = includeClosure(["scp-test"], read, "scp-wiki");
    const db = closure.map(({slug}) => (
      `${slug}|${imported.has(slug) ? sha(imported.get(slug)) : ""}|1|2026-10-15T00:00:00Z`
    )).join("\n");
    return {closure: closure.map(({slug}) => slug), report: compareDriftRows(closure, db)};
  };

  const before = observe();
  assert.deepEqual(before.closure, ["scp-test", "theme:basalt", "component:preview"]);
  assert.equal(before.report.status, "current");

  // Mutating a nested component invalidates the referenced source hash.
  source.set("component:preview", "updated preview");
  assert.deepEqual(observe().report.drifted, ["component:preview"]);
  source.set("component:preview", imported.get("component:preview"));
  assert.equal(observe().report.status, "current");

  // Replacing the parent's nested include moves the closure to a different
  // component, even when the root page's source was not edited.
  source.set("theme:basalt", "[[include :scp-wiki:component:acs-animation]]");
  assert.deepEqual(observe().closure, ["scp-test", "theme:basalt", "component:acs-animation"]);
  assert.deepEqual(observe().report.drifted, ["theme:basalt"]);

  // A root-page edit must discover an unrelated target in the new source.
  source.set("scp-test", "[[include :scp-wiki:component:preview]]");
  assert.deepEqual(observe().closure, ["scp-test", "component:preview"]);
  assert.deepEqual(observe().report.drifted, ["scp-test"]);

  source.set("scp-test", imported.get("scp-test"));
  source.set("theme:basalt", imported.get("theme:basalt"));
  assert.deepEqual(observe(), before);
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

 test("ignores complete escaped include examples and comments", () => {
  const sources = {
    root: "@@[[include component:escaped]]@@\n[!-- [[include component:commented]] --]\n[[include component:real]]\n[[code]]\n[[include component:in-code]]\n[[/code]]",
    "component:real": "plain",
    "component:in-code": "plain",
  };
  const closure = includeClosure(["root"], (slug) => sources[slug] == null ? null : Buffer.from(sources[slug]), "scp-wiki");
  assert.deepEqual(closure.map(({slug}) => slug), ["root", "component:real", "component:in-code"]);
});
