import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {openCorpusOutputDirectory, writeCorpusOutputFile} from "../src/corpus-output-writer.mjs";

test("corpus output writer creates an external directory and atomically replaces a file", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "corpus-output-writer-"));
  t.after(() => fs.rm(root, {recursive: true, force: true}));
  const corpus = path.join(root, "corpus");
  const output = path.join(root, "output", "nested");
  await fs.mkdir(corpus);
  const directory = await openCorpusOutputDirectory(corpus, output);
  t.after(() => directory.close());

  await writeCorpusOutputFile(directory, "report.json", "first");
  await writeCorpusOutputFile(directory, "report.json", "second");
  assert.equal(await fs.readFile(path.join(output, "report.json"), "utf8"), "second");
  assert.deepEqual((await fs.readdir(output)).sort(), ["report.json"]);
});

test("corpus output directory rejects destinations inside the corpus", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "corpus-output-inside-"));
  t.after(() => fs.rm(root, {recursive: true, force: true}));
  const output = path.join(root, "generated");
  await assert.rejects(() => openCorpusOutputDirectory(root, output), /outside --corpus/u);
});
