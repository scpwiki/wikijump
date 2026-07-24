import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import {constants as fsConstants} from "node:fs";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import {
  createCorpusFileReader,
  openCorpusFileNoSymlinks,
} from "../src/corpus-file-reader.mjs";

const execFileAsync = promisify(execFile);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const scriptPath = path.resolve(__dirname, "../scripts/corpus-discover.mjs");

async function writeJson(filePath, value) {
  await fs.writeFile(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

async function runDiscover(args, options = {}) {
  return execFileAsync(process.execPath, [scriptPath, ...args], options);
}

async function assertDiscoverFails(args, messagePattern) {
  await assert.rejects(runDiscover(args), (error) => {
    assert.match(error.stderr, messagePattern);
    return true;
  });
}

test("corpus-discover inventories files and writes deterministic canaries", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-discover-"));
  const corpus = path.join(root, "corpus");
  const outputDir = path.join(root, "out");
  const pageDir = path.join(corpus, "pages", "scp-001");
  const fragmentDir = path.join(corpus, "pages", "fragment:card");
  const assetDir = path.join(corpus, "assets");

  await fs.mkdir(pageDir, { recursive: true });
  await fs.mkdir(fragmentDir, { recursive: true });
  await fs.mkdir(assetDir, { recursive: true });

  await fs.writeFile(
    path.join(pageDir, "source.wikidot.txt"),
    [
      "[[include fragment:card]]",
      "[[module ListPages category=\"scp\"]]",
      "[[image assets/example.svg]]",
      "http://example.test/link"
    ].join("\n")
  );
  await writeJson(path.join(pageDir, "meta.json"), {
    title: "SCP-001 Fixture",
    tags: ["scp", "featured"],
    parent_fullname: "scp-series"
  });
  await fs.writeFile(path.join(pageDir, "entity_id.txt"), "1001\n");

  await fs.writeFile(path.join(fragmentDir, "source.wikidot.txt"), "|| cell ||\n");
  await writeJson(path.join(fragmentDir, "meta.json"), {
    title_shown: "Fragment Card",
    tags: ["fragment"]
  });
  await fs.writeFile(path.join(assetDir, "example.svg"), "<svg />\n");

  const { stdout } = await runDiscover([
    "--corpus",
    corpus,
    "--output-dir",
    outputDir,
    "--canary-count",
    "2"
  ]);
  const summary = JSON.parse(stdout);

  assert.equal(summary.filesInventoried, 6);
  assert.equal(summary.pageSourceCandidates, 2);
  assert.equal(summary.canaryRows, 2);
  assert.equal(summary.candidateTypeCounts["page-source"], 2);
  assert.equal(summary.candidateTypeCounts.image, 1);
  assert.equal(summary.constructCounts.include, 1);
  assert.equal(summary.constructCounts["module-listpages"], 1);

  const manifest = await fs.readFile(path.join(outputDir, "corpus-manifest.tsv"), "utf8");
  assert.match(manifest, /scp-001\tSCP-001 Fixture/);
  assert.match(manifest, /include:fragment:card/);
  assert.match(manifest, /module:ListPages/);
  assert.match(manifest, /metadata\/tags/);

  const canaries = await fs.readFile(path.join(outputDir, "canary-pages.tsv"), "utf8");
  assert.match(canaries, /scp-001/);

  const markdown = await fs.readFile(path.join(outputDir, "corpus-discovery-summary.md"), "utf8");
  assert.match(markdown, /files inventoried: 6/);
});

test("corpus-discover rejects missing option values and invalid canary counts", async () => {
  await assertDiscoverFails(["--corpus"], /Missing value for --corpus/);
  await assertDiscoverFails(["--output-dir", "--canary-count", "1"], /Missing value for --output-dir/);
  await assertDiscoverFails(["--canary-count", "0"], /--canary-count must be a positive integer/);
  await assertDiscoverFails(["--canary-count", "-2"], /--canary-count must be a positive integer/);
  await assertDiscoverFails(["--canary-count", "1.5"], /--canary-count must be a positive integer/);
  await assertDiscoverFails(["--canary-count", "2abc"], /--canary-count must be a positive integer/);
});

test("corpus-discover rejects output directories inside the corpus", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-discover-self-inventory-"));
  const corpus = path.join(root, "corpus");
  const outputDir = path.join(corpus, "out");

  await fs.mkdir(corpus, { recursive: true });

  await assertDiscoverFails([
    "--corpus",
    corpus,
    "--output-dir",
    outputDir
  ], /--output-dir must be outside --corpus/);
});

