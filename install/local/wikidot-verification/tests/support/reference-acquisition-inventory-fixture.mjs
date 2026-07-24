import {sha256Hex, stableStringify} from "../../src/corpus-import-manifest.mjs";

export const SOURCE_ORIGIN = "https://scp-wiki.wikidot.com";

function row(fullname, sourceEntityId, digestCharacter, attachments = []) {
  return {
    attachments,
    fullname,
    meta_sha256: digestCharacter.repeat(64),
    parent_fullname: null,
    revisions: 3,
    source_branch: "en",
    source_entity_id: sourceEntityId,
    source_sha256: (digestCharacter === "a" ? "b" : "a").repeat(64),
    source_site: "scp-wiki",
    updated_at: "2026-07-18T12:34:56+00:00",
  };
}

export function referenceAttachment(fullname = "alpha") {
  return {
    corpus_path: `/ignored/${fullname}`,
    file_path: `/host/path/${fullname}`,
    filename: "image one.png",
    metadata_path: "/host/path/_state.json",
    mime: "image/png",
    original_url: `http://scp-wiki.wdfiles.com/local--files/${fullname}/image%20one.png`,
    sha256: "c".repeat(64),
    size: 123,
    wikidot_path: `/local--files/${fullname}/image%20one.png`,
  };
}

export function inventoryFixtureInputs(rows) {
  const manifestText = `${rows.map((value) => stableStringify(value)).join("\n")}\n`;
  const summary = {
    attachment_count: rows.reduce(
      (count, value) => count + (value.attachments?.length ?? 0),
      0,
    ),
    attachment_page_count: rows.filter(
      (value) => (value.attachments?.length ?? 0) > 0,
    ).length,
    first_fullname: rows[0].fullname,
    last_fullname: rows.at(-1).fullname,
    manifest_sha256: sha256Hex(manifestText),
    parent_count: rows.filter((value) => value.parent_fullname !== null).length,
    required_browser_count: rows.filter(
      (value) => value.required_browser === true,
    ).length,
    row_count: rows.length,
    source_browser_visibility_counts: {},
    source_branches: ["en"],
    source_required_actor_count: 0,
    source_sites: ["scp-wiki"],
  };
  const summaryBytes = Buffer.from(`${stableStringify(summary)}\n`);
  return {
    expectedCount: rows.length,
    expectedManifestSha256: summary.manifest_sha256,
    expectedSummarySha256: sha256Hex(summaryBytes),
    manifestBytes: Buffer.from(manifestText),
    summaryBytes,
  };
}

export const TWO_REFERENCE_ROWS = [
  row("alpha", "00000000-0000-0000-0000-000000000001", "a", [referenceAttachment()]),
  row("theme:雪 space", "00000000-0000-0000-0000-000000000002", "b"),
];
