#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import http from "node:http";
import https from "node:https";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const verifierRoot = path.resolve(__dirname, "..");
const corpusRoot = path.join(verifierRoot, "corpus");
const manifestPath = path.join(corpusRoot, "manifest.json");
const ADMIN_USER_ID = -1;
const IP_ADDRESS = "127.0.0.1";

function parseArgs(argv) {
  const args = {
    outputDir: path.resolve(process.cwd(), "wikidot-verification-run"),
  };

  for (let index = 2; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--output-dir") {
      args.outputDir = path.resolve(argv[++index]);
    } else if (arg === "--rpc-url") {
      args.rpcUrl = argv[++index];
    } else if (arg === "--site") {
      args.siteSlug = argv[++index];
    } else if (arg === "--force-files") {
      args.forceFiles = true;
    } else if (arg === "--presigned-connect-url") {
      args.presignedConnectUrl = argv[++index];
    } else if (arg === "--help") {
      printHelpAndExit();
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return args;
}

function printHelpAndExit() {
  console.log(`Usage: node seed-or-import.mjs [--output-dir DIR] [--rpc-url URL] [--site SLUG] [--force-files] [--presigned-connect-url URL]`);
  process.exit(0);
}

async function readJson(file) {
  return JSON.parse(await fs.readFile(file, "utf8"));
}

async function readCorpusText(relativePath) {
  return fs.readFile(path.join(corpusRoot, relativePath), "utf8");
}

async function readCorpusBytes(relativePath) {
  return fs.readFile(path.join(corpusRoot, relativePath));
}

function sha256Buffer(buffer) {
  return crypto.createHash("sha256").update(buffer).digest("hex");
}

function normalizeTags(tags = []) {
  return [...new Set(tags)].sort();
}

function sameTags(left = [], right = []) {
  const a = normalizeTags(left);
  const b = normalizeTags(right);
  return a.length === b.length && a.every((value, index) => value === b[index]);
}

class DeepwellClient {
  constructor(rpcUrl) {
    this.rpcUrl = rpcUrl;
    this.nextId = 1;
  }

  async call(method, params = {}, context = {}) {
    const headers = { "content-type": "application/json" };
    if (context.sessionToken) headers["X-Deepwell-Session-Token"] = context.sessionToken;
    if (context.siteId) headers["X-Deepwell-Site-Id"] = String(context.siteId);
    if (context.page) headers["X-Deepwell-Page"] = String(context.page);

    const response = await fetch(this.rpcUrl, {
      method: "POST",
      headers,
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: this.nextId++,
        method,
        params,
      }),
    });

    const bodyText = await response.text();
    let body;
    try {
      body = JSON.parse(bodyText);
    } catch (error) {
      throw new Error(`Invalid JSON-RPC response for ${method}: HTTP ${response.status} ${bodyText.slice(0, 300)}`);
    }

    if (!response.ok || body.error) {
      const message = body.error ? JSON.stringify(body.error) : bodyText;
      throw new Error(`JSON-RPC ${method} failed: HTTP ${response.status} ${message}`);
    }

    return body.result;
  }
}

async function maybeGetPage(client, siteId, slug) {
  return client.call("page_get", {
    site_id: siteId,
    page: slug,
    details: {
      wikitext: true,
      compiled: true,
    },
  });
}

async function getExistingPage(client, siteId, slug) {
  try {
    return await maybeGetPage(client, siteId, slug);
  } catch (error) {
    if (String(error.message).includes("PageMissing") || String(error.message).includes("not found")) {
      return null;
    }
    return null;
  }
}

async function createPage(client, siteId, page, source) {
  return client.call("page_create", {
    site_id: siteId,
    wikitext: source,
    title: page.title,
    alt_title: null,
    slug: page.slug,
    layout: page.layout || null,
    revision_comments: "local wikidot compatibility verifier create",
    user_id: ADMIN_USER_ID,
    ip_address: IP_ADDRESS,
  });
}

