import defaults from "$lib/defaults"

import { client } from "$lib/server/deepwell"
import { pageFileRevisionHistory } from "$lib/server/deepwell/page-file"
import { Layout, PageLockType } from "$lib/types"
import { mergeWikidotHistoryTimeline } from "$lib/server/wikidot-history-timeline.js"

import type {
  FileRevisionType,
  Nullable,
  Optional,
  PageRevisionType,
  PageVoteModel,
  ParseError,
  UserInfo
} from "$lib/types"
import type { RequestContext } from "../request-context"

const MAX_PAGE_HISTORY_REVISIONS = 1_000

/* ----- Page Delete ----- */
interface PageDelete {
  page_id: number
  revision_id: number
  revision_number: number
}
export interface PageDeleteInput {
  siteId: number
  pageId: Optional<number>
  userId: number
  userIpAddr: string
  slug: string
  lastRevisionId: number
  revisionComments: Optional<string>
}
export async function pageDelete(
  input: PageDeleteInput,
  requestContext: RequestContext
): Promise<PageDelete> {
  const { siteId, pageId, userId, userIpAddr, slug, lastRevisionId, revisionComments } =
    input
  return client.request(
    "page_delete",
    {
      site_id: siteId,
      page: pageId ?? slug,
      user_id: userId,
      ip_address: userIpAddr,
      last_revision_id: lastRevisionId,
      revision_comments: revisionComments
    },
    requestContext
  )
}

/* ----- Page Edit ----- */
export interface CreatePageRevisionOutput {
  page_id?: number
  revision_id: number
  revision_number: number
  parser_errors: Nullable<ParseError[]>
  slug?: string
}
export interface PageEditInput {
  siteId: number
  pageId: Optional<number>
  userId: Optional<number>
  userIpAddr: string
  slug: string
  lastRevisionId: Optional<number>
  revisionComments: Optional<string>
  wikitext: Optional<string>
  title: Optional<string>
  altTitle: Optional<string>
  tags: string[]
  layout: Optional<Nullable<Layout>>
}
export async function pageEdit(
  input: PageEditInput,
  requestContext: RequestContext
): Promise<CreatePageRevisionOutput> {
  const {
    siteId,
    pageId,
    userId,
    userIpAddr,
    slug,
    lastRevisionId,
    revisionComments,
    wikitext,
    title,
    altTitle,
    tags,
    layout
  } = input
  const pageReference = pageId ? { page: pageId, last_revision_id: lastRevisionId } : {}
  return client.request(
    pageId ? "page_edit" : "page_create",
    {
      site_id: siteId,
      slug,
      ...pageReference,
      user_id: userId,
      ip_address: userIpAddr,
      revision_comments: revisionComments,
      wikitext,
      title,
      alt_title: altTitle,
      tags,
      layout:
        layout !== undefined
          ? (Layout[layout?.toUpperCase() as keyof typeof Layout] ?? null)
          : undefined
    },
    requestContext
  )
}

export async function pageGet(
  siteId: number,
  page: number | string,
  requestContext: RequestContext = {}
): Promise<Nullable<unknown>> {
  return client.request(
    "page_get",
    {
      site_id: siteId,
      page,
      details: {
        compiled_html: false,
        wikitext: false
      }
    },
    requestContext
  )
}

export async function pageViewPermission(
  siteId: number,
  page: number | string,
  requestContext: RequestContext = {}
): Promise<boolean> {
  return client.request(
    "page_view_permission",
    {
      site_id: siteId,
      page
    },
    requestContext
  )
}

export interface PageBacklinkView {
  slug: string
  title: string
}

export async function pageBacklinksView(
  siteId: number,
  page: number | string,
  requestContext: RequestContext = {}
): Promise<PageBacklinkView[]> {
  return client.request(
    "page_backlinks_view",
    {
      site_id: siteId,
      page
    },
    requestContext
  )
}

export interface SiteToolsPageView {
  slug: string
  title: string
}

export interface SiteToolsWantedPageView {
  slug: string
  sources: SiteToolsPageView[]
}

export interface PageDraftView {
  slug: string
  title: string
}

export interface PageDraftIdentityInput {
  siteId: number
  userId: number
  pageId?: number
  slug: string
}

