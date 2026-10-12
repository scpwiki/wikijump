#!/usr/bin/env node
import fs from "node:fs/promises";
import path from "node:path";
import {
  buildCorpusDiscussionManifest,
  canonicalDiscussionJson,
  canonicalCorpusPayloadJson,
  discussionSha256,
} from "../src/corpus-discussion-manifest.mjs";
if ((process.argv.length - 2) % 2 !== 0)
  throw new Error("Every argument needs an explicit value");
const args = Object.fromEntries(
  Array.from({ length: Math.floor((process.argv.length - 2) / 2) }, (_, i) => [
    process.argv[i * 2 + 2],
    process.argv[i * 2 + 3],
  ]),
);
const required = [
  "--corpus-root",
  "--branch",
  "--slug",
  "--source-site",
  "--page-dom",
  "--thread-document",
  "--forum-visible-document",
  "--forum-all-document",
  "--observation",
  "--source-reference",
  "--site-id",
  "--page-id",
  "--output",
];
if (
  required.some((key) => !args[key]) ||
  Object.keys(args).some((key) => !required.includes(key))
)
  throw new Error(`Required arguments: ${required.join(" ")}`);
const slug = args["--slug"],
  branch = args["--branch"];
for (const value of [slug, branch])
  if (value === "." || value === ".." || /[\\/\0]/u.test(value))
    throw new Error("Invalid corpus path component");
const root = path.resolve(args["--corpus-root"], branch),
  page = path.join(root, "pages", slug);
const readJson = async (p) => JSON.parse(await fs.readFile(p, "utf8"));
const entity = (
  await fs.readFile(path.join(page, "entity_id.txt"), "utf8")
).trim();
if (!/^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$/u.test(entity))
  throw new Error("Invalid corpus entity UUID");
const membership = await readJson(
  path.join(root, "posts", "pages", entity, "current.json"),
);
const corpusPosts = [];
const captures = [];
for (const id of membership.post_ids) {
  if (!Number.isSafeInteger(id) || id <= 0)
    throw new Error("Invalid source post ID");
  const currentPath = path.join(
    root,
    "posts",
    "by-id",
    String(id),
    "current.json",
  );
  const current = await readJson(currentPath);
  const hash = current.payload_sha256?.replace(/^sha256:/u, "");
  if (!/^[0-9a-f]{64}$/u.test(hash ?? ""))
    throw new Error(`Post ${id} lacks immutable capture identity`);
  const snapshot = await readJson(
    path.join(root, "posts", "by-id", String(id), "snapshots", `${hash}.json`),
  );
  if (discussionSha256(canonicalCorpusPayloadJson(snapshot.payload)) !== hash)
    throw new Error(`Post ${id} immutable payload hash mismatch`);
  for (const key of Object.keys(snapshot.payload))
    if (
      canonicalDiscussionJson(snapshot.payload[key]) !==
      canonicalDiscussionJson(current[key])
    )
      throw new Error(
        `Post ${id} current projection differs from its immutable capture`,
      );
  corpusPosts.push(current);
  captures.push({ post_id: id, payload_sha256: hash, run_id: current.run_id });
}
const manifest = buildCorpusDiscussionManifest({
  sourceSite: args["--source-site"],
  slug,
  pageDom: await fs.readFile(args["--page-dom"]),
  threadDocument: await fs.readFile(args["--thread-document"]),
  forumVisibleDocument: await fs.readFile(args["--forum-visible-document"]),
  forumAllDocument: await fs.readFile(args["--forum-all-document"]),
  observation: await readJson(args["--observation"]),
  pageMeta: await readJson(path.join(page, "meta.json")),
  membership,
  corpusPosts,
  sourceReference: args["--source-reference"],
  siteId: Number(args["--site-id"]),
  pageId: Number(args["--page-id"]),
  provenance: {
    corpus_branch: branch,
    corpus_entity_id: entity,
    membership_run_id: membership.run_id,
    post_captures: captures,
  },
});
await fs.writeFile(args["--output"], `${JSON.stringify(manifest, null, 2)}\n`, {
  mode: 0o600,
  flag: "wx",
});
console.log(
  JSON.stringify({
    output: path.resolve(args["--output"]),
    posts: manifest.params.source.posts.length,
    actors: manifest.params.source.actors.length,
    source_thread_id: manifest.params.source.thread.id,
    evidence_sha256: manifest.params.source.evidence_sha256,
  }),
);
