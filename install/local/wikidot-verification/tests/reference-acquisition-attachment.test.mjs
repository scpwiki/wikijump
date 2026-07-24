import assert from "node:assert/strict";
import test from "node:test";

import {normalizeAcquisitionAttachment} from "../src/reference-acquisition-attachment.mjs";

function attachment(overrides = {}) {
  return {
    filename: "image one.png",
    mime: "image/png",
    original_url: "https://scp-wiki.wdfiles.com/local--files/alpha/image%20one.png",
    sha256: "a".repeat(64),
    size: 123,
    wikidot_path: "/local--files/alpha/image%20one.png",
    ...overrides,
  };
}

test("acquisition attachment normalization binds canonical URL, path, filename, and uniqueness", () => {
  const seen = new Set();
  assert.deepEqual(normalizeAcquisitionAttachment(attachment(), "row 1", seen), attachment());
  assert.throws(() => normalizeAcquisitionAttachment(attachment(), "row 1", seen), /duplicate attachment URL/u);
  assert.throws(
    () => normalizeAcquisitionAttachment(attachment({original_url: "https://example.test/local--files/alpha/image%20one.png"}), "row 2", new Set()),
    /host is out of scope/u,
  );
  assert.throws(
    () => normalizeAcquisitionAttachment(attachment({wikidot_path: "/local--files/alpha/%2e%2e"}), "row 3", new Set()),
    /unsafe segment/u,
  );
});