export interface SavePageDraftInput extends PageDraftIdentityInput {
  title: string
  wikitext: string
}

export async function pageDraftSave(
  input: SavePageDraftInput,
  requestContext: RequestContext = {}
): Promise<boolean> {
  return client.request(
    "page_draft_save",
    {
      site_id: input.siteId,
      user_id: input.userId,
      page_id: input.pageId,
      slug: input.slug,
      title: input.title,
      wikitext: input.wikitext
    },
    requestContext
  )
}

export async function pageDraftExists(
  input: PageDraftIdentityInput,
  requestContext: RequestContext = {}
): Promise<boolean> {
  return client.request(
    "page_draft_exists",
    {
      site_id: input.siteId,
      user_id: input.userId,
      page_id: input.pageId,
      slug: input.slug
    },
    requestContext
  )
}

export async function pageDraftRemove(
  input: PageDraftIdentityInput,
  requestContext: RequestContext = {}
): Promise<boolean> {
  return client.request(
    "page_draft_remove",
    {
      site_id: input.siteId,
      user_id: input.userId,
      page_id: input.pageId,
      slug: input.slug
    },
    requestContext
  )
}

export async function siteToolsOrphanedPages(
  siteId: number,
  requestContext: RequestContext = {}
): Promise<SiteToolsPageView[]> {
  return client.request("site_tools_orphaned_pages", { site_id: siteId }, requestContext)
}

export async function siteToolsWantedPages(
  siteId: number,
  requestContext: RequestContext = {}
): Promise<SiteToolsWantedPageView[]> {
  return client.request("site_tools_wanted_pages", { site_id: siteId }, requestContext)
}

export async function siteToolsListDrafts(
  siteId: number,
  requestContext: RequestContext = {}
): Promise<PageDraftView[]> {
  return client.request("site_tools_list_drafts", { site_id: siteId }, requestContext)
}

export async function pageWatchers(
  siteId: number,
  pageId: number,
  requestContext: RequestContext = {}
): Promise<UserInfo[]> {
  return client.request(
    "page_watchers",
    {
      site_id: siteId,
      page_id: pageId
    },
    requestContext
  )
}

export interface PageWhoRatedVote {
  user: UserInfo
  value: number
}

export async function pageWhoRated(
  siteId: number,
  pageId: number,
  requestContext: RequestContext = {}
): Promise<PageWhoRatedVote[]> {
  return client.request(
    "page_who_rated",
    {
      site_id: siteId,
      page_id: pageId
    },
    requestContext
  )
}

export interface PageMetaTag {
  name: string
  content: string
  all_pages: boolean
}

export async function pageMetaTags(
  siteId: number,
  pageId: number,
  requestContext: RequestContext = {}
): Promise<PageMetaTag[]> {
  return client.request(
    "page_meta_tags",
    {
      site_id: siteId,
      page_id: pageId
    },
    requestContext
  )
}

export interface WikidotSiteChangesFeedOutput {
  site_name: string
  items: {
    revision_id: number
    revision_number: number
    is_new_page: boolean
    is_file: boolean
    created_at: string
    slug: string
    title: string
    change_type: string
    user_id: number
    user_name: string
    body_html: string
  }[]
}

export async function wikidotSiteChangesFeed(
  siteId: number,
  requestContext: RequestContext = {}
): Promise<WikidotSiteChangesFeedOutput | null> {
  return client.request(
    "wikidot_site_changes_feed",
    {
      site_id: siteId
    },
    requestContext
  )
}

export async function pageMetaTagSet(
  siteId: number,
  pageId: number,
  name: string,
  content: string,
  allPages: boolean,
  requestContext: RequestContext = {}
): Promise<void> {
  return client.request(
    "page_meta_tag_set",
    {
      site_id: siteId,
      page_id: pageId,
      name,
      content,
      all_pages: allPages
    },
    requestContext
  )
}

export async function pageMetaTagDelete(
  siteId: number,
  pageId: number,
  name: string,
  allPages: boolean,
  requestContext: RequestContext = {}
): Promise<void> {
  return client.request(
    "page_meta_tag_delete",
    {
      site_id: siteId,
      page_id: pageId,
      name,
      all_pages: allPages
    },
    requestContext
  )
}

