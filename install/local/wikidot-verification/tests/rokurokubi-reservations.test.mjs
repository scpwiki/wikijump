import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  extractRokurokubiReservations,
  extractRokurokubiReservationsFromSheets,
  formatCsv,
  mapWikidotUrl,
  parseCsv,
  sha256Hex
} from "../src/rokurokubi-reservations.mjs";

const TEST_DIR = dirname(fileURLToPath(import.meta.url));
const CLI_PATH = join(TEST_DIR, "../scripts/extract-rokurokubi-reservations.mjs");

const SAMPLE_CSV = `タイムスタンプ,翻訳者名,記事のURL,記事のタイトル,翻訳完了期限,支部コード,備考
2026/05/17 3:09:46,C-Dives,https://scp-wiki.wikidot.com/scp-9712,SCP-9712,2026/07/17,EN,

2026/04/30 18:49:37,rokurokubi ,https://scp-wiki.wikidot.com/scp-3922,SCP-3922,2026/06/30,EN,
2026/04/24 23:25:50,Rokurokubi,https://scp-wiki.wikidot.com/scp-9506,National Fog Safety Initiative,2026/06/24,EN,"needs, review"
2026/05/15 18:53:26,rokurokubi,http://scp-zh-tr.wikidot.com/scp-zh-005,SCP-ZH-005,2026/07/15,ZH(ZHTR),
`;

const EXPIRED_CSV = `row,タイムスタンプ,翻訳者名,記事のURL,記事のタイトル,翻訳完了期限,支部コード,備考
0,2024/05/18 02:01:24,rokurokubi,https://scp-wiki.wikidot.com/scp-6670,SCP-6670,2024/07/18,EN,
1,2024/05/20 10:43:43,Rokurokubi,https://scp-wiki.wikidot.com/scp-3922,SCP-3922,2024/07/20,EN,duplicate from active sheet
2,2024/08/18 13:28:48,rokurokubi,https://scp-wiki-cn.wikidot.com/scp-cn-3999,SCP-CN-3999,2024/10/18,CN,
`;

test("extracts rokurokubi rows case-insensitively and preserves sheet provenance", () => {
  const { rows, manifest } = extractRokurokubiReservations(SAMPLE_CSV, {
    sourceRole: "active",
    sourceName: "Main",
    sourceGid: "1325361212"
  });

  assert.equal(rows.length, 3);
  assert.deepEqual(
    rows.map((row) => row.source_rows),
    ["4", "5", "6"]
  );
  assert.equal(rows[0].sheet_roles, "active");
  assert.equal(rows[0].translators, "rokurokubi");
  assert.equal(rows[0].sheet_names, "Main");
  assert.equal(rows[0].sheet_gids, "1325361212");
  assert.equal(rows[0].normalized_path, "scp-3922");
  assert.equal(rows[0].wikijump_mirror_url, "https://scp-wiki.wikijump.localhost/scp-3922");
  assert.equal(rows[1].wikijump_mirror_url, "https://scp-wiki.wikijump.localhost/scp-9506");
  assert.equal(rows[1].notes, "needs, review");
  assert.equal(rows[2].mirror_url_status, "unmapped_scp-zh-tr.wikidot.com");
  assert.equal(rows[2].wikijump_mirror_url, "");
  assert.equal(manifest.source_row_count_excluding_header, 5);
  assert.equal(manifest.rokurokubi_row_count, 3);
  assert.equal(manifest.target_en_row_count, 2);
  assert.equal(manifest.mapped_scp_wiki_count, 2);
  assert.equal(manifest.unmapped_count, 1);
});

