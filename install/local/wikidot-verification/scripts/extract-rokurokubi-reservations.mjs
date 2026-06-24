#!/usr/bin/env node
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname } from "node:path";
import {
  DEFAULT_CANONICAL_MIRROR_HOST,
  extractRokurokubiReservations,
  formatCsv
} from "../src/rokurokubi-reservations.mjs";

function parseArgs(argv) {
  const args = {
    mirrorHost: DEFAULT_CANONICAL_MIRROR_HOST,
    sourceLabel: null
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = argv[i + 1];
    if (arg === "--source") {
      args.source = next;
      i += 1;
    } else if (arg === "--output") {
      args.output = next;
      i += 1;
    } else if (arg === "--manifest") {
      args.manifest = next;
      i += 1;
    } else if (arg === "--mirror-host") {
      args.mirrorHost = next;
      i += 1;
    } else if (arg === "--source-label") {
      args.sourceLabel = next;
      i += 1;
    } else if (arg === "--help" || arg === "-h") {
      args.help = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return args;
}

function usage() {
  return `Usage: node scripts/extract-rokurokubi-reservations.mjs --source <csv> --output <csv> --manifest <json> [--mirror-host <host>] [--source-label <label>]`;
}

async function writeEnsuringParent(path, text) {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, text, "utf8");
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    console.log(usage());
    return;
  }
  if (!args.source || !args.output || !args.manifest) {
    throw new Error(usage());
  }

  const csvText = await readFile(args.source, "utf8");
  const { rows, manifest } = extractRokurokubiReservations(csvText, {
    mirrorHost: args.mirrorHost
  });

  const enrichedManifest = {
    ...manifest,
    source_sheet: args.sourceLabel ?? args.source,
    output_csv: args.output
  };

  await writeEnsuringParent(args.output, formatCsv(rows));
  await writeEnsuringParent(args.manifest, `${JSON.stringify(enrichedManifest, null, 2)}\n`);

  console.log(`wrote ${rows.length} rokurokubi reservation rows to ${args.output}`);
}

main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
