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
    usersCases: [],
    absentPages: [],
    output: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--report") args.report = valueAfter(argv, index++, option);
    else if (option === "--categories-case") {
      args.categoriesCases.push(valueAfter(argv, index++, option));
    } else if (option === "--users-case") {
      args.usersCases.push(valueAfter(argv, index++, option));
    } else if (option === "--absent-page") {
      args.absentPages.push(valueAfter(argv, index++, option));
    } else if (option === "--output") args.output = valueAfter(argv, index++, option);
    else throw new Error(`unknown option: ${option}`);
  }
  if (!args.report) throw new Error("--report is required");
  if (
    args.categoriesCases.length === 0 &&
    args.usersCases.length === 0 &&
    args.absentPages.length === 0
  ) {
    throw new Error("--categories-case, --users-case, or --absent-page is required");
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

export function wikidotUsersFromComparison(comparison, provenance) {
  const html = comparison?.diagnostic?.wikidot_html;
  if (typeof html !== "string") {
    throw new Error(`user comparison has no live HTML: ${comparison?.case_id}`);
  }
  const pattern = /<span class="printuser avatarhover"><a href="http:\/\/www\.wikidot\.com\/user:info\/(?<slug>[a-z0-9-]+)" onclick="WIKIDOT\.page\.listeners\.userInfo\((?<id>\d+)\); return false;"><img alt="(?<name>[^"<]+)" class="small" src="http:\/\/www\.wikidot\.com\/avatar\.php\?userid=(?<avatarId>\d+)&amp;amp;size=small&amp;amp;timestamp=\d+" style="background-image:url\(http:\/\/www\.wikidot\.com\/userkarma\.php\?u=(?<karmaId>\d+)\)"><\/a><a href="http:\/\/www\.wikidot\.com\/user:info\/(?<secondSlug>[a-z0-9-]+)" onclick="WIKIDOT\.page\.listeners\.userInfo\((?<secondId>\d+)\); return false;">(?<secondName>[^<]+)<\/a><\/span>/gu;
  const users = [];
  for (const match of html.matchAll(pattern)) {
    const {
      id,
      avatarId,
      karmaId,
      secondId,
      name,
      secondName,
      slug,
      secondSlug,
    } = match.groups;
    const userId = Number(id);
    if (
      !Number.isSafeInteger(userId) ||
      userId <= 0 ||
      userId > 2_147_483_647 ||
      id !== avatarId ||
      id !== karmaId ||
      id !== secondId ||
      name !== secondName ||
      slug !== secondSlug
    ) {
      throw new Error(`printuser identity is internally inconsistent: ${comparison.case_id}`);
    }
    users.push({user_id: userId, name, slug, provenance: {...provenance}});
  }
  const printuserCount = [...html.matchAll(/<span class="[^"]*\bprintuser\b[^"]*">/gu)].length;
  if (users.length === 0 || users.length !== printuserCount) {
    throw new Error(`user comparison contains an unsupported printuser shape: ${comparison.case_id}`);
  }
  return users;
}

async function comparisonProvenance(comparison) {
  const {
    capture_file: captureFile,
    capture_line: captureLine,
    page_identity: pageIdentity,
    wikidot_html_sha256: wikidotHtmlSha256,
    saved_source_sha256: savedSourceSha256,
  } = comparison.identities ?? {};
  if (
    typeof captureFile !== "string" ||
    !Number.isSafeInteger(captureLine) ||
    captureLine <= 0 ||
    !Number.isSafeInteger(pageIdentity) ||
    pageIdentity <= 0 ||
    !/^[0-9a-f]{64}$/u.test(wikidotHtmlSha256 ?? "") ||
    !/^[0-9a-f]{64}$/u.test(savedSourceSha256 ?? "") ||
    sha256(comparison.diagnostic?.wikidot_html ?? "") !== wikidotHtmlSha256
  ) {
    throw new Error(`user comparison provenance is incomplete: ${comparison.case_id}`);
  }
  const captureBytes = await fs.readFile(captureFile);
  const lines = captureBytes.toString("utf8").split("\n");
  const line = lines[captureLine - 1];
  if (!line) throw new Error(`user comparison capture line is absent: ${comparison.case_id}`);
  const capture = JSON.parse(line);
  if (
    capture.capture_status !== "captured" ||
    capture.page_identity !== pageIdentity ||
    capture.saved_source_sha256 !== savedSourceSha256 ||
    Number.isNaN(Date.parse(capture.captured_at))
  ) {
    throw new Error(`user comparison capture identity changed: ${comparison.case_id}`);
  }
  return {
    source: captureFile,
    capture_file_sha256: sha256(captureBytes),
    captured_at: capture.captured_at,
    capture_line: captureLine,
    page_identity: pageIdentity,
    saved_source_sha256: savedSourceSha256,
    wikidot_html_sha256: wikidotHtmlSha256,
  };
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
  const userEntries = new Map();
  const userNames = new Map();
  const userSlugs = new Map();
  const categoryEvidence = [];
  const userEvidence = [];
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
  for (const caseId of args.usersCases) {
    const comparison = comparisons.get(caseId);
    if (!comparison) throw new Error(`users case is absent from report: ${caseId}`);
    const provenance = await comparisonProvenance(comparison);
    for (const user of wikidotUsersFromComparison(comparison, provenance)) {
      const previousById = userEntries.get(user.user_id);
      const previousNameId = userNames.get(user.name);
      const previousSlugId = userSlugs.get(user.slug);
      if (
        (previousById && (
          previousById.name !== user.name ||
          previousById.slug !== user.slug
        )) ||
        (previousNameId != null && previousNameId !== user.user_id) ||
        (previousSlugId != null && previousSlugId !== user.user_id)
      ) {
        throw new Error(`Wikidot user identity differs between cases: ${user.user_id}`);
      }
      if (!previousById) userEntries.set(user.user_id, user);
      userNames.set(user.name, user.user_id);
      userSlugs.set(user.slug, user.user_id);
    }
    userEvidence.push({case_id: caseId, ...provenance});
  }
  const fixture = {
    schema: SCHEMA,
    captured_at: new Date().toISOString(),
    capture_source: {
      kind: "frozen-live-reference",
      report: args.report,
      report_sha256: sha256(reportBytes),
      category_evidence: categoryEvidence,
      user_evidence: userEvidence,
    },
    roots: [],
    pages: [],
    unresolved_pages: [],
    absent_pages: args.absentPages.map(splitPageKey),
    categories: [...categoryEntries.values()],
    wikidot_users: [...userEntries.values()],
  };
  await fs.writeFile(args.output, `${JSON.stringify(fixture, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: 0o600,
  });
  process.stdout.write(`${JSON.stringify({
    output: args.output,
    categories: fixture.categories.length,
    wikidot_users: fixture.wikidot_users.length,
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
