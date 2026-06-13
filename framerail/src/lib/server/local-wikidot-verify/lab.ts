import { client } from "$lib/server/deepwell"
import { pageFileCreate, pageFileList } from "$lib/server/deepwell/pageFile"
import { Layout } from "$lib/types"

import type { Nullable, SiteModel } from "$lib/types"

const SITE_SLUG = process.env.WIKIDOT_VERIFY_SITE_SLUG || "scp-wiki"
const ADMIN_EMAIL = process.env.WIKIDOT_VERIFY_ADMIN_EMAIL || "admin@wikijump"
const ADMIN_PASSWORD = process.env.WIKIDOT_VERIFY_ADMIN_PASS || "wikijumpadmin1"
const ADMIN_USER_ID = -1
const IP_ADDRESS = "127.0.0.1"
const USER_AGENT = "wikijump-local-authoring-lab/0.1"

interface SiteGetOutput extends SiteModel {
  aliases?: unknown[]
  domains?: unknown[]
}

export interface LabPage {
  page_id: number
  page_created_at: string
  page_updated_at: Nullable<string>
  page_deleted_at: Nullable<string>
  page_revision_count: number
  site_id: number
  page_category_id: number
  page_category_slug: string
  discussion_thread_id: Nullable<number>
  revision_id: number
  revision_type: string
  revision_created_at: string
  revision_number: number
  revision_user_id: number
  wikitext: Nullable<string>
  compiled_body_html: Nullable<string>
  compiled_at: string
  compiled_generator: string
  revision_comments: string
  hidden_fields: string[]
  title: string
  alt_title: Nullable<string>
  slug: string
  tags: string[]
  rating: number
  layout: Layout
}

export interface LabSession {
  site: SiteGetOutput
  sessionToken: string
}

interface LoginOutput {
  session_token: string
}

export interface SavePageInput {
  slug: string
  title: string
  wikitext: string
  tags: string[]
  parent?: string
}

export interface PreviewPageOutput {
  slug: string
  html: string
  parserErrors: unknown[]
}

export interface SavePageOutput {
  page: LabPage
  parents: string[]
  created: boolean
  parserErrors: unknown[]
}

export interface LabBundlePage {
  slug: string
  title: string
  tags: string[]
  wikitext: string
  revisionNumber: number
}

export interface LabBundle {
  manifest: {
    exportedAt: string
    sourceSite: string
    pageCount: number
    format: "wikijump-local-authoring-lab-v1"
  }
  pages: LabBundlePage[]
  assets: unknown[]
  metadata: Record<string, unknown>
}

export interface DependencyScenarioOutput {
  fragment: SavePageOutput
  component: SavePageOutput
  host: SavePageOutput
  hostHtml: string
}

export interface ListPagesScenarioOutput {
  targets: SavePageOutput[]
  index: SavePageOutput
  indexHtml: string
}

export interface ThemeNavCssScenarioOutput {
  topNav: SavePageOutput
  sideNav: SavePageOutput
  proof: SavePageOutput
  proofHtml: string
}

export interface ScenarioPageSummary {
  slug: string
  title: string
  exists: boolean
  revisionNumber?: number
  tags: string[]
}

export interface ProofSummaryCheck {
  name: string
  pass: boolean
  detail: string
}

export interface ProofSummaryOutput {
  generatedAt: string
  selectedSlug: string
  checks: ProofSummaryCheck[]
  passed: number
  failed: number
}

export const PROOF_SCENARIO_SLUGS = [
  "ui-authoring-basic",
  "ui-authoring-include-host",
  "ui-authoring-listpages-index",
  "ui-authoring-theme-nav-css",
  "ui-authoring-assets-browser-5",
  "ui-authoring-browser-proof-5",
  "nav:top",
  "nav:side"
]

export function normalizeSlug(value: FormDataEntryValue | null): string {
  return String(value ?? "")
    .trim()
    .toLowerCase()
    .replace(/\s+/g, "-")
}

