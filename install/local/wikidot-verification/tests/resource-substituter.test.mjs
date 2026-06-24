import assert from "node:assert/strict";
import test from "node:test";

import {scanForFixtureLocalResources} from "../src/resource-scanner.mjs";
import {substituteFixtureResourceUrls} from "../src/resource-substituter.mjs";

function materializedManifest(sourceText, fixtureSlug = "substitution-fixture") {
  const {manifest} = scanForFixtureLocalResources({
    sourceText,
    fixtureSlug,
    sourcePath: `fixtures/${fixtureSlug}.wikidot.txt`,
  });
  return manifest.map((entry, index) => ({
    ...entry,
    sha256: String(index + 1).repeat(64),
  }));
}

test("substitutes every materialized manifest URL and leaves other URLs unchanged", () => {
  const cssUrl =
    "https://scp-wiki.wikidot.com/local--files/scp-8980/departuremono.css";
  const imageUrl =
    "https://scp-wiki.wikidot.com/local--files/scp-8980/fractal.webp";
  const untouchedUrl =
    "https://scp-wiki.wikidot.com/local--files/scp-8980/not-in-manifest.png";
  const manifest = materializedManifest(`${cssUrl}\n${imageUrl}`);
  const sourceText = [
    `@import url('${cssUrl}');`,
    `body { background: url("${imageUrl}"); }`,
    `a { mask-image: url('${cssUrl}'); }`,
    `keep ${untouchedUrl}`,
    "keep https://example.com/local--files/scp-8980/external.png",
  ].join("\n");

  const result = substituteFixtureResourceUrls({
    sourceText,
    manifest,
    localUrlPrefix: "/fixture-assets",
  });

  assert.equal(result.substitutions, 3);
  assert.ok(!result.text.includes(cssUrl));
  assert.ok(!result.text.includes(imageUrl));
  assert.ok(
    result.text.includes(
      "/fixture-assets/resources/substitution-fixture/scp-wiki_wikidot_com/local--files/scp-8980/departuremono.css",
    ),
  );
  assert.ok(
    result.text.includes(
      "/fixture-assets/resources/substitution-fixture/scp-wiki_wikidot_com/local--files/scp-8980/fractal.webp",
    ),
  );
  assert.ok(result.text.includes(untouchedUrl));
  assert.ok(
    result.text.includes(
      "https://example.com/local--files/scp-8980/external.png",
    ),
  );
  assert.ok(sourceText.includes(cssUrl));
});

test("refuses to substitute an entry that has not been materialized", () => {
  const url = "https://scp-wiki.wikidot.com/local--files/test/style.css";
  const {manifest} = scanForFixtureLocalResources({
    sourceText: url,
    fixtureSlug: "unmaterialized-fixture",
    sourcePath: "fixtures/unmaterialized.wikidot.txt",
  });

  assert.throws(
    () => substituteFixtureResourceUrls({sourceText: url, manifest}),
    /has not been materialized/,
  );
});

test("rejects a combined manifest that maps one URL to two fixture paths", () => {
  const url = "https://scp-wiki.wikidot.com/local--files/test/style.css";
  const first = materializedManifest(url, "fixture-one")[0];
  const second = materializedManifest(url, "fixture-two")[0];

  assert.throws(
    () =>
      substituteFixtureResourceUrls({
        sourceText: url,
        manifest: [first, second],
      }),
    /multiple local paths/,
  );
});

test("uses a root-relative resources URL by default", () => {
  const url = "https://scp-wiki.wikidot.com/local--files/test/style.css";
  const [entry] = materializedManifest(url, "default-prefix-fixture");

  const result = substituteFixtureResourceUrls({
    sourceText: `@import url('${url}');`,
    manifest: [entry],
  });

  assert.equal(result.substitutions, 1);
  assert.equal(
    result.text,
    "@import url('/resources/default-prefix-fixture/scp-wiki_wikidot_com/local--files/test/style.css');",
  );
});

test("does not replace a non-manifest URL that only has a manifest URL prefix", () => {
  const url = "https://scp-wiki.wikidot.com/local--files/test/asset.png";
  const queryVariant = `${url}?variant=large`;
  const longerFilename = `${url}2`;
  const [entry] = materializedManifest(url, "prefix-boundary-fixture");

  const result = substituteFixtureResourceUrls({
    sourceText: [`exact ${url}.`, `query ${queryVariant}`, `longer ${longerFilename}`].join("\n"),
    manifest: [entry],
  });

  assert.equal(result.substitutions, 1);
  assert.ok(
    result.text.includes(
      "exact /resources/prefix-boundary-fixture/scp-wiki_wikidot_com/local--files/test/asset.png.",
    ),
  );
  assert.ok(result.text.includes(`query ${queryVariant}`));
  assert.ok(result.text.includes(`longer ${longerFilename}`));
});

test("preserves a manifest query and client-side fragment on the local URL", () => {
  const url =
    "https://scp-wiki.wikidot.com/local--files/test/icons.svg?version=2#warning";
  const [entry] = materializedManifest(url, "suffix-fixture");

  const result = substituteFixtureResourceUrls({
    sourceText: `background: url('${url}')`,
    manifest: [entry],
  });

  assert.equal(result.substitutions, 1);
  assert.equal(
    result.text,
    "background: url('/resources/suffix-fixture/scp-wiki_wikidot_com/local--files/test/icons.svg?version=2#warning')",
  );
});
