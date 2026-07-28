import fs from "node:fs/promises";
import path from "node:path";

import {
  canonicalDom,
  sha256,
  validateWikidotReference,
  visibleText,
} from "./syntax-differential.mjs";

export const LISTPAGES_PREVIEW_DIFFERENTIAL_SCHEMA =
  "wikijump_listpages_compat.preview_differential.v1";

export class DeepwellJsonRpcClient {
  constructor({ rpcUrl, fetchImpl = globalThis.fetch, timeoutMs = 30000 }) {
    this.rpcUrl = rpcUrl;
    this.fetchImpl = fetchImpl;
    this.timeoutMs = timeoutMs;
    this.nextId = 1;
  }

  async call(method, params) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.timeoutMs);
    let response;
    try {
      response = await this.fetchImpl(this.rpcUrl, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ jsonrpc: "2.0", id: this.nextId++, method, params }),
        signal: controller.signal,
      });
    } finally {
      clearTimeout(timer);
    }
    const text = await response.text();
    if (!response.ok) {
      throw new Error(`JSON-RPC ${method} failed with HTTP ${response.status}: ${text.slice(0, 300)}`);
    }
    const body = JSON.parse(text);
    if (body.error) {
      throw new Error(`JSON-RPC ${method} error: ${JSON.stringify(body.error)}`);
    }
    return body.result;
  }
}

async function readJsonl(filePath) {
  const text = await fs.readFile(filePath, "utf8");
  if (!text.trim()) return [];
  return text.trimEnd().split(/\r?\n/u).map((line) => JSON.parse(line));
}

function compareHtml(reference, localHtml) {
  const liveHtml = reference.raw_html;
  const liveDom = canonicalDom(liveHtml);
  const localDom = canonicalDom(localHtml);
  const domMatches = JSON.stringify(liveDom) === JSON.stringify(localDom);
  const liveVisibleText = visibleText(liveHtml);
  const localVisibleText = visibleText(localHtml);
  const textMatches = liveVisibleText === localVisibleText;
  return {
    status: domMatches && textMatches ? "match" : "mismatch",
    checks: {
      dom_tree: {
        status: domMatches ? "match" : "mismatch",
        ...(domMatches ? {} : { live: liveDom, local: localDom }),
      },
      visible_text: {
        status: textMatches ? "match" : "mismatch",
        live: liveVisibleText,
        local: localVisibleText,
      },
    },
    identities: {
      source_sha256: reference.source_sha256,
      live_html_sha256: reference.raw_html_sha256,
      local_html_sha256: sha256(localHtml),
    },
  };
}

export async function runListPagesPreviewDifferential({
  referencesPath,
  runtimeIdentityPath = null,
  rpcUrl,
  siteSlug,
  rpcClient = new DeepwellJsonRpcClient({ rpcUrl }),
}) {
  const references = (await readJsonl(referencesPath)).map(validateWikidotReference);
  const runtimeIdentity = runtimeIdentityPath
    ? JSON.parse(await fs.readFile(runtimeIdentityPath, "utf8"))
    : null;
  const site = await rpcClient.call("site_get", { site: siteSlug });
  if (!Number.isSafeInteger(site?.site_id)) {
    throw new Error(`local site lookup did not return a site_id for ${siteSlug}`);
  }

  const cases = [];
  for (const reference of references) {
    const syntaxCase = reference.syntax_case;
    let result;
    try {
      const preview = await rpcClient.call("wikidot_page_preview", {
        site_id: site.site_id,
        title: syntaxCase.title,
        wikitext: syntaxCase.source,
      });
      if (!preview || typeof preview.body !== "string") {
        throw new Error("local preview returned no body");
      }
      result = {
        schema: `${LISTPAGES_PREVIEW_DIFFERENTIAL_SCHEMA}.case`,
        case_id: syntaxCase.case_id,
        status: null,
        live: {
          html_sha256: reference.raw_html_sha256,
          visible_text: visibleText(reference.raw_html),
        },
        local: {
          html_sha256: sha256(preview.body),
          visible_text: visibleText(preview.body),
          styles: Array.isArray(preview.styles) ? preview.styles : [],
        },
        comparison: compareHtml(reference, preview.body),
      };
      result.status = result.comparison.status;
    } catch (error) {
      result = {
        schema: `${LISTPAGES_PREVIEW_DIFFERENTIAL_SCHEMA}.case`,
        case_id: syntaxCase.case_id,
        status: "local-error",
        error: error instanceof Error ? error.message : String(error),
        live: {
          html_sha256: reference.raw_html_sha256,
          visible_text: visibleText(reference.raw_html),
        },
      };
    }
    cases.push(result);
  }

  const counts = {};
  for (const row of cases) counts[row.status] = (counts[row.status] ?? 0) + 1;
  return {
    schema: LISTPAGES_PREVIEW_DIFFERENTIAL_SCHEMA,
    generated_at: new Date().toISOString(),
    inputs: {
      references_path: referencesPath,
      references_sha256: sha256(await fs.readFile(referencesPath, "utf8")),
      runtime_identity_path: runtimeIdentityPath,
      rpc_url: rpcUrl,
      site_slug: siteSlug,
      local_site: { slug: site.slug, site_id: site.site_id },
    },
    runtime_identity: runtimeIdentity,
    cases,
    summary: {
      total: cases.length,
      counts,
      exit_code: (counts.mismatch ?? 0) > 0 || (counts["local-error"] ?? 0) > 0 ? 1 : 0,
    },
  };
}

export async function writePreviewDifferential(verdict, outputPath) {
  await fs.mkdir(path.dirname(outputPath), { recursive: true });
  await fs.writeFile(outputPath, `${JSON.stringify(verdict, null, 2)}\n`, { mode: 0o600 });
}