export function normalizeTags(value: FormDataEntryValue | null): string[] {
  return [
    ...new Set(
      String(value ?? "")
        .split(/[,\s]+/)
        .map((tag) => tag.trim())
        .filter(Boolean)
        .sort()
    )
  ]
}

export function localPreviewHtml(wikitext: string): string {
  const escaped = escapeHtml(wikitext)
  const paragraphs = escaped
    .split(/\n{2,}/)
    .map((paragraph) => `<p>${paragraph.replace(/\n/g, "<br>")}</p>`)
    .join("")

  return `<div class="local-preview">${paragraphs || "<p></p>"}</div>`
}

export async function renderPreviewPage(input: SavePageInput): Promise<PreviewPageOutput> {
  const previewSlug = `ui-authoring-preview-${input.slug}`
  const preview = await savePage({
    slug: previewSlug,
    title: `${input.title || input.slug} Preview`,
    wikitext: input.wikitext,
    tags: ["preview", "ui-authoring", "verification"]
  })
  const rendered = await rerenderPage(previewSlug)

  return {
    slug: previewSlug,
    html: rendered.compiled_body_html ?? "",
    parserErrors: preview.parserErrors
  }
}

export function previewWarnings(wikitext: string): string[] {
  const warnings: string[] = []
  if (/\[\[include\s+/i.test(wikitext)) warnings.push("include syntax present")
  if (/\[\[module\s+ListPages/i.test(wikitext)) warnings.push("ListPages syntax present")
  if (/\[\[image\s+/i.test(wikitext)) warnings.push("image syntax present")
  return warnings
}

export async function openLabSession(): Promise<LabSession> {
  const site = await getSite()
  const login = (await client.request("login", {
    name_or_email: ADMIN_EMAIL,
    password: ADMIN_PASSWORD,
    ip_address: IP_ADDRESS,
    user_agent: USER_AGENT
  })) as LoginOutput

  return { site, sessionToken: login.session_token }
}

export async function getSite(): Promise<SiteGetOutput> {
  const site = (await client.request("site_get", {
    site: SITE_SLUG
  })) as Nullable<SiteGetOutput>
  if (!site) throw new Error(`Local verification site not found: ${SITE_SLUG}`)
  return site
}

export async function getPage(siteId: number, slug: string): Promise<Nullable<LabPage>> {
  return (await client.request("page_get", {
    site_id: siteId,
    page: slug,
    details: {
      wikitext: true,
      compiled: true
    }
  })) as Nullable<LabPage>
}

export async function getScenarioPages(
  siteId: number,
  selectedSlug: string
): Promise<ScenarioPageSummary[]> {
  const slugs = [...new Set([selectedSlug, ...PROOF_SCENARIO_SLUGS])]
  const summaries = []
  for (const slug of slugs) {
    const page = await getPage(siteId, slug)
    summaries.push({
      slug,
      title: page?.title ?? slug,
      exists: Boolean(page),
      revisionNumber: page?.revision_number,
      tags: page?.tags ?? []
    })
  }
  return summaries
}

export async function savePage(input: SavePageInput): Promise<SavePageOutput> {
  const session = await openLabSession()
  const siteId = session.site.site_id
  const existing = await getPage(siteId, input.slug)
  let created = false
  let parserErrors: unknown[] = []

  if (!existing) {
    const output = (await client.request("page_create", {
      site_id: siteId,
      wikitext: input.wikitext,
      title: input.title,
      alt_title: null,
      slug: input.slug,
      layout: Layout.WIKIDOT,
      revision_comments: "local authoring lab create",
      user_id: ADMIN_USER_ID,
      ip_address: IP_ADDRESS
    })) as { parser_errors?: unknown[] }
    created = true
    parserErrors = output.parser_errors ?? []
  }

  const current = await requirePage(siteId, input.slug)
  const needsEdit =
    current.wikitext !== input.wikitext ||
    current.title !== input.title ||
    !sameTags(current.tags, input.tags)

  if (needsEdit) {
    const output = (await client.request(
      "page_edit",
      {
        site_id: siteId,
        page: input.slug,
        last_revision_id: current.revision_id,
        revision_comments: created
          ? "local authoring lab apply metadata"
          : "local authoring lab edit",
        user_id: ADMIN_USER_ID,
        ip_address: IP_ADDRESS,
        wikitext: input.wikitext,
        title: input.title,
        tags: input.tags
      },
      {
        sessionToken: session.sessionToken,
        siteId,
        page: input.slug
      }
    )) as Nullable<{ parser_errors?: unknown[] }>
    parserErrors = [...parserErrors, ...(output?.parser_errors ?? [])]
  }

  if (input.parent) {
    await client.request("parent_update", {
      site_id: siteId,
      child: input.slug,
      add: [input.parent],
      remove: null
    })
  }

  const page = await requirePage(siteId, input.slug)
  const parents = await getParents(siteId, input.slug)
  return { page, parents, created, parserErrors }
}

export async function updateTags(slug: string, tags: string[]): Promise<LabPage> {
  const session = await openLabSession()
  const siteId = session.site.site_id
  const current = await requirePage(siteId, slug)
  await client.request(
    "page_edit",
    {
      site_id: siteId,
      page: slug,
      last_revision_id: current.revision_id,
      revision_comments: "local authoring lab tag edit",
      user_id: ADMIN_USER_ID,
      ip_address: IP_ADDRESS,
      tags
    },
    {
      sessionToken: session.sessionToken,
      siteId,
      page: slug
    }
  )
  return requirePage(siteId, slug)
}

export async function createDependencyScenario(): Promise<DependencyScenarioOutput> {
  const fragment = await savePage({
    slug: "ui-authoring-fragment-alpha",
    title: "UI Authoring Fragment Alpha",
    wikitext: "Fragment alpha body created through the dependency panel.",
    tags: ["ui-authoring", "dependency", "verification"]
  })
  const component = await savePage({
    slug: "component:ui-authoring-card",
    title: "UI Authoring Card Component",
    wikitext: "Component card body created through the dependency panel.",
    tags: ["ui-authoring", "component", "verification"]
  })
  const host = await savePage({
    slug: "ui-authoring-include-host",
    title: "UI Authoring Include Host",
    wikitext:
      "+ UI Authoring Include Host\n\n[[include ui-authoring-fragment-alpha]]\n\n[[include component:ui-authoring-card]]\n",
    tags: ["ui-authoring", "include-host", "verification"]
  })
  const refreshedHost = await rerenderPage(host.page.slug)

  return {
    fragment,
    component,
    host,
    hostHtml: refreshedHost.compiled_body_html ?? ""
  }
}

export async function createListPagesScenario(): Promise<ListPagesScenarioOutput> {
  const targets = []
  for (const target of [
    ["ui-authoring-list-target-alpha", "UI Authoring List Target Alpha"],
    ["ui-authoring-list-target-beta", "UI Authoring List Target Beta"],
    ["ui-authoring-list-target-gamma", "UI Authoring List Target Gamma"]
  ] as const) {
    targets.push(
      await savePage({
        slug: target[0],
        title: target[1],
        wikitext: `${target[1]} body marker.`,
        tags: ["ui-authoring", "ui-authoring-listpages", "verification"]
      })
    )
  }

  const index = await savePage({
    slug: "ui-authoring-listpages-index",
    title: "UI Authoring ListPages Index",
    wikitext:
      '+ UI Authoring ListPages Index\n\n[[module ListPages tags="+ui-authoring-listpages" limit="10" order="name"]]\n* %%title_linked%%\n[[/module]]\n',
    tags: ["ui-authoring", "listpages-index", "verification"]
  })
  for (const target of targets) {
    await savePage({
      slug: target.page.slug,
      title: target.page.title,
      wikitext: target.page.wikitext ?? "",
      tags: target.page.tags,
      parent: index.page.slug
    })
  }
  const refreshedIndex = await rerenderPage(index.page.slug)

  return {
    targets,
    index,
    indexHtml: refreshedIndex.compiled_body_html ?? ""
  }
}

export async function removeListPagesGamma(): Promise<LabPage> {
  const page = await updateTags("ui-authoring-list-target-gamma", [
    "ui-authoring",
    "verification"
  ])
  await rerenderPage("ui-authoring-listpages-index")
  return page
}

export async function createThemeNavCssScenario(): Promise<ThemeNavCssScenarioOutput> {
  const topNav = await savePage({
    slug: "nav:top",
    title: "Top Navigation",
    wikitext:
      "* [[[start | Homepage]]]\n* [[[ui-authoring-theme-nav-css | UI Authoring Theme CSS]]]\n* [[[ui-authoring-include-host | Include Host]]]\n",
    tags: ["navigation", "ui-authoring", "verification"]
  })
  const sideNav = await savePage({
    slug: "nav:side",
    title: "Side Navigation",
    wikitext:
      "= Local Verification\n* [[[ui-authoring-theme-nav-css | Theme Nav CSS]]]\n* [[[ui-authoring-listpages-index | ListPages Index]]]\n\n[[div class=\"ui-authoring-side-nav-marker\"]]\nUI Authoring Side Nav Marker\n[[/div]]\n\n[[div style=\"text-align: center;\"]]\n[[size 80%]][[[nav:side | edit this panel]]][[/size]]\n[[/div]]\n",
    tags: ["navigation", "ui-authoring", "verification"]
  })
  const proof = await savePage({
    slug: "ui-authoring-theme-nav-css",
    title: "UI Authoring Theme Navigation CSS",
    wikitext:
      "[[module CSS]]\n.ui-authoring-theme-css-marker { color: rgb(12, 98, 140); border: 2px solid rgb(12, 98, 140); padding: 0.5rem; }\n[[/module]]\n\n+ UI Authoring Theme Navigation CSS\n\n[[div class=\"ui-authoring-theme-css-marker\"]]\nUI Authoring Theme CSS Applied Marker.\n[[/div]]\n\nUI Authoring Nav Target marker.\n",
    tags: ["theme-nav-css", "ui-authoring", "verification"]
  })
  const refreshedProof = await rerenderPage(proof.page.slug)

  return {
    topNav,
    sideNav,
    proof,
    proofHtml: refreshedProof.compiled_body_html ?? ""
  }
}

export async function runProofSummary(selectedSlug: string): Promise<ProofSummaryOutput> {
  const site = await getSite()
  const checks: ProofSummaryCheck[] = []
  const selected = selectedSlug ? await getPage(site.site_id, selectedSlug) : null
  const selectedHistory = selected
    ? ((await getHistory(site.site_id, selected.page_id)) as unknown[])
    : []
  const includeHost = await getPage(site.site_id, "ui-authoring-include-host")
  const listPages = await getPage(site.site_id, "ui-authoring-listpages-index")
  const themeProof = await getPage(site.site_id, "ui-authoring-theme-nav-css")
  const topNav = await getPage(site.site_id, "nav:top")
  const sideNav = await getPage(site.site_id, "nav:side")
  const assetPage =
    (await getPage(site.site_id, "ui-authoring-assets-browser-5")) ??
    (await getPage(site.site_id, "ui-authoring-assets-browser-3")) ??
    (await getPage(site.site_id, "ui-authoring-assets"))

  checks.push({
    name: "selected-page-saved",
    pass: Boolean(selected?.compiled_body_html),
    detail: selected
      ? `${selected.slug} revision ${selected.revision_number}`
      : `${selectedSlug || "selected page"} missing`
  })
  checks.push({
    name: "selected-page-history",
    pass: selectedHistory.length >= 2,
    detail: `${selectedHistory.length} revisions available`
  })
  checks.push({
    name: "include-dependencies",
    pass:
      includeHost?.compiled_body_html?.includes(
        "Fragment alpha body created through the dependency panel."
      ) === true &&
      includeHost?.compiled_body_html?.includes(
        "Component card body created through the dependency panel."
      ) === true &&
      includeHost?.compiled_body_html?.includes("No such page") !== true,
    detail: includeHost ? includeHost.slug : "include host missing"
  })
  checks.push({
    name: "listpages-tag-update",
    pass:
      listPages?.compiled_body_html?.includes("ui-authoring-list-target-alpha") ===
        true &&
      listPages?.compiled_body_html?.includes("ui-authoring-list-target-beta") ===
        true &&
      listPages?.compiled_body_html?.includes("ui-authoring-list-target-gamma") !==
        true,
    detail: listPages?.compiled_body_html?.replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim() ?? "ListPages index missing"
  })
  checks.push({
    name: "theme-navigation-css",
    pass:
      themeProof?.compiled_body_html?.includes(
        "UI Authoring Theme CSS Applied Marker"
      ) === true &&
      topNav?.compiled_body_html?.includes("UI Authoring Theme CSS") === true &&
      sideNav?.compiled_body_html?.includes("UI Authoring Side Nav Marker") === true,
    detail: "checks ui-authoring-theme-nav-css, nav:top, and nav:side"
  })
  checks.push({
    name: "asset-page-image-source",
    pass:
      assetPage?.compiled_body_html?.includes("local--files") === true &&
      assetPage?.compiled_body_html?.includes("fog-green.svg") === true,
    detail: assetPage ? assetPage.slug : "asset page missing"
  })

  const passed = checks.filter((check) => check.pass).length
  return {
    generatedAt: new Date().toISOString(),
    selectedSlug,
    checks,
    passed,
    failed: checks.length - passed
  }
}

export async function rerenderPage(slug: string): Promise<LabPage> {
  const session = await openLabSession()
  const page = await requirePage(session.site.site_id, slug)
  await client.request("page_rerender", {
    site_id: session.site.site_id,
    category_id: page.page_category_id,
    page_id: page.page_id
  })
  return requirePage(session.site.site_id, slug)
}

export async function getParents(siteId: number, slug: string): Promise<string[]> {
  return (await client.request("parent_get_all", {
    site_id: siteId,
    page: slug
  })) as string[]
}

export async function getHistory(siteId: number, pageId: number) {
  return client.request("page_revision_range", {
    site_id: siteId,
    page_id: pageId,
    revision_number: 999999,
    revision_direction: "before",
    limit: 20,
    details: {
      wikitext: true,
      compiled: false
    }
  })
}

export async function uploadPageFile(slug: string, file: File, name?: string) {
  const session = await openLabSession()
  const page = await requirePage(session.site.site_id, slug)
  await pageFileCreate(
    session.site.site_id,
    page.page_id,
    ADMIN_USER_ID,
    name || file.name,
    file,
    "local authoring lab file upload",
    true
  )
  return pageFileList(session.site.site_id, page.page_id, false)
}

export async function exportBundle(slugs: string[]): Promise<LabBundle> {
  const session = await openLabSession()
  const pages: LabBundlePage[] = []

  for (const slug of slugs) {
    const page = await requirePage(session.site.site_id, slug)
    pages.push({
      slug: page.slug,
      title: page.title,
      tags: page.tags,
      wikitext: page.wikitext ?? "",
      revisionNumber: page.revision_number
    })
  }

  return {
    manifest: {
      exportedAt: new Date().toISOString(),
      sourceSite: session.site.slug,
      pageCount: pages.length,
      format: "wikijump-local-authoring-lab-v1"
    },
    pages,
    assets: [],
    metadata: {
      route: "/__local-wikidot-verify"
    }
  }
}

export async function importBundle(bundle: LabBundle, prefix: string) {
  const outputs: SavePageOutput[] = []
  for (const page of bundle.pages) {
    outputs.push(
      await savePage({
        slug: `${prefix}${page.slug}`,
        title: `${page.title} Import`,
        wikitext: page.wikitext,
        tags: page.tags
      })
    )
  }
  return outputs
}

async function requirePage(siteId: number, slug: string): Promise<LabPage> {
  const page = await getPage(siteId, slug)
  if (!page) throw new Error(`Page not found after write: ${slug}`)
  return page
}

function sameTags(left: string[] = [], right: string[] = []): boolean {
  const a = [...left].sort()
  const b = [...right].sort()
  return a.length === b.length && a.every((value, index) => value === b[index])
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;")
}