async function editPage(client, siteId, page, current, body, sessionToken, comment = "local wikidot compatibility verifier edit") {
  const output = await client.call("page_edit", {
    site_id: siteId,
    page: page.slug,
    last_revision_id: current.revision_id,
    revision_comments: comment,
    user_id: ADMIN_USER_ID,
    ip_address: IP_ADDRESS,
    ...body,
  }, {
    sessionToken,
    siteId,
    page: page.slug,
  });

  return output || await maybeGetPage(client, siteId, page.slug);
}

async function ensurePage(client, siteId, sessionToken, page, source) {
  const actions = [];
  let current = await getExistingPage(client, siteId, page.slug);

  if (!current) {
    const created = await createPage(client, siteId, page, source);
    actions.push(`created:${page.slug}`);
    if (created.parser_errors?.length) {
      throw new Error(`Parser errors while creating ${page.slug}: ${JSON.stringify(created.parser_errors)}`);
    }
    current = await maybeGetPage(client, siteId, page.slug);
  }

  const expectedTags = normalizeTags(page.tags || []);
  const needsEdit = current.wikitext !== source ||
    current.title !== page.title ||
    !sameTags(current.tags, expectedTags);

  if (needsEdit) {
    const edited = await editPage(client, siteId, page, current, {
      wikitext: source,
      title: page.title,
      tags: expectedTags,
    }, sessionToken);
    actions.push(`edited:${page.slug}`);
    if (edited.parser_errors?.length) {
      throw new Error(`Parser errors while editing ${page.slug}: ${JSON.stringify(edited.parser_errors)}`);
    }
    current = await maybeGetPage(client, siteId, page.slug);
  } else {
    actions.push(`unchanged:${page.slug}`);
  }

  if (page.parent) {
    await ensureParent(client, siteId, page.slug, page.parent);
    actions.push(`parent:${page.slug}->${page.parent}`);
  }

  return { page: current, actions };
}

async function ensureParent(client, siteId, child, parent) {
  const parents = await client.call("parent_get_all", {
    site_id: siteId,
    page: child,
  });

  if (parents.includes(parent)) return;

  await client.call("parent_update", {
    site_id: siteId,
    child,
    add: [parent],
    remove: null,
  });
}

async function uploadFile(client, siteId, page, file, forceFiles) {
  const buffer = await readCorpusBytes(file.path);
  const hash = sha256Buffer(buffer);
  const existingFiles = await client.call("page_get_files", {
    site_id: siteId,
    page_id: page.page_id,
    deleted: false,
  });

  const existing = existingFiles.find((candidate) => candidate.name === file.name);
  if (existing && !forceFiles) {
    return { name: file.name, action: "skipped-existing", sha256: hash, fileId: existing.file_id };
  }

  const upload = await client.call("blob_upload", {
    user_id: ADMIN_USER_ID,
    blob_size: buffer.length,
  });

  await putPresigned(upload.presign_url, buffer);

  const created = await client.call("file_create", {
    site_id: siteId,
    page_id: page.page_id,
    name: file.name,
    uploaded_blob_id: upload.pending_blob_id,
    revision_comments: "local wikidot compatibility verifier file",
    user_id: ADMIN_USER_ID,
    bypass_filter: true,
  });

  return { name: file.name, action: "uploaded", sha256: hash, fileId: created.file_id };
}

function presignedConnectUrlFor(presignUrl) {
  const explicit = process.env.WIKIDOT_VERIFY_PRESIGNED_CONNECT_URL;
  if (explicit) return new URL(explicit);

  const parsed = new URL(presignUrl);
  if (parsed.hostname === "files") {
    const port = process.env.WIKIDOT_VERIFY_MINIO_PORT || "9000";
    return new URL(`http://127.0.0.1:${port}`);
  }

  return null;
}

