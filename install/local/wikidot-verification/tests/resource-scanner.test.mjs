import assert from "node:assert/strict";
import test from "node:test";

import { scanForFixtureLocalResources } from "../src/resource-scanner.mjs";

test("detects CSS @import local--files URL", () => {
  const sourceText =
    "@import url('https://scp-wiki.wikidot.com/local--files/scp-8980/departuremono.css')";

  const result = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "fixture-1",
    sourcePath: "samples/import.css",
  });

  assert.equal(result.manifest.length, 1);
  const item = result.manifest[0];
  assert.equal(item.site, "scp-wiki.wikidot.com");
  assert.equal(item.wikidot_path, "/local--files/scp-8980/departuremono.css");
  assert.equal(item.kind_guess, "css");
  assert.equal(item.sha256, null);
  assert.equal(item.source_path, "samples/import.css");
  assert.equal(item.fixture_slug, "fixture-1");
});

test("detects CSS url(...) local--files URL", () => {
  const sourceText = 'body { background: url("https://scp-wiki.wikidot.com/local--files/scp-8980/fractal.webp"); }';

  const result = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "fixture-2",
    sourcePath: "samples/style.css",
  });

  assert.equal(result.manifest.length, 1);
  const item = result.manifest[0];
  assert.equal(item.site, "scp-wiki.wikidot.com");
  assert.equal(item.wikidot_path, "/local--files/scp-8980/fractal.webp");
  assert.equal(item.filename, "fractal.webp");
  assert.equal(item.kind_guess, "image");
});

test("detects bare local--files URL", () => {
  const sourceText =
    "Embed this link: https://scptestwiki.wikidot.com/local--files/sigma:lol/ecw.png";

  const result = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "fixture-3",
    sourcePath: "samples/plain.txt",
  });

  assert.equal(result.manifest.length, 1);
  const item = result.manifest[0];
  assert.equal(item.site, "scptestwiki.wikidot.com");
  assert.equal(item.filename, "ecw.png");
  assert.equal(item.kind_guess, "image");
});

test("uses distinct target paths for query variants", () => {
  const sourceText = [
    "https://scp-wiki.wikidot.com/local--files/test/asset.png?v=1",
    "https://scp-wiki.wikidot.com/local--files/test/asset.png?v=2",
  ].join("\n");

  const result = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "query-fixture",
    sourcePath: "samples/query-variants.txt",
  });

  assert.equal(result.manifest.length, 2);
  assert.notEqual(
    result.manifest[0].local_target_path,
    result.manifest[1].local_target_path,
  );
  assert.ok(
    result.manifest.every((entry) => entry.local_target_path.includes(".__query-")),
  );
});

test("deduplicates duplicate resource references deterministically", () => {
  const sourceText = [
    "https://scp-wiki.wikidot.com/local--files/scp-8980/fractal.webp",
    'url("https://scp-wiki.wikidot.com/local--files/scp-8980/fractal.webp")',
    "@import url('https://scp-wiki.wikidot.com/local--files/scp-8980/fractal.webp')",
  ].join("\n");

  const result = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "fixture-4",
    sourcePath: "samples/dup.txt",
  });

  assert.equal(result.manifest.length, 1);
  assert.equal(result.manifest[0].local_target_path, "resources/fixture-4/scp-wiki_wikidot_com/local--files/scp-8980/fractal.webp");
});

test("non-local external URLs are ignored or out-of-scope", () => {
  const sourceText =
    "https://example.com/local--files/scp-8980/fractal.webp";

  const result = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "fixture-5",
    sourcePath: "samples/oos.txt",
  });

  assert.equal(result.manifest.length, 0);
  assert.equal(result.out_of_scope.length, 1);
  assert.equal(result.out_of_scope[0].site, "example.com");
});

test("trims sentence-ending punctuation from bare local--files URLs", () => {
  const sourceText =
    "See https://scp-wiki.wikidot.com/local--files/scp-8980/fractal.webp.";

  const result = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "fixture-6",
    sourcePath: "samples/punctuation.txt",
  });

  assert.equal(result.manifest.length, 1);
  assert.equal(result.manifest[0].filename, "fractal.webp");
  assert.equal(
    result.manifest[0].original_url,
    "https://scp-wiki.wikidot.com/local--files/scp-8980/fractal.webp",
  );
});

test("canonicalization preserves path double slashes", () => {
  const sourceText =
    "https://scp-wiki.wikidot.com/local--files/scp-8980//fractal.webp\n" +
    "https://scp-wiki.wikidot.com/local--files/scp-8980/fractal.webp";

  const result = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "fixture-7",
    sourcePath: "samples/double-slash.txt",
  });

  assert.equal(result.manifest.length, 2);
  assert.deepEqual(
    result.manifest.map((item) => item.wikidot_path).sort(),
    [
      "/local--files/scp-8980//fractal.webp",
      "/local--files/scp-8980/fractal.webp",
    ],
  );
});

test("deduplicates repeated out-of-scope local--files URLs", () => {
  const sourceText = [
    "https://example.com/local--files/scp-8980/fractal.webp",
    "https://example.com/local--files/scp-8980/fractal.webp",
  ].join("\n");

  const result = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "fixture-8",
    sourcePath: "samples/oos-dup.txt",
  });

  assert.equal(result.manifest.length, 0);
  assert.equal(result.out_of_scope.length, 1);
});

test("trims a Wikidot image-option pipe after a local--files URL", () => {
  const sourceText =
    "[[image https://scp-wiki.wikidot.com/local--files/scp-8980/femalescientist.png|width=300]]";

  const result = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "fixture-9",
    sourcePath: "samples/image-options.txt",
  });

  assert.equal(result.manifest.length, 1);
  assert.equal(result.manifest[0].filename, "femalescientist.png");
  assert.equal(result.manifest[0].kind_guess, "image");
  assert.equal(
    result.manifest[0].original_url,
    "https://scp-wiki.wikidot.com/local--files/scp-8980/femalescientist.png",
  );
});

test("keeps wdfiles.com resources out-of-scope for the anthology follow-up", () => {
  const sourceText =
    "https://scp-sandbox-3.wdfiles.com/local--files/test544/INTRO.mp3";

  const result = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "scp-anthology-2024",
    sourcePath: "corpus/en/pages/scp-anthology-2024/source.wikidot.txt",
  });

  assert.equal(result.manifest.length, 0);
  assert.equal(result.out_of_scope.length, 1);
  assert.equal(result.out_of_scope[0].filename, "INTRO.mp3");
  assert.equal(result.out_of_scope[0].kind_guess, "audio");
});

test("unsafe in-scope resource paths do not abort later scan results", () => {
  const sourceText = [
    "https://scp-wiki.wikidot.com/local--files/scp-8980/bad%00secret.png",
    "https://scp-wiki.wikidot.com/local--files/scp-8980/fractal.webp",
  ].join("\n");

  const result = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug: "fixture-10",
    sourcePath: "samples/unsafe-path.txt",
  });

  assert.equal(result.manifest.length, 1);
  assert.equal(result.manifest[0].filename, "fractal.webp");
  assert.equal(result.out_of_scope.length, 1);
  assert.equal(result.out_of_scope[0].site, "scp-wiki.wikidot.com");
  assert.equal(result.out_of_scope[0].filename, "bad%00secret.png");
  assert.equal(result.out_of_scope[0].wikidot_path, "/local--files/scp-8980/bad%00secret.png");
  assert.equal(result.out_of_scope[0].local_target_path, null);
});