export interface WikidotSiteChangesModuleInput {
  pageId?: string
  page: string
  perpage: string
  categoryId?: string
  options: string
}

export interface WikidotSiteChangesModuleOutput {
  status: string
  body: string
}

export async function wikidotSiteChangesModule(
  siteId: number,
  input: WikidotSiteChangesModuleInput,
  requestContext: RequestContext = {}
): Promise<WikidotSiteChangesModuleOutput> {
  return client.request(
    "wikidot_site_changes_module",
    {
      site_id: siteId,
      page: input.page,
      perpage: input.perpage,
      options: input.options,
      ...(input.pageId === undefined ? {} : { page_id: input.pageId }),
      ...(input.categoryId === undefined ? {} : { category_id: input.categoryId })
    },
    requestContext
  )
}

export async function pageEditPermission(
  requestContext: RequestContext = {}
): Promise<{ can_edit: boolean }> {
  return client.request("page_edit_permission", {}, requestContext)
}

export async function wikidotLegacySetTags(
  pageId: number,
  lastRevisionId: number,
  actionIndex: number,
  actionFingerprint: string,
  userId: number,
  ipAddress: string,
  requestContext: RequestContext
): Promise<Optional<CreatePageRevisionOutput>> {
  return client.request(
    "wikidot_legacy_set_tags",
    {
      page_id: pageId,
      last_revision_id: lastRevisionId,
      action_index: actionIndex,
      action_fingerprint: actionFingerprint,
      user_id: userId,
      ip_address: ipAddress
    },
    requestContext
  )
}

export interface WikidotPageDiscussionOutput {
  thread_id: number
  thread_unix_title: string
}

export async function wikidotPageDiscussionCreate(
  siteId: number,
  pageId: number,
  requestContext: RequestContext = {}
): Promise<Nullable<WikidotPageDiscussionOutput>> {
  return client.request(
    "wikidot_page_discussion_create",
    {
      site_id: siteId,
      page_id: pageId
    },
    requestContext
  )
}

/* ----- Page History ----- */
export interface PageRevisionModelFiltered {
  revision_id: number
  revision_type: PageRevisionType
  created_at: string
  updated_at: Nullable<string>
  from_wikidot: boolean
  revision_number: number
  page_id: number
  site_id: number
  user_id: number
  author: Nullable<UserInfo>
  changes: string[]
  wikitext: Nullable<string>
  compiled_body_html: Nullable<string>
  compiled_body_styles: Nullable<string[]>
  compiled_top_bar_html: Nullable<string>
  compiled_side_bar_html: Nullable<string>
  compiled_at: string
  compiled_generator: string
  comments: Nullable<string>
  hidden: string[]
  title: Nullable<string>
  alt_title: Nullable<string>
  slug: Nullable<string>
  tags: Nullable<string[]>
}

