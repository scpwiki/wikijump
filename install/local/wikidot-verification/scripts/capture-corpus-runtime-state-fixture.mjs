#!/usr/bin/env node

import {createHash} from "node:crypto";
import fs from "node:fs/promises";
import {spawnSync} from "node:child_process";

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
  const args = {container: null, roots: [], output: null};
  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--database-container") {
      args.container = valueAfter(argv, index++, option);
    } else if (option === "--root") {
      args.roots.push(valueAfter(argv, index++, option));
    } else if (option === "--output") {
      args.output = valueAfter(argv, index++, option);
    } else {
      throw new Error(`unknown option: ${option}`);
    }
  }
  if (!args.container) throw new Error("--database-container is required");
  if (args.roots.length === 0) throw new Error("--root is required");
  if (!args.output) throw new Error("--output is required");
  return args;
}

function pageKey(site, slug) {
  const normalizedSite = site.toLowerCase();
  const normalizedSlug = slug.toLowerCase();
  if (!/^[a-z0-9-]+$/u.test(normalizedSite) || !/^[a-z0-9:_-]+$/u.test(normalizedSlug)) {
    throw new Error(`invalid page identity: ${site}:${slug}`);
  }
  return `${normalizedSite}:${normalizedSlug}`;
}

function splitPageKey(value) {
  const separator = value.indexOf(":");
  if (separator <= 0 || separator === value.length - 1) {
    throw new Error(`page root must be site:slug: ${value}`);
  }
  return [value.slice(0, separator), value.slice(separator + 1)];
}

function queryPage(container, site, slug) {
  const sql = `
SELECT json_build_object(
  'site', s.slug,
  'slug', p.slug,
  'title', pr.title,
  'wikitext', t.contents,
  'provenance', json_build_object(
    'source', 'standing-corpus',
    'site_id', s.site_id,
    'page_id', p.page_id,
    'revision_id', pr.revision_id,
    'revision_number', pr.revision_number,
    'revision_created_at', pr.created_at,
    'page_from_wikidot', p.from_wikidot,
    'revision_from_wikidot', pr.from_wikidot,
    'wikitext_hash', encode(pr.wikitext_hash, 'hex')
  )
)::text
FROM page p
JOIN site s ON s.site_id = p.site_id
JOIN page_revision pr ON pr.revision_id = p.latest_revision_id
JOIN text t ON t.hash = pr.wikitext_hash
WHERE s.slug = '${site}' AND p.slug = '${slug}' AND p.deleted_at IS NULL;
`.trim();
  const result = spawnSync("docker", [
    "exec",
    container,
    "sh",
    "-lc",
    'PGPASSWORD="$POSTGRES_PASSWORD" exec psql -h 127.0.0.1 -U "$POSTGRES_USER" -d "$POSTGRES_DB" --tuples-only --no-align --command "$1"',
    "sh",
    sql,
  ], {encoding: "utf8"});
  if (result.status !== 0) {
    throw new Error(`standing corpus query failed for ${site}:${slug}: ${result.stderr}`);
  }
  const output = result.stdout.trim();
  return output ? JSON.parse(output) : null;
}

function includedPages(site, wikitext) {
  const pages = [];
  for (const match of wikitext.matchAll(/\[\[\s*include[ \t]+(?<target>[^\s|\]]+)/giu)) {
    let target = match.groups.target;
    let targetSite = site;
    if (target.startsWith(":")) {
      const crossSite = /^:([^:]+):(.+)$/u.exec(target);
      if (!crossSite) continue;
      [, targetSite, target] = crossSite;
    }
    target = target.split(/[\/#]/u, 1)[0];
    if (!target || target.includes("{") || target.includes("}")) continue;
    try {
      pages.push(pageKey(targetSite, target));
    } catch {
      // Dynamic and malformed include targets remain outside the static closure.
    }
  }
  return pages;
}

export async function main(argv) {
  const args = parseArgs(argv);
  try {
    await fs.access(args.output);
    throw new Error(`output already exists: ${args.output}`);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }

  const roots = args.roots.map((root) => {
    const [site, slug] = splitPageKey(root);
    return pageKey(site, slug);
  });
  const queue = [...roots];
  const pages = new Map();
  const missing = [];
  while (queue.length > 0) {
    if (pages.size + missing.length >= 256) {
      throw new Error("runtime state fixture closure exceeded 256 pages");
    }
    const key = queue.shift();
    if (pages.has(key) || missing.includes(key)) continue;
    const [site, slug] = splitPageKey(key);
    const page = queryPage(args.container, site, slug);
    if (!page) {
      missing.push(key);
      continue;
    }
    page.source_sha256 = sha256(page.wikitext);
    pages.set(key, page);
    for (const included of includedPages(site, page.wikitext)) {
      if (!pages.has(included) && !missing.includes(included)) queue.push(included);
    }
  }

  const fixture = {
    schema: SCHEMA,
    captured_at: new Date().toISOString(),
    capture_source: {
      kind: "standing-corpus",
      database_container: args.container,
    },
    roots,
    pages: [...pages.values()],
    unresolved_pages: missing.sort(),
    absent_pages: [],
    categories: [],
  };
  await fs.writeFile(args.output, `${JSON.stringify(fixture, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: 0o600,
  });
  process.stdout.write(`${JSON.stringify({
    output: args.output,
    roots: roots.length,
    pages: pages.size,
    unresolved_pages: missing.length,
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
