import test from "node:test";
import assert from "node:assert/strict";
import {
  extractRokurokubiReservations,
  formatCsv,
  mapWikidotUrl,
  parseCsv
} from "../src/rokurokubi-reservations.mjs";

const SAMPLE_CSV = `タイムスタンプ,翻訳者名,記事のURL,記事のタイトル,翻訳完了期限,支部コード,備考
2026/05/17 3:09:46,C-Dives,https://scp-wiki.wikidot.com/scp-9712,SCP-9712,2026/07/17,EN,
2026/04/30 18:49:37,rokurokubi,https://scp-wiki.wikidot.com/scp-3922,SCP-3922,2026/06/30,EN,
2026/04/24 23:25:50,Rokurokubi,https://scp-wiki.wikidot.com/scp-9506,National Fog Safety Initiative,2026/06/24,EN,"needs, review"
2026/05/15 18:53:26,rokurokubi,http://scp-zh-tr.wikidot.com/scp-zh-005,SCP-ZH-005,2026/07/15,ZH(ZHTR),
`;

test("extracts rokurokubi rows case-insensitively and preserves source row numbers", () => {
  const { rows, manifest } = extractRokurokubiReservations(SAMPLE_CSV);

  assert.equal(rows.length, 3);
  assert.deepEqual(
    rows.map((row) => row.source_row),
    ["3", "4", "5"]
  );
  assert.equal(rows[0].wikijump_mirror_url, "http://scp-wiki.wikijump.localhost:18443/scp-3922");
  assert.equal(rows[1].wikijump_mirror_url, "http://scp-wiki.wikijump.localhost:18443/scp-9506");
  assert.equal(rows[1].notes, "needs, review");
  assert.equal(rows[2].mirror_url_status, "unmapped_scp-zh-tr.wikidot.com");
  assert.equal(rows[2].wikijump_mirror_url, "");
  assert.equal(manifest.source_row_count_excluding_header, 4);
  assert.equal(manifest.rokurokubi_row_count, 3);
  assert.equal(manifest.mapped_scp_wiki_count, 2);
  assert.equal(manifest.unmapped_count, 1);
});

test("formats extracted rows as round-trippable CSV", () => {
  const { rows } = extractRokurokubiReservations(SAMPLE_CSV);
  const output = formatCsv(rows);
  const parsed = parseCsv(output);

  assert.equal(parsed.length, 4);
  assert.deepEqual(parsed[0].slice(0, 4), ["source_row", "timestamp", "translator", "source_url"]);
  assert.equal(parsed[2][7], "needs, review");
});

test("maps only scp-wiki Wikidot URLs to the canonical mirror host", () => {
  assert.deepEqual(mapWikidotUrl("https://scp-wiki.wikidot.com/scp-9506"), {
    slug: "scp-9506",
    mirrorUrl: "http://scp-wiki.wikijump.localhost:18443/scp-9506",
    status: "mapped_scp-wiki"
  });
  assert.equal(
    mapWikidotUrl("https://scp-jp.wikidot.com/scp-173").status,
    "unmapped_scp-jp.wikidot.com"
  );
  assert.equal(mapWikidotUrl("not a url").status, "unmapped_invalid_url");
});
