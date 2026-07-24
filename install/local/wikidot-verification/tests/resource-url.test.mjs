import assert from "node:assert/strict";
import test from "node:test";

import {
  createFixtureLocalResourceUrlRegExp,
  matchFixtureLocalResourceUrls,
  parseFixtureLocalResourceUrlToken,
} from "../src/resource-url.mjs";

test("fixture resource URL parsing preserves surrounding punctuation and query identity", () => {
  const source = 'before "https://scp-wiki.wdfiles.com/local--files/page/image.png?rev=2", after';
  const matches = [...matchFixtureLocalResourceUrls(source)];
  assert.equal(matches.length, 1);
  assert.equal(createFixtureLocalResourceUrlRegExp().global, true);
  assert.deepEqual(
    parseFixtureLocalResourceUrlToken('"https://scp-wiki.wdfiles.com/local--files/page/image.png?rev=2",'),
    {
      canonicalUrl: "https://scp-wiki.wdfiles.com/local--files/page/image.png?rev=2",
      leadingText: '"',
      parsed: new URL("https://scp-wiki.wdfiles.com/local--files/page/image.png?rev=2"),
      resourceUrl: "https://scp-wiki.wdfiles.com/local--files/page/image.png?rev=2",
      trailingText: '",',
    },
  );
});