test("combines multiple source sheets and deduplicates by normalized source path", () => {
  const { rows, manifest } = extractRokurokubiReservationsFromSheets(
    [
      {
        csvText: SAMPLE_CSV,
        role: "active",
        name: "Main",
        gid: "1325361212",
        label: "active gid"
      },
      {
        csvText: EXPIRED_CSV,
        role: "expired",
        name: "期限切れ削除済",
        gid: "unknown",
        label: "expired tab export"
      }
    ],
    {
      mirrorOrigin: "https://scp-wiki.wikijump.localhost"
    }
  );

  assert.equal(rows.length, 5);
  const duplicate = rows.find((row) => row.normalized_path === "scp-3922");
  assert.equal(duplicate.provenance_count, "2");
  assert.equal(duplicate.sheet_roles, "active | expired");
  assert.equal(duplicate.sheet_names, "Main | 期限切れ削除済");
  assert.equal(duplicate.sheet_gids, "1325361212 | unknown");
  assert.equal(duplicate.source_rows, "4 | 3");
  assert.equal(duplicate.notes, "duplicate from active sheet");
  assert.equal(manifest.source_sheets.length, 2);
  assert.equal(manifest.rokurokubi_row_count_before_deduplication, 6);
  assert.equal(manifest.rokurokubi_row_count, 5);
  assert.equal(manifest.target_en_row_count, 3);
});

test("does not deduplicate unrelated blank or malformed source URLs", () => {
  const csv = `タイムスタンプ,翻訳者名,記事のURL,記事のタイトル,翻訳完了期限,支部コード,備考
2026/01/01,rokurokubi,,Blank A,2026/02/01,EN,
2026/01/02,rokurokubi,,Blank B,2026/02/02,EN,
2026/01/03,rokurokubi,https://scp-wiki.wikidot.com/bad%zzslug,Bad Encoding,2026/02/03,EN,
`;
  const { rows } = extractRokurokubiReservations(csv, {
    sourceRole: "active",
    sourceName: "Main",
    sourceGid: "gid"
  });

  assert.equal(rows.length, 3);
  assert.deepEqual(rows.map((row) => row.title), ["Blank A", "Blank B", "Bad Encoding"]);
  assert.equal(rows[2].mirror_url_status, "unmapped_invalid_slug_encoding");
});

test("uses the first non-empty CSV row as the header", () => {
  const { rows, manifest } = extractRokurokubiReservations(`\n   \n${SAMPLE_CSV}`, {
    sourceRole: "active",
    sourceName: "Main",
    sourceGid: "1325361212"
  });

  assert.equal(rows.length, 3);
  assert.deepEqual(
    rows.map((row) => row.source_rows),
    ["6", "7", "8"]
  );
  assert.equal(manifest.source_row_count_excluding_header, 5);
});

test("formats extracted rows as round-trippable CSV", () => {
  const { rows } = extractRokurokubiReservations(SAMPLE_CSV);
  const output = formatCsv(rows);
  const parsed = parseCsv(output);

  assert.equal(parsed.length, 4);
  assert.deepEqual(parsed[0].slice(0, 4), ["sheet_roles", "sheet_names", "sheet_gids", "source_rows"]);
  assert.equal(parsed[2][10], "needs, review");
});

