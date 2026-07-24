import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";

import {
  assertFixtureResourceManifestEntry,
  assertResolvedPathWithin,
  buildFixtureResourceTargetPath,
  isWikidotResourceHost,
  resolveFixtureResourcePath,
} from "../src/resource-manifest.mjs";

test("resource manifest derives a deterministic query-sensitive target and validates its binding", () => {
  const localTargetPath = buildFixtureResourceTargetPath({
    fixtureSlug: "EN:alpha",
    site: "scp-wiki.wdfiles.com",
    wikidotPath: "/local--files/alpha/image.png",
    urlSearch: "?rev=2",
  });
  const entry = {
    filename: "image.png",
    fixture_slug: "EN:alpha",
    kind_guess: "image",
    local_target_path: localTargetPath,
    original_url: "https://scp-wiki.wdfiles.com/local--files/alpha/image.png?rev=2",
    sha256: "a".repeat(64),
    site: "scp-wiki.wdfiles.com",
    source_path: "source.wikidot.txt",
    wikidot_path: "/local--files/alpha/image.png",
  };
  assert.equal(isWikidotResourceHost(entry.site), true);
  assert.equal(assertFixtureResourceManifestEntry(entry, {requireSha256: true}), entry);
  assert.equal(resolveFixtureResourcePath("/tmp/root", localTargetPath), path.join("/tmp/root", localTargetPath));
  assert.doesNotThrow(() => assertResolvedPathWithin("/tmp/root", "/tmp/root/resources/file"));
  assert.throws(() => assertResolvedPathWithin("/tmp/root", "/tmp/elsewhere"), /escapes/u);
});
