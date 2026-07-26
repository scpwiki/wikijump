import crypto from "node:crypto";

import {DeepwellJsonRpcClient} from "./theme-localization-deepwell-adapter.mjs";
import {
  validateRuntimeIdentity,
  validateSavedPageReference,
} from "./saved-page-runtime-differential.mjs";

const SITE_SLUG = "scp-wiki";
const IP_ADDRESS = "127.0.0.1";
export const RERENDER_RECEIPT_SCHEMA =
  "wikijump_syntax_differential.saved_page_runtime_rerender_receipt.v1";

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function pageSnapshot(page, reference, siteId) {
  if (
    page === null ||
    !Number.isSafeInteger(page.page_id) ||
    !Number.isSafeInteger(page.page_category_id) ||
    !Number.isSafeInteger(page.revision_id) ||
    page.site_id !== siteId ||
    page.slug !== reference.page.slug ||
    typeof page.wikitext !== "string" ||
    typeof page.compiled_generator !== "string" ||
    typeof page.compiled_at !== "string"
  ) {
    throw new Error(`local page ${reference.page.slug} returned an incomplete identity`);
  }
  const sourceSha256 = sha256(page.wikitext);
  if (sourceSha256 !== reference.page.source_sha256) {
    throw new Error(`local page ${reference.page.slug} source differs from the frozen Wikidot source`);
  }
  return {
    page_id: page.page_id,
    page_category_id: page.page_category_id,
    revision_id: page.revision_id,
    source_sha256: sourceSha256,
    compiled_at: page.compiled_at,
    compiled_generator: page.compiled_generator,
  };
}

function assertStablePage(before, after, slug) {
  for (const field of ["page_id", "page_category_id", "revision_id", "source_sha256"]) {
    if (before[field] !== after[field]) {
      throw new Error(`local page ${slug} changed ${field} during rerender`);
    }
  }
}

function assertCurrentGenerator(generator, ftmlSha, slug) {
  const expectedRevision = ftmlSha.slice(0, 8);
  if (typeof generator !== "string" || !generator.includes(`[${expectedRevision}]`)) {
    throw new Error(`local page ${slug} was not compiled by FTML ${expectedRevision}`);
  }
}

export function validateSavedPageRerenderReceipt(receipt, references, runtimeIdentity) {
  validateRuntimeIdentity(runtimeIdentity);
  if (receipt?.schema !== RERENDER_RECEIPT_SCHEMA || receipt.status !== "pass") {
    throw new Error("saved-page rerender receipt is not a passing supported receipt");
  }
  for (const field of [
    "schema",
    "wikijump_sha",
    "ftml_sha",
    "dependency_lock_sha256",
    "executable_sha256",
    "runtime_config_sha256",
  ]) {
    if (receipt.runtime_identity?.[field] !== runtimeIdentity[field]) {
      throw new Error("saved-page rerender receipt runtime identity differs from the verdict identity");
    }
  }
  if (receipt.local_site?.slug !== SITE_SLUG || !Number.isSafeInteger(receipt.local_site?.site_id)) {
    throw new Error("saved-page rerender receipt local site is invalid");
  }
  const pages = new Map();
  for (const page of receipt.pages ?? []) {
    if (typeof page?.slug !== "string" || pages.has(page.slug)) {
      throw new Error("saved-page rerender receipt page slugs are invalid or duplicated");
    }
    pages.set(page.slug, page);
  }
  if (pages.size !== references.length) {
    throw new Error("saved-page rerender receipt does not cover the reference set");
  }
  for (const unvalidatedReference of references) {
    const reference = validateSavedPageReference(unvalidatedReference);
    const page = pages.get(reference.page.slug);
    if (
      page?.case_id !== reference.case.case_id ||
      page.wikidot_source_sha256 !== reference.page.source_sha256 ||
      page.before?.source_sha256 !== reference.page.source_sha256 ||
      page.after?.source_sha256 !== reference.page.source_sha256
    ) {
      throw new Error(`saved-page rerender receipt source identity differs for ${reference.page.slug}`);
    }
    assertStablePage(page.before, page.after, reference.page.slug);
    assertCurrentGenerator(page.after.compiled_generator, runtimeIdentity.ftml_sha, reference.page.slug);
  }
  return receipt;
}