export interface PageHistoryEntry {
  history_kind: "page" | "file"
  history_row_id: string
  timeline_number: number
  page_revision_number: Nullable<number>
  history_action_revision_id?: Nullable<number>
  revision_id: number
  revision_type: PageRevisionType | FileRevisionType
  created_at: string
  revision_number: number
  page_id: number
  site_id: number
  user_id: number
  author: Nullable<UserInfo>
  changes: string[]
  comments: Nullable<string>
  wikitext: Nullable<string>
  compiled_body_html: Nullable<string>
  compiled_body_styles: Nullable<string[]>
  compiled_top_bar_html: Nullable<string>
  compiled_side_bar_html: Nullable<string>
  compiled_at: Nullable<string>
  compiled_generator: Nullable<string>
}
export async function pageHistory(
  siteId: number,
  pageId: Optional<number>,
  revisionNumber: Optional<number>,
  limit: Optional<number>,
  requestContext: RequestContext = {},
  includeFileRevisions = false
): Promise<PageHistoryEntry[]> {
  const requestedLimit = limit ?? defaults.page.history.limit
  if (pageId === undefined) return []

  if (!includeFileRevisions) {
    const pageRevisions = (await client.request(
      "page_revision_range",
      {
        site_id: siteId,
        page_id: pageId,
        revision_number: revisionNumber ?? defaults.page.history.revisionNumber,
        revision_direction: "before",
        limit: Math.min(Math.max(requestedLimit, 1), MAX_PAGE_HISTORY_REVISIONS)
      },
      requestContext
    )) as PageRevisionModelFiltered[]
    return pageRevisions.map((revision) => ({
      ...revision,
      history_kind: "page",
      history_row_id: String(revision.revision_id),
      timeline_number: revision.revision_number,
      page_revision_number: revision.revision_number
    }))
  }

  const limitPerSource =
    revisionNumber !== undefined && revisionNumber >= 0
      ? MAX_PAGE_HISTORY_REVISIONS
      : Math.min(Math.max(requestedLimit, 1), MAX_PAGE_HISTORY_REVISIONS)

  const [pageRevisionsRaw, pageRevisionCountRaw, fileHistory] = await Promise.all([
    client.request(
      "page_revision_range",
      {
        site_id: siteId,
        page_id: pageId,
        revision_number: -1,
        revision_direction: "before",
        limit: limitPerSource
      },
      requestContext
    ),
    client.request(
      "page_revision_count",
      {
        site_id: siteId,
        page: pageId
      },
      requestContext
    ),
    pageFileRevisionHistory(siteId, pageId, limitPerSource, requestContext)
  ])
  const pageRevisions = pageRevisionsRaw as PageRevisionModelFiltered[]
  const pageRevisionCount = pageRevisionCountRaw as PageRevisionCountOutput
  const timeline = mergeWikidotHistoryTimeline(
    pageRevisions,
    fileHistory.revisions,
    pageRevisionCount.revision_count + fileHistory.revision_count
  )
  return timeline
    .filter(
      (revision) =>
        revisionNumber === undefined ||
        revisionNumber < 0 ||
        revision.timeline_number <= revisionNumber
    )
    .slice(0, Math.min(Math.max(requestedLimit, 1), MAX_PAGE_HISTORY_REVISIONS)) as PageHistoryEntry[]
}

export interface PageRevisionCountOutput {
  revision_count: number
  first_revision: number
  last_revision: number
}

/* ----- Page Move ----- */
interface PageMove {
  old_slug: string
  new_slug: string
  revision_id: number
  revision_number: number
  parser_errors: Nullable<ParseError[]>
}
export interface PageMoveInput {
  siteId: number
  pageId: Optional<number>
  userId: number
  userIpAddr: string
  slug: string
  lastRevisionId: number
  newSlug: string
  revisionComments: Optional<string>
}
export async function pageMove(
  input: PageMoveInput,
  requestContext: RequestContext
): Promise<PageMove> {
  const {
    siteId,
    pageId,
    userId,
    userIpAddr,
    slug,
    lastRevisionId,
    newSlug,
    revisionComments
  } = input
  return client.request(
    "page_move",
    {
      site_id: siteId,
      page: pageId ?? slug,
      new_slug: newSlug,
      user_id: userId,
      ip_address: userIpAddr,
      last_revision_id: lastRevisionId,
      revision_comments: revisionComments
    },
    requestContext
  )
}

/* ----- Page Revision ----- */
export async function pageRevision(
  siteId: number,
  pageId: Optional<number>,
  revisionNumber: Optional<number>,
  compiledHtml?: boolean,
  wikitext?: boolean,
  requestContext: RequestContext = {}
): Promise<Nullable<PageRevisionModelFiltered>> {
  return client.request(
    "page_revision_get",
    {
      site_id: siteId,
      page_id: pageId,
      revision_number: revisionNumber ?? defaults.page.history.revisionNumber,
      details: {
        compiled_html: compiledHtml ?? false,
        wikitext: wikitext ?? false
      }
    },
    requestContext
  )
}

export async function pageRevisionById(
  siteId: number,
  revisionId: number,
  compiledHtml: boolean,
  wikitext: boolean,
  requestContext: RequestContext = {}
): Promise<Nullable<PageRevisionModelFiltered>> {
  return client.request(
    "page_revision_get_by_id",
    {
      site_id: siteId,
      revision_id: revisionId,
      details: {
        compiled_html: compiledHtml,
        wikitext
      }
    },
    requestContext
  )
}

