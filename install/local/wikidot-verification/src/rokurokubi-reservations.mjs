import { createHash } from "node:crypto";

export const DEFAULT_CANONICAL_MIRROR_HOST = "scp-wiki.wikijump.localhost:18443";

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
  "source_row",
  "timestamp",
  "translator",
  "source_url",
  "title",
  "deadline",
  "branch_code",
  "notes",
  "wikijump_mirror_url",
  "mirror_url_status"
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

export function mapWikidotUrl(sourceUrl, mirrorHost = DEFAULT_CANONICAL_MIRROR_HOST) {
  let parsed;
  try {
    parsed = new URL(sourceUrl);
  } catch {
    return { slug: "", mirrorUrl: "", status: "unmapped_invalid_url" };
  }

  let slug;
  try {
    slug = decodeURIComponent(parsed.pathname.replace(/^\/+|\/+$/gu, ""));
  } catch {
    return { slug: "", mirrorUrl: "", status: "unmapped_invalid_slug_encoding" };
  }
  if (!slug) {
    return { slug: "", mirrorUrl: "", status: "unmapped_missing_slug" };
  }

  if (parsed.hostname !== "scp-wiki.wikidot.com") {
    return { slug, mirrorUrl: "", status: `unmapped_${parsed.hostname}` };
  }

  return {
    slug,
    mirrorUrl: `http://${mirrorHost}/${encodeURI(slug)}`,
    status: "mapped_scp-wiki"
  };
}

export function extractRokurokubiReservations(csvText, options = {}) {
  const mirrorHost = options.mirrorHost ?? DEFAULT_CANONICAL_MIRROR_HOST;
  const rows = parseCsv(csvText);
  if (rows.length === 0) {
    throw new Error("CSV is empty");
  }

  const hasContent = (row) => row.some((value) => value.trim() !== "");
  const headerRowIndex = rows.findIndex(hasContent);
  if (headerRowIndex === -1) {
    throw new Error("CSV is empty");
  }

  const headers = rows[headerRowIndex];
  const index = headerIndex(headers);
  const outputRows = [];

  rows.slice(headerRowIndex + 1).forEach((row, offset) => {
    if (!hasContent(row)) {
      return;
    }

    const translator = getValue(row, index, "翻訳者名");
    if (translator.toLowerCase() !== "rokurokubi") {
      return;
    }

    const sourceUrl = getValue(row, index, "記事のURL");
    const { mirrorUrl, status } = mapWikidotUrl(sourceUrl, mirrorHost);
    outputRows.push({
      source_row: String(headerRowIndex + offset + 2),
      timestamp: getValue(row, index, "タイムスタンプ"),
      translator,
      source_url: sourceUrl,
      title: getValue(row, index, "記事のタイトル"),
      deadline: getValue(row, index, "翻訳完了期限"),
      branch_code: getValue(row, index, "支部コード"),
      notes: getValue(row, index, "備考"),
      wikijump_mirror_url: mirrorUrl,
      mirror_url_status: status
    });
  });

  const manifest = {
    schema_version: 1,
    source_csv_sha256: sha256Hex(csvText),
    source_row_count_excluding_header: rows.length - headerRowIndex - 1,
    rokurokubi_row_count: outputRows.length,
    mapped_scp_wiki_count: outputRows.filter((row) => row.mirror_url_status === "mapped_scp-wiki").length,
    unmapped_count: outputRows.filter((row) => !row.mirror_url_status.startsWith("mapped_")).length,
    canonical_mirror_host: mirrorHost
  };

  return { rows: outputRows, manifest };
}

export { OUTPUT_HEADERS };
