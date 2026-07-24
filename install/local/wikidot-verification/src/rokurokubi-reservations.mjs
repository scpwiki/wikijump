import { createHash } from "node:crypto";

export const DEFAULT_CANONICAL_MIRROR_ORIGIN = "https://scp-wiki.wikijump.localhost";
export const DEFAULT_CANONICAL_MIRROR_HOST = "scp-wiki.wikijump.localhost";

const REQUIRED_HEADERS = [
  "タイムスタンプ",
  "翻訳者名",
  "記事のURL",
  "記事のタイトル",
  "翻訳完了期限",
  "支部コード",
  "備考"
];

const OUTPUT_HEADERS = [
  "sheet_roles",
  "sheet_names",
  "sheet_gids",
  "source_rows",
  "timestamps",
  "translators",
  "source_url",
  "title",
  "deadlines",
  "branch_codes",
  "notes",
  "normalized_path",
  "wikijump_mirror_url",
  "mirror_url_status",
  "provenance_count"
];

export function sha256Hex(text) {
  return createHash("sha256").update(text, "utf8").digest("hex");
}

export function parseCsv(text) {
  const rows = [];
  let row = [];
  let field = "";
  let inQuotes = false;

  for (let i = 0; i < text.length; i += 1) {
    const ch = text[i];
    const next = text[i + 1];

    if (inQuotes) {
      if (ch === '"' && next === '"') {
        field += '"';
        i += 1;
      } else if (ch === '"') {
        inQuotes = false;
      } else {
        field += ch;
      }
      continue;
    }

    if (ch === '"') {
      inQuotes = true;
    } else if (ch === ",") {
      row.push(field);
      field = "";
    } else if (ch === "\n") {
      row.push(field);
      rows.push(row);
      row = [];
      field = "";
    } else if (ch !== "\r") {
      field += ch;
    }
  }

  if (field !== "" || row.length > 0) {
    row.push(field);
    rows.push(row);
  }

  return rows;
}

export function formatCsv(rows, headers = OUTPUT_HEADERS) {
  return [headers, ...rows.map((row) => headers.map((header) => row[header] ?? ""))]
    .map((row) => row.map(escapeCsvField).join(","))
    .join("\n") + "\n";
}