export type PageRevisionDiffLineKind = "added" | "removed" | "unchanged"

export interface PageRevisionDiffLine {
  kind: PageRevisionDiffLineKind
  text: string
}

export interface PageRevisionDiffOutput {
  site_id: number
  page_id: number
  from_revision_number: number
  to_revision_number: number
  lines: PageRevisionDiffLine[]
}

export async function pageRevisionDiff(
  siteId: number,
  pageId: number,
  fromRevisionNumber: number,
  toRevisionNumber: number,
  requestContext: RequestContext = {}
): Promise<Nullable<PageRevisionDiffOutput>> {
  return client.request(
    "page_revision_diff",
    {
      site_id: siteId,
      page_id: pageId,
      from_revision_number: fromRevisionNumber,
      to_revision_number: toRevisionNumber
    },
    requestContext
  )
}

/* ----- Page Rollback ----- */
export interface PageRollbackInput {
  siteId: number
  pageId: Optional<number>
  userId: number
  userIpAddr: string
  slug: string
  lastRevisionId: number
  revisionNumber: Optional<number>
  revisionComments: Optional<string>
}
export async function pageRollback(
  input: PageRollbackInput,
  requestContext: RequestContext
): Promise<Nullable<CreatePageRevisionOutput>> {
  const {
    siteId,
    pageId,
    userId,
    userIpAddr,
    slug,
    lastRevisionId,
    revisionNumber,
    revisionComments
  } = input
  return client.request(
    "page_rollback",
    {
      site_id: siteId,
      page: pageId ?? slug,
      user_id: userId,
      ip_address: userIpAddr,
      last_revision_id: lastRevisionId,
      revision_number: revisionNumber ?? defaults.page.history.revisionNumber,
      revision_comments: revisionComments
    },
    requestContext
  )
}

/* ----- Page Vote List ----- */
export async function pageVoteList(
  pageId: Optional<number>,
  requestContext: RequestContext
): Promise<PageVoteModel[]> {
  return client.request(
    "vote_list",
    {
      type: "Page",
      id: pageId,
      deleted: false,
      disabled: false,
      start_id: 0,
      limit: 100
    },
    requestContext
  )
}

/* ----- Page Vote Cast ----- */
export async function pageVoteCast(
  pageId: Optional<number>,
  value: number,
  requestContext: RequestContext
): Promise<Nullable<PageVoteModel>> {
  return client.request(
    "vote_set",
    {
      page_id: pageId,
      value
    },
    requestContext
  )
}

/* ----- Page Vote Remove ----- */
export async function pageVoteRemove(
  pageId: Optional<number>,
  requestContext: RequestContext
): Promise<PageVoteModel> {
  return client.request("vote_remove", { page_id: pageId }, requestContext)
}

/* ----- Renderer-bound Wikidot Rate action ----- */
export async function wikidotLegacyRate(
  pageId: number,
  lastRevisionId: number,
  actionIndex: number,
  actionFingerprint: string,
  requestContext: RequestContext
): Promise<PageScore> {
  return client.request(
    "wikidot_legacy_rate",
    {
      page_id: pageId,
      last_revision_id: lastRevisionId,
      action_index: actionIndex,
      action_fingerprint: actionFingerprint
    },
    requestContext
  )
}

/* ----- Page Rerender ----- */
// Not used for now
export async function pageRerender(siteId: number, pageId: number): Promise<void> {
  return client.request("page_rerender", {
    site_id: siteId,
    page_id: pageId
  })
}

/* ----- Page Layout ----- */
export async function pageLayout(
  siteId: number,
  pageId: number,
  userId: number,
  userIpAddr: string,
  layout: Optional<Nullable<Layout>>,
  requestContext: RequestContext
): Promise<void> {
  return client.request(
    "page_set_layout",
    {
      site_id: siteId,
      page_id: pageId,
      user_id: userId,
      ip_address: userIpAddr,
      layout: Layout[layout?.toUpperCase() as keyof typeof Layout] ?? null
    },
    requestContext
  )
}

