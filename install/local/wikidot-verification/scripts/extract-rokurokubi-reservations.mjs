#!/usr/bin/env node
import { mkdir, readFile, writeFile } from "node:fs/promises";
import {runCliIfMain} from "../src/cli-entry.mjs";
import { dirname } from "node:path";
import {
  DEFAULT_CANONICAL_MIRROR_ORIGIN,
  extractRokurokubiReservations,
  extractRokurokubiReservationsFromSheets,
  formatCsv,
  sha256Hex
} from "../src/rokurokubi-reservations.mjs";

export function parseArgs(argv) {
  const args = {
    mirrorOrigin: DEFAULT_CANONICAL_MIRROR_ORIGIN,
    sourceLabel: null,
    sourceRole: null,
    sourceName: null,
    sourceGid: null,
    sheetManifest: null
  };
  const optionTokens = new Set([
    "--source",
    "--sheet-manifest",
    "--output",
    "--manifest",
    "--mirror-origin",
    "--mirror-host",
    "--source-label",
    "--source-role",
    "--source-name",
    "--source-gid",
    "--help",
    "-h"
  ]);

  const nextValue = (flag, index) => {
    let value = argv[index + 1];
    if (value === "--") {
      argv.splice(index + 1, 1);
      value = argv[index + 1];
    }
    if (!value || optionTokens.has(value) || value.startsWith("--")) {
      throw new Error(`${flag} requires a value`);
    }
    return value;
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--source") {
      args.source = nextValue(arg, i);
      i += 1;
    } else if (arg === "--sheet-manifest") {
      args.sheetManifest = nextValue(arg, i);
      i += 1;
    } else if (arg === "--output") {
      args.output = nextValue(arg, i);
      i += 1;
    } else if (arg === "--manifest") {
      args.manifest = nextValue(arg, i);
      i += 1;
    } else if (arg === "--mirror-origin") {
      args.mirrorOrigin = nextValue(arg, i);
      i += 1;
    } else if (arg === "--mirror-host") {
      args.mirrorOrigin = nextValue(arg, i);
      i += 1;
    } else if (arg === "--source-label") {
      args.sourceLabel = nextValue(arg, i);
      i += 1;
    } else if (arg === "--source-role") {
      args.sourceRole = nextValue(arg, i);
      i += 1;
    } else if (arg === "--source-name") {
      args.sourceName = nextValue(arg, i);
      i += 1;
    } else if (arg === "--source-gid") {
      args.sourceGid = nextValue(arg, i);
      i += 1;
    } else if (arg === "--help" || arg === "-h") {
      args.help = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return args;
}

export function usage() {
  return `Usage:
  node install/local/wikidot-verification/scripts/extract-rokurokubi-reservations.mjs \
    --source <csv> --output <csv> --manifest <json> \
    [--mirror-origin <origin>] [--source-label <label>] [--source-role <role>] [--source-name <name>] [--source-gid <gid>]

  node install/local/wikidot-verification/scripts/extract-rokurokubi-reservations.mjs \
    --sheet-manifest <json> --output <csv> --manifest <json> [--mirror-origin <origin>]

Sheet manifest format:
  {
    "sheets": [
      {"csv": "/path/to/main.csv", "role": "active", "name": "Main", "gid": "1325361212", "label": "..."},
      {"csv": "/path/to/expired.csv", "role": "expired", "name": "期限切れ削除済", "gid": "unknown", "label": "..."}
    ]
  }`;
}

async function writeEnsuringParent(path, text) {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, text, "utf8");
}

function requireValue(value, message) {
  if (!value) {
    throw new Error(message);
  }
  return value;
}

async function loadSheetManifest(path) {
  const text = await readFile(path, "utf8");
  const parsed = JSON.parse(text);
  if (!Array.isArray(parsed.sheets) || parsed.sheets.length === 0) {
    throw new Error("Sheet manifest must contain a non-empty sheets array");
  }

  const sheets = [];
  for (const [index, sheet] of parsed.sheets.entries()) {
    const csvPath = requireValue(sheet.csv ?? sheet.source ?? sheet.path, `Sheet ${index + 1} is missing csv/source/path`);
    const csvText = await readFile(csvPath, "utf8");
    const metadata = {...sheet};
    delete metadata.csv;
    delete metadata.source;
    delete metadata.path;
    sheets.push({
      ...metadata,
      csvText,
      csvPath,
      role: sheet.role,
      name: sheet.name,
      gid: sheet.gid,
      label: sheet.label
    });
  }
  const metadata = {...parsed};
  delete metadata.sheets;
  return { sheets, metadata };
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  if (!args.output || !args.manifest) {
    throw new Error(usage());
  }
  if (args.source && args.sheetManifest) {
    throw new Error("Use either --source or --sheet-manifest, not both");
  }

  let rows;
  let manifest;
  if (args.sheetManifest) {
    const sheetManifest = await loadSheetManifest(args.sheetManifest);
    ({ rows, manifest } = extractRokurokubiReservationsFromSheets(sheetManifest.sheets, {
      mirrorOrigin: args.mirrorOrigin
    }));
    manifest = {
      ...sheetManifest.metadata,
      ...manifest,
      source_sheets: manifest.source_sheets.map((summary, index) => ({
        ...sheetManifest.sheets[index],
        csvText: undefined,
        ...summary
      }))
    };
    manifest.source_sheet_manifest = args.sheetManifest;
  } else if (args.source) {
    const csvText = await readFile(args.source, "utf8");
    ({ rows, manifest } = extractRokurokubiReservations(csvText, {
      mirrorOrigin: args.mirrorOrigin,
      sourcePath: args.source,
      sourceRole: args.sourceRole,
      sourceName: args.sourceName,
      sourceGid: args.sourceGid,
      sourceLabel: args.sourceLabel
    }));
    manifest.source_sheet = args.sourceLabel ?? args.source;
  } else {
    throw new Error(usage());
  }

  const outputCsv = formatCsv(rows);
  const enrichedManifest = {
    ...manifest,
    output_csv: args.output,
    ledger_csv_sha256: sha256Hex(outputCsv)
  };

  await writeEnsuringParent(args.output, outputCsv);
  await writeEnsuringParent(args.manifest, `${JSON.stringify(enrichedManifest, null, 2)}\n`);

  console.log(JSON.stringify({
    output: args.output,
    manifest: args.manifest,
    rokurokubi_row_count: enrichedManifest.rokurokubi_row_count,
    mapped_scp_wiki_count: enrichedManifest.mapped_scp_wiki_count,
    unmapped_count: enrichedManifest.unmapped_count
  }));
  return 0;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error.message);
    return 1;
  },
});
