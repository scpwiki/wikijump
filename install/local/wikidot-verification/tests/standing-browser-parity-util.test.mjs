import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  createPrivateEmptyDirectory,
  sealJsonNoReplace,
} from "../src/standing-browser-parity-util.mjs";

test("a sealed JSON retry refuses a substituted symlink", async (context) => {
  const parent = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-browser-parity-util-"),
  );
  context.after(() => fs.rm(parent, { recursive: true, force: true }));
  const output = path.join(parent, "output");
  await createPrivateEmptyDirectory(output);
  const target = path.join(parent, "target.json");
  const destination = path.join(output, "sealed.json");
  await fs.writeFile(target, '{"value":1}\n');
  await fs.symlink(target, destination);
  await assert.rejects(
    sealJsonNoReplace(destination, { value: 1 }),
    /not a private regular file/u,
  );
});
