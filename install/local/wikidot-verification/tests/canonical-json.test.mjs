import assert from "node:assert/strict";
import test from "node:test";

import {codePointCompare, sha256Hex, stableStringify} from "../src/canonical-json.mjs";

test("canonical JSON sorts object keys recursively without reordering arrays", () => {
  assert.equal(
    stableStringify({z: 1, a: {β: 2, A: 1}, list: [{b: 2, a: 1}, null]}),
    '{"a":{"A":1,"β":2},"list":[{"a":1,"b":2},null],"z":1}',
  );
  assert.equal(codePointCompare("A", "β"), -1);
  assert.equal(codePointCompare("same", "same"), 0);
  assert.equal(sha256Hex("alpha"), "8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8");
});
