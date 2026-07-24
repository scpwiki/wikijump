import assert from "node:assert/strict";
import test from "node:test";

import {
  prepareReferenceCompletionIndex,
  REFERENCE_COMPLETION_POINTER_MAX_BYTES,
} from "../src/reference-acquisition-completion-index.mjs";

test("completion index verifies store bindings before opening any directory", async () => {
  const calls = [];
  await assert.rejects(
    () => prepareReferenceCompletionIndex({fd: 1}, {
      assertStoreBindings: async () => calls.push("bindings"),
      openDirectoryAt: async () => {
        calls.push("open");
        throw new Error("open denied");
      },
      create: false,
    }),
    /open denied/u,
  );
  assert.deepEqual(calls, ["bindings", "open"]);
  assert.equal(REFERENCE_COMPLETION_POINTER_MAX_BYTES, 1024);
});