test("corpus-discover rejects dot-prefixed output directories inside the corpus on repeated runs", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-discover-dot-output-"));
  const corpus = path.join(root, "corpus");
  const outputDir = path.join(corpus, "..output");

  await fs.mkdir(corpus, { recursive: true });

  for (let attempt = 0; attempt < 2; attempt += 1) {
    await assertDiscoverFails([
      "--corpus",
      corpus,
      "--output-dir",
      outputDir,
      "--canary-count",
      "1"
    ], /--output-dir must be outside --corpus/);
  }

  await assert.rejects(fs.stat(outputDir), { code: "ENOENT" });
});

test("corpus-discover rejects an outside output symlink into the corpus", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-output-link-"));
  const corpus = path.join(root, "corpus");
  const insideOutput = path.join(corpus, ".output");
  const outsideLink = path.join(root, "outside-link");
  await fs.mkdir(insideOutput, { recursive: true });
  await fs.symlink(insideOutput, outsideLink, "dir");

  await assertDiscoverFails(
    ["--corpus", corpus, "--output-dir", outsideLink, "--canary-count", "1"],
    /must not contain symbolic links/,
  );
  assert.deepEqual(await fs.readdir(insideOutput), []);
});

test("corpus-discover rejects a symlinked parent in the output path", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-output-parent-"));
  const corpus = path.join(root, "corpus");
  const linkedParent = path.join(root, "linked-parent");
  await fs.mkdir(corpus, { recursive: true });
  await fs.symlink(corpus, linkedParent, "dir");

  await assertDiscoverFails(
    ["--corpus", corpus, "--output-dir", path.join(linkedParent, "nested"), "--canary-count", "1"],
    /must not contain symbolic links/,
  );
  await assert.rejects(fs.stat(path.join(corpus, "nested")), { code: "ENOENT" });
});

test("corpus-discover replaces an existing hard-linked output without truncating its target", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-output-hardlink-"));
  const corpus = path.join(root, "corpus");
  const outputDir = path.join(root, "out");
  const protectedPath = path.join(root, "protected.txt");
  const inventoryPath = path.join(outputDir, "corpus-file-inventory.tsv");
  const protectedContents = "PROTECTED_OUTPUT_TARGET\n";

  await fs.mkdir(path.join(corpus, "assets"), { recursive: true });
  await fs.mkdir(outputDir);
  await fs.writeFile(path.join(corpus, "assets", "example.svg"), "<svg />\n");
  await fs.writeFile(protectedPath, protectedContents);
  await fs.link(protectedPath, inventoryPath);

  await runDiscover([
    "--corpus",
    corpus,
    "--output-dir",
    outputDir,
    "--canary-count",
    "1",
  ]);

  assert.equal(await fs.readFile(protectedPath, "utf8"), protectedContents);
  assert.match(await fs.readFile(inventoryPath, "utf8"), /^path\tsize_bytes\t/);
  assert.notEqual((await fs.stat(protectedPath)).ino, (await fs.stat(inventoryPath)).ino);
});

test("corpus-discover rejects symlinked page metadata files", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-discover-symlink-"));
  const corpus = path.join(root, "corpus");
  const outputDir = path.join(root, "out");
  const pageDir = path.join(corpus, "pages", "evil");
  const secretPath = path.join(root, "secret.txt");

  await fs.mkdir(pageDir, { recursive: true });
  await fs.writeFile(path.join(pageDir, "source.wikidot.txt"), "[[module ListPages]]\n");
  await writeJson(path.join(pageDir, "meta.json"), {
    title: "Evil Fixture"
  });
  await fs.writeFile(secretPath, "LEAKED_SECRET_TOKEN=autovalidator-12345\n");
  await fs.symlink(secretPath, path.join(pageDir, "entity_id.txt"));

  await assertDiscoverFails([
    "--corpus",
    corpus,
    "--output-dir",
    outputDir,
    "--canary-count",
    "1"
  ], /Corpus path must be a regular file/);

  await assert.rejects(fs.readFile(path.join(outputDir, "corpus-manifest.tsv"), "utf8"), {
    code: "ENOENT"
  });
});