async function putPresigned(presignUrl, buffer) {
  const signed = new URL(presignUrl);
  const connectBase = presignedConnectUrlFor(presignUrl);
  const target = connectBase || signed;
  const requestUrl = new URL(`${target.protocol}//${target.host}${signed.pathname}${signed.search}`);
  const transport = requestUrl.protocol === "https:" ? https : http;

  await new Promise((resolve, reject) => {
    const request = transport.request(requestUrl, {
      method: "PUT",
      headers: {
        Host: signed.host,
        "Content-Length": String(buffer.length),
      },
    }, (response) => {
      const chunks = [];
      response.on("data", (chunk) => chunks.push(chunk));
      response.on("end", () => {
        if (response.statusCode && response.statusCode >= 200 && response.statusCode < 300) {
          resolve();
        } else {
          const body = Buffer.concat(chunks).toString("utf8");
          reject(new Error(`Presigned PUT failed: HTTP ${response.statusCode} connect=${requestUrl.origin} signed_host=${signed.host} ${body.slice(0, 500)}`));
        }
      });
    });

    request.on("error", (error) => {
      reject(new Error(`Presigned PUT connection failed: connect=${requestUrl.origin} signed_host=${signed.host} ${error.message}`));
    });
    request.end(buffer);
  });
}

async function runEditProof(client, siteId, sessionToken, editProof) {
  const initialSource = await readCorpusText(editProof.initialSource);
  const editedSource = await readCorpusText(editProof.editedSource);
  const initialPage = {
    slug: editProof.slug,
    title: editProof.title,
    layout: editProof.layout,
    tags: editProof.initialTags,
  };
  const editedPage = {
    slug: editProof.slug,
    title: editProof.title,
    layout: editProof.layout,
    tags: editProof.editedTags,
    parent: editProof.parent,
  };

  let current = await getExistingPage(client, siteId, editProof.slug);
  const actions = [];
  if (!current) {
    const created = await createPage(client, siteId, initialPage, initialSource);
    actions.push("created-initial");
    if (created.parser_errors?.length) {
      throw new Error(`Parser errors while creating ${editProof.slug}: ${JSON.stringify(created.parser_errors)}`);
    }
    current = await maybeGetPage(client, siteId, editProof.slug);
  } else {
    await editPage(client, siteId, initialPage, current, {
      wikitext: initialSource,
      title: editProof.title,
      tags: normalizeTags(editProof.initialTags),
    }, sessionToken, "local wikidot compatibility verifier reset for edit proof");
    actions.push("reset-initial");
    current = await maybeGetPage(client, siteId, editProof.slug);
  }

  const before = {
    revisionId: current.revision_id,
    revisionNumber: current.revision_number,
    tags: normalizeTags(current.tags),
    sourceSha256: sha256Buffer(Buffer.from(current.wikitext || "", "utf8")),
  };

  const edited = await editPage(client, siteId, editedPage, current, {
    wikitext: editedSource,
    title: editProof.title,
    tags: normalizeTags(editProof.editedTags),
  }, sessionToken, "local wikidot compatibility verifier final edit proof");
  if (edited.parser_errors?.length) {
    throw new Error(`Parser errors while editing ${editProof.slug}: ${JSON.stringify(edited.parser_errors)}`);
  }
  actions.push("saved-edited");

  await ensureParent(client, siteId, editProof.slug, editProof.parent);
  actions.push(`parent:${editProof.parent}`);

  const after = await maybeGetPage(client, siteId, editProof.slug);
  const parents = await client.call("parent_get_all", {
    site_id: siteId,
    page: editProof.slug,
  });

  const expectedTags = normalizeTags(editProof.editedTags);
  if (!sameTags(after.tags, expectedTags)) {
    throw new Error(`Edit proof tags mismatch for ${editProof.slug}: expected ${expectedTags.join(",")} got ${normalizeTags(after.tags).join(",")}`);
  }
  if (!parents.includes(editProof.parent)) {
    throw new Error(`Edit proof parent mismatch for ${editProof.slug}: missing ${editProof.parent}`);
  }
  if (!after.wikitext?.includes("Metadata After Edit Marker") || after.wikitext.includes("Metadata Before Edit Marker")) {
    throw new Error(`Edit proof source did not persist expected final marker for ${editProof.slug}`);
  }

  return {
    slug: editProof.slug,
    actions,
    before,
    after: {
      revisionId: after.revision_id,
      revisionNumber: after.revision_number,
      tags: normalizeTags(after.tags),
      sourceSha256: sha256Buffer(Buffer.from(after.wikitext || "", "utf8")),
      parents,
    },
  };
}

