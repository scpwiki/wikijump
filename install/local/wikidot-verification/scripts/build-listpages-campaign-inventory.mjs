#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";

import {
  buildListPagesCampaignInventory,
} from "../src/listpages-campaign-inventory.mjs";

function nextValue(argv, index, option) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) throw new Error(`missing value for ${option}`);
  return value;
}

export function parseArgs(argv) {
  const args = {
    docsRoot: "/home/roku/src/Rokurolize/scp-wiki-translation/corpus/www/pages",
    corpusRoot: "/home/roku/src/Rokurolize/scp-wiki-translation/corpus",
    outputDir: null,
    branches: null,
  };

  for (let index = 2; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--docs-root") {
      args.docsRoot = path.resolve(nextValue(argv, index, option));
      index += 1;
    } else if (option === "--corpus-root") {
      args.corpusRoot = path.resolve(nextValue(argv, index, option));
      index += 1;
    } else if (option === "--output-dir") {
      args.outputDir = path.resolve(nextValue(argv, index, option));
      index += 1;
    } else if (option === "--branch") {
      args.branches ??= [];
      args.branches.push(nextValue(argv, index, option));
      index += 1;
    } else if (option === "--help" || option === "-h") {
      return { help: true };
    } else {
      throw new Error(`unknown argument: ${option}`);
    }
  }

  if (!args.outputDir) throw new Error("--output-dir is required");
  return args;
}

function printHelp() {
  console.log(`Usage: node install/local/wikidot-verification/scripts/build-listpages-campaign-inventory.mjs --output-dir DIR [--branch en]...`);
}

function jsonLine(record) {
  return `${JSON.stringify(record)}\n`;
}

async function writeJson(filePath, value) {
  await fs.writeFile(filePath, `${JSON.stringify(value, null, 2)}\n`, { mode: 0o600 });
}

async function writeJsonl(filePath, rows) {
  await fs.writeFile(filePath, rows.map(jsonLine).join(""), { mode: 0o600 });
}

export async function main(argv = process.argv) {
  const args = parseArgs(argv);
  if (args.help) {
    printHelp();
    return;
  }

  const inventory = await buildListPagesCampaignInventory({
    docsRoot: args.docsRoot,
    corpusRoot: args.corpusRoot,
    branches: args.branches,
    onProgress(event) {
      if (event.phase === "corpus-branch-candidates") {
        console.error(
          `corpus ${event.branch}: ${event.candidate_source_count}/${event.source_page_count} candidate pages`,
        );
      }
    },
  });

  await fs.mkdir(args.outputDir, { recursive: true, mode: 0o700 });
  const manifest = {
    schema: inventory.schema,
    generated_at: inventory.generated_at,
    docs_root: inventory.docs.docs_root,
    corpus_root: inventory.corpus.corpus_root,
    files: {
      documentation_inventory: "documentation-inventory.json",
      documentation_claims: "documentation-claims.jsonl",
      documentation_missing_references: "documentation-missing-references.jsonl",
      corpus_invocations: "corpus-listpages-invocations.jsonl",
      corpus_clusters: "corpus-listpages-clusters.json",
      summary: "summary.json",
    },
    summary: inventory.summary,
  };
  await Promise.all([
    writeJson(path.join(args.outputDir, "campaign-inventory.json"), manifest),
    writeJson(path.join(args.outputDir, "documentation-inventory.json"), inventory.docs),
    writeJsonl(path.join(args.outputDir, "documentation-claims.jsonl"), inventory.docs.claims),
    writeJsonl(
      path.join(args.outputDir, "documentation-missing-references.jsonl"),
      inventory.docs.missing_references,
    ),
    writeJsonl(
      path.join(args.outputDir, "corpus-listpages-invocations.jsonl"),
      inventory.corpus.invocations,
    ),
    writeJson(path.join(args.outputDir, "corpus-listpages-clusters.json"), {
      schema: `${inventory.corpus.schema}.clusters`,
      generated_at: inventory.corpus.generated_at,
      corpus_root: inventory.corpus.corpus_root,
      clusters: inventory.corpus.clusters,
      summary: {
        cluster_count: inventory.corpus.clusters.length,
        invocation_count: inventory.corpus.summary.invocation_count,
      },
    }),
    writeJson(path.join(args.outputDir, "summary.json"), inventory.summary),
  ]);

  console.log(JSON.stringify({
    output_dir: args.outputDir,
    docs: inventory.docs.summary,
    corpus: inventory.corpus.summary,
  }));
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