test("corpus-discover rejects intermediate directory symlinks", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-discover-dirlink-"));
  const corpus = path.join(root, "corpus");
  const outsidePages = path.join(root, "outside-pages");
  const pageDir = path.join(outsidePages, "linked");
  const outputDir = path.join(root, "out");

  await fs.mkdir(corpus, { recursive: true });
  await fs.mkdir(pageDir, { recursive: true });
  await fs.writeFile(path.join(pageDir, "source.wikidot.txt"), "outside source\n");
  await writeJson(path.join(pageDir, "meta.json"), { title: "Outside Fixture" });
  await fs.writeFile(path.join(pageDir, "entity_id.txt"), "1001\n");
  await fs.symlink(outsidePages, path.join(corpus, "pages"), "dir");

  await assertDiscoverFails(
    ["--corpus", corpus, "--output-dir", outputDir, "--canary-count", "1"],
    /Corpus path must not contain symbolic links/,
  );

  await assert.rejects(fs.readFile(path.join(outputDir, "corpus-manifest.tsv"), "utf8"), {
    code: "ENOENT",
  });
});

test("corpus-discover rejects oversized metadata before reading it", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-discover-oversized-"));
  const corpus = path.join(root, "corpus");
  const outputDir = path.join(root, "out");
  const pageDir = path.join(corpus, "pages", "oversized");

  await fs.mkdir(pageDir, { recursive: true });
  await fs.writeFile(path.join(pageDir, "source.wikidot.txt"), "ordinary source\n");
  await writeJson(path.join(pageDir, "meta.json"), { title: "Oversized Fixture" });
  await fs.writeFile(path.join(pageDir, "entity_id.txt"), "x".repeat(257));

  await assertDiscoverFails(
    ["--corpus", corpus, "--output-dir", outputDir, "--canary-count", "1"],
    /Corpus file exceeds 256 byte limit/,
  );

  await assert.rejects(fs.readFile(path.join(outputDir, "corpus-manifest.tsv"), "utf8"), {
    code: "ENOENT",
  });
});

test("corpus-discover promptly rejects FIFO metadata", { timeout: 5000 }, async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-discover-fifo-"));
  const corpus = path.join(root, "corpus");
  const outputDir = path.join(root, "out");
  const pageDir = path.join(corpus, "pages", "fifo");

  await fs.mkdir(pageDir, { recursive: true });
  await fs.writeFile(path.join(pageDir, "source.wikidot.txt"), "ordinary source\n");
  await writeJson(path.join(pageDir, "meta.json"), { title: "FIFO Fixture" });
  await execFileAsync("mkfifo", [path.join(pageDir, "entity_id.txt")]);

  await assert.rejects(
    runDiscover(
      ["--corpus", corpus, "--output-dir", outputDir, "--canary-count", "1"],
      { timeout: 2000 },
    ),
    (error) => {
      assert.equal(error.killed, false);
      assert.match(error.stderr, /Corpus path must be a regular file/);
      return true;
    },
  );

  await assert.rejects(fs.readFile(path.join(outputDir, "corpus-manifest.tsv"), "utf8"), {
    code: "ENOENT",
  });
});

test("corpus-discover rejects malformed metadata JSON without echoing its contents", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-discover-json-"));
  const corpus = path.join(root, "corpus");
  const outputDir = path.join(root, "out");
  const pageDir = path.join(corpus, "pages", "malformed");
  const contentMarker = "MALFORMED_JSON_CONTENT_MARKER";

  await fs.mkdir(pageDir, { recursive: true });
  await fs.writeFile(path.join(pageDir, "source.wikidot.txt"), "ordinary source\n");
  await fs.writeFile(path.join(pageDir, "meta.json"), `{"title":"${contentMarker}"`);
  await fs.writeFile(path.join(pageDir, "entity_id.txt"), "1001\n");

  await assert.rejects(
    runDiscover(["--corpus", corpus, "--output-dir", outputDir, "--canary-count", "1"]),
    (error) => {
      assert.match(error.stderr, /Invalid JSON in corpus file/);
      assert.doesNotMatch(error.stderr, new RegExp(contentMarker));
      return true;
    },
  );

  await assert.rejects(fs.readFile(path.join(outputDir, "corpus-manifest.tsv"), "utf8"), {
    code: "ENOENT",
  });
});

