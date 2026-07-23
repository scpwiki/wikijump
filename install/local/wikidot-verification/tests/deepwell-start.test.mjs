import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import path from "node:path";
import {spawnSync} from "node:child_process";
import test from "node:test";
import {fileURLToPath} from "node:url";

const PACKAGE_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const START_SCRIPT = path.resolve(PACKAGE_ROOT, "..", "deepwell", "deepwell-start");

test("deepwell cargo-watch commands preserve the lockfile in both profiles", () => {
  const syntax = spawnSync("/bin/sh", ["-n", START_SCRIPT], {encoding: "utf8"});
  assert.equal(syntax.status, 0, syntax.stderr);

  const source = readFileSync(START_SCRIPT, "utf8");
  assert.match(source, /^RUN_COMMAND="run --locked -- \/etc\/deepwell\.toml"$/mu);
  assert.match(source, /^ {4}RUN_COMMAND="run --locked --release -- \/etc\/deepwell\.toml"$/mu);
  assert.match(source, /^ {8}-x "\$RUN_COMMAND"$/mu);
  assert.doesNotMatch(source, /PROFILE_FLAG/u);
});