export async function rerenderSavedPageRuntime({
  references,
  runtimeIdentity,
  administratorEmail,
  administratorPassword,
  rpcClient = new DeepwellJsonRpcClient({rpcUrl: "http://127.0.0.1:12747/jsonrpc"}),
  now = () => new Date().toISOString(),
}) {
  validateRuntimeIdentity(runtimeIdentity);
  if (!administratorEmail || !administratorPassword) {
    throw new Error("standing rerender administrator credentials are required");
  }
  const validatedReferences = references.map(validateSavedPageReference);
  if (
    validatedReferences.length === 0 ||
    validatedReferences.some((reference) => reference.site.unix_name !== SITE_SLUG)
  ) {
    throw new Error("standing rerender accepts only frozen scp-wiki references");
  }
  const slugs = validatedReferences.map((reference) => reference.page.slug);
  if (new Set(slugs).size !== slugs.length) {
    throw new Error("standing rerender references must use unique page slugs");
  }

  const startedAt = now();
  await rpcClient.call("ping", {});
  const site = await rpcClient.call("site_get", {site: SITE_SLUG});
  if (!Number.isSafeInteger(site?.site_id)) {
    throw new Error("standing scp-wiki site lookup did not return an integer site id");
  }
  const login = await rpcClient.call("login", {
    name_or_email: administratorEmail,
    password: administratorPassword,
    ip_address: IP_ADDRESS,
    user_agent: "saved-page-runtime-rerender/0.1",
  });
  if (login?.needs_mfa !== false || typeof login?.session_token !== "string" || !login.session_token) {
    throw new Error("standing rerender login did not return a complete session");
  }
  const session = await rpcClient.call("session_get", [login.session_token]);
  if (!Number.isSafeInteger(session?.user_id)) {
    throw new Error("standing rerender session did not return an integer user id");
  }
  const baseContext = {
    sessionToken: login.session_token,
    siteId: site.site_id,
  };
  const pages = [];
  for (const reference of validatedReferences) {
    const params = {
      site_id: site.site_id,
      page: reference.page.slug,
      details: {wikitext: true, compiled: false},
    };
    const context = {...baseContext, page: reference.page.slug};
    const before = pageSnapshot(
      await rpcClient.call("page_get", params, context),
      reference,
      site.site_id,
    );
    await rpcClient.call(
      "page_rerender",
      {
        site_id: site.site_id,
        category_id: before.page_category_id,
        page_id: before.page_id,
      },
      context,
    );
    const after = pageSnapshot(
      await rpcClient.call("page_get", params, context),
      reference,
      site.site_id,
    );
    assertStablePage(before, after, reference.page.slug);
    assertCurrentGenerator(after.compiled_generator, runtimeIdentity.ftml_sha, reference.page.slug);
    pages.push({
      case_id: reference.case.case_id,
      slug: reference.page.slug,
      wikidot_source_sha256: reference.page.source_sha256,
      before,
      after,
    });
  }
  return {
    schema: RERENDER_RECEIPT_SCHEMA,
    status: "pass",
    started_at: startedAt,
    completed_at: now(),
    runtime_identity: runtimeIdentity,
    local_site: {slug: SITE_SLUG, site_id: site.site_id},
    actor_user_id: session.user_id,
    pages,
    resource_disposition: {
      standing_containers: "retained",
      standing_volumes: "untouched",
      page_sources: "unchanged",
      page_revisions: "unchanged",
      compiled_artifacts: "updated in place",
      worktrees: "none created",
      target_directories: "none created",
    },
  };
}
