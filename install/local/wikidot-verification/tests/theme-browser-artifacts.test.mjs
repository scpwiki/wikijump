import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  prepareThemeArtifactDirectory,
  writePrivateThemeFile,
  writePrivateThemeJson,
  writeThemeViewportArtifacts,
} from "../src/theme-browser-artifacts.mjs";

test("theme browser artifacts remain private and no-replace", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "theme-artifacts-"));
  t.after(() => fs.rm(root, {recursive: true, force: true}));
  const directory = await prepareThemeArtifactDirectory(path.join(root, "capture"));
  const file = path.join(directory, "value.json");
  await writePrivateThemeJson(file, {status: "pass"});
  assert.equal((await fs.stat(directory)).mode & 0o777, 0o700);
  assert.equal((await fs.stat(file)).mode & 0o777, 0o600);
  assert.match(await fs.readFile(file, "utf8"), /"status": "pass"/u);
  await assert.rejects(
    () => writePrivateThemeFile(file, "replacement"),
    (error) => error.code === "EEXIST",
  );
});

test("viewport artifact writer requires a private screenshot and emits every receipt", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "theme-viewport-"));
  t.after(() => fs.rm(root, {recursive: true, force: true}));
  await fs.chmod(root, 0o700);
  await writePrivateThemeFile(path.join(root, "screenshot.png"), Buffer.from("png"));
  const artifacts = await writeThemeViewportArtifacts(root, {
    screenshot_status: "captured",
    dom: "<html></html>",
    computed_styles: [],
    navigation_timing: null,
    web_vitals: {},
    performance_attribution: {},
    interactions: [],
    errors: {},
    raw_syntax: [],
    verdict: {status: "pass"},
  });
  assert.deepEqual(
    (await fs.readdir(root)).sort(),
    [
      "computed-styles.json",
      "dom.html",
      "interactions.json",
      "network-errors.json",
      "performance-attribution.json",
      "raw-syntax.json",
      "screenshot.png",
      "verdict.json",
      "web-vitals.json",
    ],
  );
  assert.equal(artifacts.screenshot, path.join(root, "screenshot.png"));
});