test("CLI records the checksum of the generated ledger CSV", () => {
  const tempDir = mkdtempSync(join(tmpdir(), "rokurokubi-reservations-"));
  try {
    const sourcePath = join(tempDir, "source.csv");
    const outputPath = join(tempDir, "output.csv");
    const manifestPath = join(tempDir, "manifest.json");
    writeFileSync(sourcePath, SAMPLE_CSV, "utf8");

    const result = spawnSync(process.execPath, [
      CLI_PATH,
      "--source",
      sourcePath,
      "--output",
      outputPath,
      "--manifest",
      manifestPath,
      "--source-role",
      "active",
      "--source-name",
      "Main",
      "--source-gid",
      "1325361212"
    ], { encoding: "utf8" });

    assert.equal(result.status, 0, result.stderr);
    const outputCsv = readFileSync(outputPath, "utf8");
    const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
    assert.equal(manifest.ledger_csv_sha256, sha256Hex(outputCsv));
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("CLI preserves sheet manifest metadata", () => {
  const tempDir = mkdtempSync(join(tmpdir(), "rokurokubi-sheet-manifest-"));
  try {
    const activePath = join(tempDir, "active.csv");
    const expiredPath = join(tempDir, "expired.csv");
    const sheetManifestPath = join(tempDir, "sheet-manifest.json");
    const outputPath = join(tempDir, "output.csv");
    const manifestPath = join(tempDir, "manifest.json");
    writeFileSync(activePath, SAMPLE_CSV, "utf8");
    writeFileSync(expiredPath, EXPIRED_CSV, "utf8");
    writeFileSync(
      sheetManifestPath,
      JSON.stringify({
        workbook: "reservations.xlsx",
        notes: "manual export",
        sheets: [
          { csv: activePath, role: "active", name: "Main", gid: "1325361212", label: "active gid", xlsx_sheet_id: 1, range: "A:G", gid_discovery_status: "matched" },
          { csv: expiredPath, role: "expired", name: "期限切れ削除済", gid: "unknown", label: "expired tab export", xlsx_sheet_id: 2, range: "A:H", gid_discovery_status: "missing" }
        ]
      }),
      "utf8"
    );

    const result = spawnSync(process.execPath, [
      CLI_PATH,
      "--sheet-manifest",
      sheetManifestPath,
      "--output",
      outputPath,
      "--manifest",
      manifestPath
    ], { encoding: "utf8" });

    assert.equal(result.status, 0, result.stderr);
    const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
    assert.equal(manifest.workbook, "reservations.xlsx");
    assert.equal(manifest.notes, "manual export");
    assert.equal(manifest.source_sheets[0].xlsx_sheet_id, 1);
    assert.equal(manifest.source_sheets[0].range, "A:G");
    assert.equal(manifest.source_sheets[0].gid_discovery_status, "matched");
    assert.equal(manifest.source_sheets[1].xlsx_sheet_id, 2);
    assert.equal(manifest.source_sheets[1].range, "A:H");
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("maps only scp-wiki Wikidot URLs to the canonical mirror origin", () => {
  assert.deepEqual(mapWikidotUrl("https://scp-wiki.wikidot.com/scp-9506"), {
    slug: "scp-9506",
    mirrorUrl: "https://scp-wiki.wikijump.localhost/scp-9506",
    status: "mapped_scp-wiki"
  });
  assert.deepEqual(mapWikidotUrl("https://scp-wiki.wikidot.com/scp-9506", "scp-wiki.wikijump.localhost:18443"), {
    slug: "scp-9506",
    mirrorUrl: "https://scp-wiki.wikijump.localhost:18443/scp-9506",
    status: "mapped_scp-wiki"
  });
  assert.deepEqual(mapWikidotUrl("https://scp-wiki.wikidot.com/foo%23bar/baz%3Fqux"), {
    slug: "foo#bar/baz?qux",
    mirrorUrl: "https://scp-wiki.wikijump.localhost/foo%23bar/baz%3Fqux",
    status: "mapped_scp-wiki"
  });
  assert.deepEqual(mapWikidotUrl("https://scp-wiki.wikidot.com/%2E%2E/admin"), {
    slug: "../admin",
    mirrorUrl: "",
    status: "unmapped_invalid_slug_dot_segment"
  });
  assert.deepEqual(mapWikidotUrl("https://scp-wiki.wikidot.com/foo/%2E/bar"), {
    slug: "foo/./bar",
    mirrorUrl: "",
    status: "unmapped_invalid_slug_dot_segment"
  });
  assert.equal(
    mapWikidotUrl("https://scp-jp.wikidot.com/scp-173").status,
    "unmapped_scp-jp.wikidot.com"
  );
  assert.equal(mapWikidotUrl("not a url").status, "unmapped_invalid_url");
  assert.equal(
    mapWikidotUrl("https://scp-wiki.wikidot.com/bad%zzslug").status,
    "unmapped_invalid_slug_encoding"
  );
});