function escapeCsvField(value) {
  const text = String(value);
  if (/[",\r\n]/u.test(text)) {
    return `"${text.replaceAll('"', '""')}"`;
  }
  return text;
}

function headerIndex(headers) {
  const index = new Map();
  headers.forEach((header, position) => index.set(header, position));
  for (const required of REQUIRED_HEADERS) {
    if (!index.has(required)) {
      throw new Error(`Missing required CSV header: ${required}`);
    }
  }
  return index;
}

function getValue(row, index, header) {
  return (row[index.get(header)] ?? "").trim();
}

function normalizeMirrorOrigin(value = DEFAULT_CANONICAL_MIRROR_ORIGIN) {
  const trimmed = String(value || DEFAULT_CANONICAL_MIRROR_ORIGIN).trim().replace(/\/+$/u, "");
  if (/^[a-z][a-z0-9+.-]*:\/\//iu.test(trimmed)) {
    return trimmed;
  }
  return `https://${trimmed}`;
}

function normalizePathname(pathname) {
  const trimmed = pathname.replace(/^\/+|\/+$/gu, "");
  try {
    return decodeURIComponent(trimmed);
  } catch {
    return null;
  }
}

function encodeMirrorPath(pathname) {
  const segments = pathname.split("/");
  if (segments.some((segment) => segment === "." || segment === "..")) {
    return null;
  }
  return segments.map((segment) => encodeURIComponent(segment)).join("/");
}

function rawPathnameFromUrl(sourceUrl) {
  const match = String(sourceUrl).match(/^[a-z][a-z0-9+.-]*:\/\/[^/?#]*([^?#]*)/iu);
  if (!match) {
    return null;
  }
  return match[1] || "/";
}

function normalizeSourceKey({ sourceUrl, slug, status, sheetRole, sheetName, sheetGid, sourceRow }) {
  if (status === "mapped_scp-wiki" && slug) {
    return `scp-wiki/${slug.toLowerCase()}`;
  }

  try {
    const parsed = new URL(sourceUrl);
    const normalizedPathname = normalizePathname(rawPathnameFromUrl(sourceUrl) ?? parsed.pathname);
    if (normalizedPathname) {
      return `${parsed.hostname.toLowerCase()}/${normalizedPathname.toLowerCase()}`;
    }
  } catch (error) {
    void error;
    // Fall through to provenance key when the URL has no stable normalized identity.
  }

  return `unmapped/${sheetRole}/${sheetName}/${sheetGid}/${sourceRow}`.toLowerCase();
}

function appendJoined(existing, next) {
  const text = String(next ?? "").trim();
  if (!text) {
    return existing;
  }
  if (!existing) {
    return text;
  }
  return `${existing} | ${text}`;
}

function sheetValue(sheet, key, fallback = "") {
  return String(sheet[key] ?? fallback ?? "").trim();
}

export function mapWikidotUrl(sourceUrl, mirrorOrigin = DEFAULT_CANONICAL_MIRROR_ORIGIN) {
  let parsed;
  try {
    parsed = new URL(sourceUrl);
  } catch {
    return { slug: "", mirrorUrl: "", status: "unmapped_invalid_url" };
  }

  const slug = normalizePathname(rawPathnameFromUrl(sourceUrl) ?? parsed.pathname);
  if (slug === null) {
    return { slug: "", mirrorUrl: "", status: "unmapped_invalid_slug_encoding" };
  }
  if (!slug) {
    return { slug: "", mirrorUrl: "", status: "unmapped_missing_slug" };
  }

  if (parsed.hostname !== "scp-wiki.wikidot.com") {
    return { slug, mirrorUrl: "", status: `unmapped_${parsed.hostname}` };
  }

  const mirrorPath = encodeMirrorPath(slug);
  if (mirrorPath === null) {
    return { slug, mirrorUrl: "", status: "unmapped_invalid_slug_dot_segment" };
  }

  return {
    slug,
    mirrorUrl: `${normalizeMirrorOrigin(mirrorOrigin)}/${mirrorPath}`,
    status: "mapped_scp-wiki"
  };
}

function extractRowsFromSheet(sheet, options) {
  const mirrorOrigin = options.mirrorOrigin ?? options.mirrorHost ?? DEFAULT_CANONICAL_MIRROR_ORIGIN;
  const csvText = sheet.csvText;
  const rows = parseCsv(csvText);
  if (rows.length === 0) {
    throw new Error(`CSV is empty: ${sheetValue(sheet, "name", sheet.csvPath)}`);
  }

  const hasContent = (row) => row.some((value) => value.trim() !== "");
  const headerRowIndex = rows.findIndex(hasContent);
  if (headerRowIndex === -1) {
    throw new Error("CSV is empty");
  }

  const headers = rows[headerRowIndex];
  const index = headerIndex(headers);
  const outputRows = [];
  const sheetRole = sheetValue(sheet, "role");
  const sheetName = sheetValue(sheet, "name", sheetValue(sheet, "label", sheet.csvPath));
  const sheetGid = sheetValue(sheet, "gid", "unknown");

  rows.slice(headerRowIndex + 1).forEach((row, offset) => {
    if (!hasContent(row)) {
      return;
    }

    const translator = getValue(row, index, "翻訳者名");
    if (translator.trim().toLowerCase() !== "rokurokubi") {
      return;
    }

    const sourceUrl = getValue(row, index, "記事のURL");
    const sourceRow = String(headerRowIndex + offset + 2);
    const { slug, mirrorUrl, status } = mapWikidotUrl(sourceUrl, mirrorOrigin);
    outputRows.push({
      sheet_roles: sheetRole,
      sheet_names: sheetName,
      sheet_gids: sheetGid,
      source_rows: sourceRow,
      timestamps: getValue(row, index, "タイムスタンプ"),
      translators: translator,
      source_url: sourceUrl,
      title: getValue(row, index, "記事のタイトル"),
      deadlines: getValue(row, index, "翻訳完了期限"),
      branch_codes: getValue(row, index, "支部コード"),
      notes: getValue(row, index, "備考"),
      normalized_path: slug,
      wikijump_mirror_url: mirrorUrl,
      mirror_url_status: status,
      provenance_count: "1",
      source_key: normalizeSourceKey({ sourceUrl, slug, status, sheetRole, sheetName, sheetGid, sourceRow })
    });
  });

  const mappedCount = outputRows.filter((row) => row.mirror_url_status === "mapped_scp-wiki").length;
  return {
    rows: outputRows,
    summary: {
      role: sheetRole,
      name: sheetName,
      gid: sheetGid,
      label: sheetValue(sheet, "label", sheet.csvPath),
      csv: sheet.csvPath,
      source_csv_sha256: sha256Hex(csvText),
      source_row_count_excluding_header: rows.length - headerRowIndex - 1,
      rokurokubi_row_count: outputRows.length,
      mapped_scp_wiki_count: mappedCount,
      unmapped_count: outputRows.length - mappedCount
    }
  };
}

function dedupeRows(rows) {
  const byKey = new Map();

  for (const row of rows) {
    const existing = byKey.get(row.source_key);
    if (!existing) {
      const publicRow = {...row};
      delete publicRow.source_key;
      byKey.set(row.source_key, publicRow);
      continue;
    }

    existing.sheet_roles = appendJoined(existing.sheet_roles, row.sheet_roles);
    existing.sheet_names = appendJoined(existing.sheet_names, row.sheet_names);
    existing.sheet_gids = appendJoined(existing.sheet_gids, row.sheet_gids);
    existing.source_rows = appendJoined(existing.source_rows, row.source_rows);
    existing.timestamps = appendJoined(existing.timestamps, row.timestamps);
    existing.translators = appendJoined(existing.translators, row.translators);
    existing.deadlines = appendJoined(existing.deadlines, row.deadlines);
    existing.branch_codes = appendJoined(existing.branch_codes, row.branch_codes);
    existing.notes = appendJoined(existing.notes, row.notes);
    existing.provenance_count = String(Number(existing.provenance_count) + 1);
  }

  return [...byKey.values()];
}

export function extractRokurokubiReservationsFromSheets(sheets, options = {}) {
  if (!Array.isArray(sheets) || sheets.length === 0) {
    throw new Error("At least one source sheet is required");
  }

  const extracted = sheets.map((sheet) => extractRowsFromSheet(sheet, options));
  const allRows = extracted.flatMap((sheet) => sheet.rows);
  const rows = dedupeRows(allRows);
  const mappedCount = rows.filter((row) => row.mirror_url_status === "mapped_scp-wiki").length;
  const targetEnRows = rows.filter(
    (row) => row.mirror_url_status === "mapped_scp-wiki" && row.branch_codes.split(" | ").some((code) => code.trim() === "EN")
  ).length;

  const manifest = {
    schema_version: 2,
    source_sheets: extracted.map((sheet) => sheet.summary),
    source_row_count_excluding_header: extracted.reduce(
      (total, sheet) => total + sheet.summary.source_row_count_excluding_header,
      0
    ),
    rokurokubi_row_count_before_deduplication: allRows.length,
    rokurokubi_row_count: rows.length,
    target_en_row_count: targetEnRows,
    mapped_scp_wiki_count: mappedCount,
    unmapped_count: rows.length - mappedCount,
    canonical_mirror_origin: normalizeMirrorOrigin(options.mirrorOrigin ?? options.mirrorHost)
  };

  return { rows, manifest };
}

export function extractRokurokubiReservations(csvText, options = {}) {
  const sheet = {
    csvText,
    csvPath: options.sourcePath,
    role: options.sourceRole,
    name: options.sourceName ?? options.sourceLabel,
    gid: options.sourceGid,
    label: options.sourceLabel
  };
  return extractRokurokubiReservationsFromSheets([sheet], options);
}

export { OUTPUT_HEADERS };
