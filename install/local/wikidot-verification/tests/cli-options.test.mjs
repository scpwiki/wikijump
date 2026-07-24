import assert from "node:assert/strict";
import test from "node:test";

import {
  parsePositiveIntegerOption,
  readRequiredOptionValue,
  UsageError,
} from "../src/cli-options.mjs";

test("shared CLI option helpers preserve usage-error contracts", () => {
  assert.equal(readRequiredOptionValue(["--ttl", "30"], 0, "--ttl"), "30");
  assert.equal(parsePositiveIntegerOption("30", "--ttl"), 30);
  assert.throws(
    () => readRequiredOptionValue(["--ttl"], 0, "--ttl"),
    (error) => error instanceof UsageError && error.message === "--ttl needs a value",
  );
  assert.throws(
    () => parsePositiveIntegerOption("0", "--ttl"),
    (error) => error instanceof UsageError && error.message === "--ttl must be a positive integer",
  );
});