/* ----- Page Parent Update ----- */
interface PageParentUpdateResult {
  added: Nullable<number[]>
  removed: Nullable<boolean[]>
}
export async function pageParentUpdate(
  siteId: number,
  pageId: number | string,
  userId: number,
  add: Optional<string[]>,
  remove: Optional<string[]>,
  requestContext: RequestContext
): Promise<PageParentUpdateResult> {
  return client.request(
    "parent_update",
    {
      site_id: siteId,
      child: pageId,
      user_id: userId,
      add,
      remove
    },
    requestContext
  )
}

/* ----- Page Parent Get ----- */
export async function pageParentGet(
  siteId: number,
  pageId: Optional<number>,
  slug: string,
  requestContext: RequestContext
): Promise<string[]> {
  return client.request(
    "parent_get_all",
    {
      site_id: siteId,
      page: pageId ?? slug
    },
    requestContext
  )
}

/* ----- Page Deleted Get ----- */
export interface PageDeletedGet {
  page_id: number
  page_created_at: string
  page_updated_at: Optional<string>
  page_deleted_at: string
  page_revision_count: number
  site_id: number
  discussion_thread_id: Optional<number>
  hidden_fields: string[]
  title: string
  alt_title: Optional<string>
  slug: string
  tags: string[]
  rating: number
}
export async function pageDeletedGet(
  requestContext: RequestContext
): Promise<PageDeletedGet[]> {
  const siteId = requestContext?.siteId
  const slug = requestContext?.page
  if (siteId === undefined || typeof slug !== "string") {
    throw new Error("Deleted page lookup requires trusted site and page context")
  }

  return client.request(
    "page_get_deleted",
    {
      site_id: siteId,
      slug
    },
    requestContext
  )
}

/* ----- Page Restore ----- */
interface PageRestore {
  slug: string
  revision_id: number
  revision_number: number
  parser_errors: ParseError[]
}
export async function pageRestore(
  siteId: number,
  pageId: number,
  userId: number,
  userIpAddr: string,
  revisionComments: Optional<string>,
  requestContext: RequestContext
): Promise<PageRestore> {
  return client.request(
    "page_restore",
    {
      site_id: siteId,
      page_id: pageId,
      user_id: userId,
      ip_address: userIpAddr,
      revision_comments: revisionComments
    },
    requestContext
  )
}

/* ----- Page Lock History ----- */
export interface PageLockModel {
  page_lock_id: number
  created_at: string
  updated_at: Nullable<string>
  deleted_at: Nullable<string>
  expires_at: Nullable<string>
  from_wikidot: boolean
  lock_type: string
  page_id: number
  user_id: number
  reason: string
}
export async function pageLockHistory(
  pageId: number,
  reqContext: RequestContext
): Promise<PageLockModel[]> {
  return client.request("page_lock_get_history", { page: pageId }, reqContext)
}

/* ----- Page Lock Remove ----- */
export async function pageLockRemove(
  pageId: number,
  userIpAddr: string,
  reqContext: RequestContext
): Promise<void> {
  return client.request(
    "page_lock_remove",
    {
      page: pageId,
      ip_address: userIpAddr
    },
    reqContext
  )
}

/* ----- Page Lock Create ----- */
export async function pageLockCreate(
  pageId: number,
  lockType: PageLockType,
  reason: string,
  expiresAt: Optional<string>,
  overrideExisting: boolean,
  userIpAddr: string,
  reqContext: RequestContext
): Promise<void> {
  return client.request(
    "page_lock_create",
    {
      page: pageId,
      lock_type: lockType,
      reason: reason,
      expires_at: expiresAt ?? null,
      from_wikidot: false,
      override_existing: overrideExisting ?? false,
      ip_address: userIpAddr
    },
    reqContext
  )
}

/* ----- Page Score ----- */
export interface PageScore {
  page_id: number
  score: number
}
export async function pageScore(requestContext: RequestContext): Promise<PageScore> {
  const siteId = requestContext?.siteId
  const page = requestContext?.page
  if (siteId === undefined || page === undefined) {
    throw new Error("Page score lookup requires trusted site and page context")
  }

  return client.request(
    "page_get_score",
    {
      site_id: siteId,
      page
    },
    requestContext
  )
}
