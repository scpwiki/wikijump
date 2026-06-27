import test from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

const packageRoot = path.resolve(import.meta.dirname, "..");
const script = path.join(packageRoot, "scripts/extract-rokurokubi-reservations.mjs");

const SAMPLE_CSV = `タイムスタンプ,翻訳者名,記事のURL,記事のタイトル,翻訳完了期限,支部コード,備考
2026/04/30 18:49:37,rokurokubi,https://scp-wiki.wikidot.com/scp-3922,SCP-3922,2026/06/30,EN,
`;

test("extract CLI rejects value-taking flags without values", () => {
  const result = spawnSync(process.execPath, [script, "--source", "--output", "out.csv", "--manifest", "manifest.json"], {
    cwd: packageRoot,
    encoding: "utf8"
  });

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /--source requires a value/u);
});

test("extract CLI writes output CSV and manifest metadata", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "rokurokubi-cli-"));
  try {
    const source = path.join(tempDir, "source.csv");
    const output = path.join(tempDir, "out", "reservations.csv");
    const manifest = path.join(tempDir, "out", "manifest.json");
    writeFileSync(source, SAMPLE_CSV, "utf8");

    const result = spawnSync(
      process.execPath,
      [
        script,
        "--source",
        source,
        "--output",
        output,
        "--manifest",
        manifest,
        "--source-label",
        "reservations-sheet"
      ],
      { cwd: packageRoot, encoding: "utf8" }
    );

    assert.equal(result.status, 0, result.stderr);
    const summary = JSON.parse(result.stdout);
    assert.equal(summary.output, output);
    assert.equal(summary.manifest, manifest);
    assert.equal(summary.rokurokubi_row_count, 1);
    assert.equal(summary.mapped_scp_wiki_count, 1);
    assert.equal(summary.unmapped_count, 0);
    assert.match(readFileSync(output, "utf8"), /scp-3922/u);
    const manifestJson = JSON.parse(readFileSync(manifest, "utf8"));
    assert.equal(manifestJson.source_sheet, "reservations-sheet");
    assert.equal(manifestJson.output_csv, output);
    assert.equal(manifestJson.rokurokubi_row_count, 1);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
