#!/usr/bin/env node

import {createHash} from "node:crypto";
import fs from "node:fs/promises";

import {runCliIfMain} from "../src/cli-entry.mjs";

const SCHEMA = "wikijump_syntax_differential.runtime_state_fixture.v1";

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function valueAfter(argv, index, option) {
  const value = argv[index + 1];
  if (value == null || value.startsWith("--")) throw new Error(`${option} requires a value`);
  return value;
}

export function parseArgs(argv) {
  const args = {
    report: null,
    categoriesCases: [],
    absentPages: [],
    output: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--report") args.report = valueAfter(argv, index++, option);
    else if (option === "--categories-case") {
      args.categoriesCases.push(valueAfter(argv, index++, option));
    } else if (option === "--absent-page") {
      args.absentPages.push(valueAfter(argv, index++, option));
    } else if (option === "--output") args.output = valueAfter(argv, index++, option);
    else throw new Error(`unknown option: ${option}`);
  }
  if (!args.report) throw new Error("--report is required");
  if (args.categoriesCases.length === 0 && args.absentPages.length === 0) {
    throw new Error("--categories-case or --absent-page is required");
  }
  if (!args.output) throw new Error("--output is required");
  return args;
}

function splitPageKey(value) {
  const separator = value.indexOf(":");
  if (separator <= 0 || separator === value.length - 1) {
    throw new Error(`page identity must be site:slug: ${value}`);
  }
  const site = value.slice(0, separator).toLowerCase();
  const slug = value.slice(separator + 1).toLowerCase();
  if (!/^[a-z0-9-]+$/u.test(site) || !/^[a-z0-9:_-]+$/u.test(slug)) {
    throw new Error(`invalid page identity: ${value}`);
  }
  return {site, slug};
}

function categoriesFromComparison(comparison) {
  const html = comparison?.diagnostic?.wikidot_html;
  if (typeof html !== "string") {
    throw new Error(`category comparison has no live HTML: ${comparison?.case_id}`);
  }
  const categories = new Map();
  const pattern = /<h3>(?<slug>[^<]+)<\/h3>\s*<a[^>]+id="category-pages-toggler-(?<id>\d+)"/gu;
  for (const match of html.matchAll(pattern)) {
    const slug = match.groups.slug;
    const oracleId = Number(match.groups.id);
    const previous = categories.get(slug);
    if (previous != null && previous !== oracleId) {
      throw new Error(`category identity changed within live capture: ${slug}`);
    }
    categories.set(slug, oracleId);
  }
  if (categories.size === 0) {
    throw new Error(`category comparison yielded no categories: ${comparison.case_id}`);
  }
  return [...categories].map(([slug, oracle_id]) => ({
    site: "sandbox-for-codex",
    slug,
    oracle_id,
  }));
}

export async function main(argv) {
  const args = parseArgs(argv);
  try {
    await fs.access(args.output);
    throw new Error(`output already exists: ${args.output}`);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  const reportBytes = await fs.readFile(args.report);
  const report = JSON.parse(reportBytes);
  const comparisons = new Map(report.comparisons.map((value) => [value.case_id, value]));
  const categoryEntries = new Map();
  const categoryEvidence = [];
  for (const caseId of args.categoriesCases) {
    const comparison = comparisons.get(caseId);
    if (!comparison) throw new Error(`categories case is absent from report: ${caseId}`);
    for (const category of categoriesFromComparison(comparison)) {
      const key = `${category.site}:${category.slug}`;
      const previous = categoryEntries.get(key);
      if (previous && previous.oracle_id !== category.oracle_id) {
        throw new Error(`category identity differs between cases: ${key}`);
      }
      categoryEntries.set(key, category);
    }
    categoryEvidence.push({
      case_id: caseId,
      capture_file: comparison.identities?.capture_file,
      capture_line: comparison.identities?.capture_line,
      page_identity: comparison.identities?.page_identity,
      wikidot_html_sha256: comparison.identities?.wikidot_html_sha256,
    });
  }
  const fixture = {
    schema: SCHEMA,
    captured_at: new Date().toISOString(),
    capture_source: {
      kind: "frozen-live-reference",
      report: args.report,
      report_sha256: sha256(reportBytes),
      category_evidence: categoryEvidence,
    },
    roots: [],
    pages: [],
    unresolved_pages: [],
    absent_pages: args.absentPages.map(splitPageKey),
    categories: [...categoryEntries.values()],
  };
  await fs.writeFile(args.output, `${JSON.stringify(fixture, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: 0o600,
  });
  process.stdout.write(`${JSON.stringify({
    output: args.output,
    categories: fixture.categories.length,
    absent_pages: fixture.absent_pages.length,
    sha256: sha256(await fs.readFile(args.output)),
  })}\n`);
  return 0;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error.stack ?? error);
    return 2;
  },
});