test("descriptor reader rejects a deterministic pathname swap after open", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-reader-swap-"));
  const corpus = path.join(root, "corpus");
  const targetPath = path.join(corpus, "entity_id.txt");
  const openedPath = path.join(corpus, "entity_id.opened.txt");
  const replacementPath = path.join(corpus, "replacement.txt");
  const replacementMarker = "PATH_SWAP_CONTENT_MARKER";

  await fs.mkdir(corpus, { recursive: true });
  await fs.writeFile(targetPath, "original-safe-value\n");
  await fs.writeFile(replacementPath, `${replacementMarker}\n`);

  const readWithSwap = createCorpusFileReader({
    async openFile(corpusRoot, filePath, flags) {
      const fileHandle = await openCorpusFileNoSymlinks(corpusRoot, filePath, flags);
      await fs.rename(filePath, openedPath);
      await fs.rename(replacementPath, filePath);
      return fileHandle;
    },
  });

  await assert.rejects(readWithSwap(corpus, targetPath, { maxBytes: 256 }), (error) => {
    assert.match(error.message, /Corpus path changed while being read/);
    assert.doesNotMatch(error.message, new RegExp(replacementMarker));
    return true;
  });

  assert.equal(await fs.readFile(openedPath, "utf8"), "original-safe-value\n");
  assert.equal(await fs.readFile(targetPath, "utf8"), `${replacementMarker}\n`);
});

test("descriptor opener preserves both open and cleanup failures", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-reader-cleanup-"));
  const corpus = path.join(root, "corpus");
  const missingPath = path.join(corpus, "missing.txt");
  const cleanupError = new Error("simulated descriptor cleanup failure");
  await fs.mkdir(corpus);

  await assert.rejects(
    openCorpusFileNoSymlinks(corpus, missingPath, fsConstants.O_RDONLY, {
      async closeHandles(handles) {
        for (const handle of handles) await handle.close();
        throw cleanupError;
      },
    }),
    (error) => {
      assert(error instanceof AggregateError);
      assert.equal(error.errors.length, 2);
      assert.equal(error.errors[0].code, "ENOENT");
      assert.equal(error.errors[1], cleanupError);
      return true;
    },
  );
});

test("descriptor reader rejects a pathname swapped to a symlink of the opened file", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-reader-link-swap-"));
  const corpus = path.join(root, "corpus");
  const targetPath = path.join(corpus, "entity_id.txt");
  const openedPath = path.join(corpus, "entity_id.opened.txt");

  await fs.mkdir(corpus, { recursive: true });
  await fs.writeFile(targetPath, "original-safe-value\n");

  const readWithSymlinkSwap = createCorpusFileReader({
    async openFile(corpusRoot, filePath, flags) {
      const fileHandle = await openCorpusFileNoSymlinks(corpusRoot, filePath, flags);
      await fs.rename(filePath, openedPath);
      await fs.symlink(path.basename(openedPath), filePath);
      return fileHandle;
    },
  });

  await assert.rejects(
    readWithSymlinkSwap(corpus, targetPath, { maxBytes: 256 }),
    /Corpus path changed while being read/,
  );

  assert.equal((await fs.lstat(targetPath)).isSymbolicLink(), true);
  assert.equal(await fs.readFile(openedPath, "utf8"), "original-safe-value\n");
});

test("corpus-discover handles a corpus without pages directory", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-corpus-discover-no-pages-"));
  const corpus = path.join(root, "corpus");
  const outputDir = path.join(root, "out");

  await fs.mkdir(path.join(corpus, "assets"), { recursive: true });
  await fs.writeFile(path.join(corpus, "assets", "example.svg"), "<svg />\n");

  const { stdout } = await runDiscover([
    "--corpus",
    corpus,
    "--output-dir",
    outputDir,
    "--canary-count",
    "1"
  ]);
  const summary = JSON.parse(stdout);

  assert.equal(summary.filesInventoried, 1);
  assert.equal(summary.pageSourceCandidates, 0);
  assert.equal(summary.canaryRows, 0);
  assert.equal(summary.candidateTypeCounts.image, 1);

  const manifest = await fs.readFile(path.join(outputDir, "corpus-manifest.tsv"), "utf8");
  assert.equal(manifest, "page_id\tslug\ttitle\tsource_path\tmetadata_path\ttags\tasset_paths\tdependency_hints\tconstruct_hints\tbytes\tline_count\tstatus\tnotes\n");
});