async function main() {
  const args = parseArgs(process.argv);
  await fs.mkdir(args.outputDir, { recursive: true });

  const manifest = await readJson(manifestPath);
  const rpcUrl = args.rpcUrl || process.env.WIKIDOT_VERIFY_RPC_URL || "http://127.0.0.1:2747/jsonrpc";
  if (args.presignedConnectUrl) {
    process.env.WIKIDOT_VERIFY_PRESIGNED_CONNECT_URL = args.presignedConnectUrl;
  }
  const siteSlug = args.siteSlug || process.env.WIKIDOT_VERIFY_SITE_SLUG || manifest.siteSlug;
  const adminEmail = process.env.WIKIDOT_VERIFY_ADMIN_EMAIL || manifest.admin.email;
  const adminPassword = process.env.WIKIDOT_VERIFY_ADMIN_PASS || manifest.admin.password;
  const client = new DeepwellClient(rpcUrl);

  await client.call("ping", {});
  const siteResult = await client.call("site_get", { site: siteSlug });
  const siteId = siteResult.site_id;
  const login = await client.call("login", {
    name_or_email: adminEmail,
    password: adminPassword,
    ip_address: IP_ADDRESS,
    user_agent: "wikidot-verifier/1.0",
  });
  const sessionToken = login.session_token;

  const results = [];
  const fileResults = [];
  for (const page of manifest.pages) {
    const source = await readCorpusText(page.source);
    const ensured = await ensurePage(client, siteId, sessionToken, page, source);
    results.push({
      slug: page.slug,
      pageId: ensured.page.page_id,
      revisionId: ensured.page.revision_id,
      revisionNumber: ensured.page.revision_number,
      tags: normalizeTags(ensured.page.tags),
      actions: ensured.actions,
      sourceSha256: sha256Buffer(Buffer.from(source, "utf8")),
    });

    for (const file of page.files || []) {
      const uploaded = await uploadFile(client, siteId, ensured.page, file, Boolean(args.forceFiles));
      fileResults.push({ page: page.slug, ...uploaded });
    }
  }

  const editProof = await runEditProof(client, siteId, sessionToken, manifest.editProof);
  results.push({
    slug: manifest.editProof.slug,
    pageId: (await maybeGetPage(client, siteId, manifest.editProof.slug)).page_id,
    revisionId: editProof.after.revisionId,
    revisionNumber: editProof.after.revisionNumber,
    tags: editProof.after.tags,
    actions: editProof.actions,
    sourceSha256: editProof.after.sourceSha256,
  });

  const summary = {
    generatedAt: new Date().toISOString(),
    rpcUrl,
    siteSlug,
    siteId,
    verifierRoot,
    pageCount: results.length,
    fileCount: fileResults.length,
    pages: results,
    files: fileResults,
    editProof,
  };

  const summaryPath = path.join(args.outputDir, "seed-summary.json");
  await fs.writeFile(summaryPath, JSON.stringify(summary, null, 2));
  await fs.writeFile(path.join(args.outputDir, "seed-results.tsv"), [
    "slug\tpage_id\trevision_id\trevision_number\ttags\tactions",
    ...results.map((result) => [
      result.slug,
      result.pageId,
      result.revisionId,
      result.revisionNumber,
      result.tags.join(","),
      result.actions.join(","),
    ].join("\t")),
    "",
  ].join("\n"));

  console.log(`Seeded ${results.length} pages and checked ${fileResults.length} files.`);
  console.log(`Summary: ${summaryPath}`);
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
