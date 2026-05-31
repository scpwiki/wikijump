import defaults from "$lib/defaults"

import { client } from "$lib/server/deepwell"
import { Layout, PageLockType } from "$lib/types"

import type {
  Nullable,
  Optional,
  PageRevisionType,
  PageVoteModel,
  ParseError
} from "$lib/types"
import type { RequestContext } from "../load/request-ctx"

/* ----- Page Delete ----- */
interface PageDelete {
  page_id: number
  revision_id: number
  revision_number: number
}
export async function pageDelete(
  siteId: number,
  pageId: Optional<number>,
  userId: number,
  userIpAddr: string,
  slug: string,
  lastRevisionId: number,
  revisionComments: Optional<string>
): Promise<PageDelete> {
  return client.request("page_delete", {
    site_id: siteId,
    page: pageId ?? slug,
    user_id: userId,
    ip_address: userIpAddr,
    last_revision_id: lastRevisionId,
    revision_comments: revisionComments
  })
}

/* ----- Page Edit ----- */
export interface CreatePageRevisionOutput {
  revision_id: number
  revision_number: number
  parser_errors: Nullable<ParseError[]>
}
export async function pageEdit(
  siteId: number,
  pageId: Optional<number>,
  userId: number,
  userIpAddr: string,
  slug: string,
  lastRevisionId: Optional<number>,
  revisionComments: Optional<string>,
  wikitext: Optional<string>,
  title: Optional<string>,
  altTitle: Optional<string>,
  tags: string[],
  layout: Optional<Nullable<Layout>>
): Promise<CreatePageRevisionOutput> {
  return client.request(pageId ? "page_edit" : "page_create", {
    site_id: siteId,
    page: pageId ?? slug,
    slug,
    user_id: userId,
    ip_address: userIpAddr,
    last_revision_id: lastRevisionId,
    revision_comments: revisionComments,
    wikitext,
    title,
    alt_title: altTitle,
    tags,
    layout:
      layout !== undefined
        ? (Layout[layout?.toUpperCase() as keyof typeof Layout] ?? null)
        : undefined
  })
}

export async function pageEditPermission(
  requestContext: RequestContext = {}
): Promise<{ can_edit: boolean }> {
  return client.request("page_edit_permission", {}, requestContext)
}

/* ----- Page History ----- */
export interface PageRevisionModelFiltered {
  revision_id: number
  revision_type: PageRevisionType
  created_at: number
  updated_at: Nullable<number>
  from_wikidot: boolean
  revision_number: number
  page_id: number
  site_id: number
  user_id: number
  changes: string[]
  wikitext: Nullable<string>
  compiled_body_html: Nullable<string>
  compiled_top_bar_html: Nullable<string>
  compiled_side_bar_html: Nullable<string>
  compiled_at: number
  compiled_generator: string
  comments: Nullable<string>
  hidden: string[]
  title: Nullable<string>
  alt_title: Nullable<string>
  slug: Nullable<string>
  tags: Nullable<string[]>
}
export async function pageHistory(
  siteId: number,
  pageId: Optional<number>,
  revisionNumber: Optional<number>,
  limit: Optional<number>
): Promise<PageRevisionModelFiltered[]> {
  return client.request("page_revision_range", {
    site_id: siteId,
    page_id: pageId,
    revision_number: revisionNumber ?? defaults.page.history.revisionNumber,
    revision_direction: "before",
    limit: limit ?? defaults.page.history.limit
  })
}

/* ----- Page Move ----- */
interface PageMove {
  old_slug: string
  new_slug: string
  revision_id: number
  revision_number: number
  parser_errors: Nullable<ParseError[]>
}
export async function pageMove(
  siteId: number,
  pageId: Optional<number>,
  userId: number,
  userIpAddr: string,
  slug: string,
  lastRevisionId: number,
  newSlug: string,
  revisionComments: Optional<string>
): Promise<PageMove> {
  return client.request("page_move", {
    site_id: siteId,
    page: pageId ?? slug,
    new_slug: newSlug,
    user_id: userId,
    ip_address: userIpAddr,
    last_revision_id: lastRevisionId,
    revision_comments: revisionComments
  })
}

/* ----- Page Revision ----- */
export async function pageRevision(
  siteId: number,
  pageId: Optional<number>,
  revisionNumber: Optional<number>,
  compiledHtml?: boolean,
  wikitext?: boolean
): Promise<Nullable<PageRevisionModelFiltered>> {
  return client.request("page_revision_get", {
    site_id: siteId,
    page_id: pageId,
    revision_number: revisionNumber ?? defaults.page.history.revisionNumber,
    details: {
      compiled_html: compiledHtml ?? false,
      wikitext: wikitext ?? false
    }
  })
}

/* ----- Page Rollback ----- */
export async function pageRollback(
  siteId: number,
  pageId: Optional<number>,
  userId: number,
  userIpAddr: string,
  slug: string,
  lastRevisionId: number,
  revisionNumber: Optional<number>,
  revisionComments: Optional<string>
): Promise<Nullable<CreatePageRevisionOutput>> {
  return client.request("page_rollback", {
    site_id: siteId,
    page: pageId ?? slug,
    user_id: userId,
    ip_address: userIpAddr,
    last_revision_id: lastRevisionId,
    revision_number: revisionNumber ?? defaults.page.history.revisionNumber,
    revision_comments: revisionComments
  })
}

/* ----- Page Vote List ----- */
export async function pageVoteList(
  siteId: number,
  pageId: Optional<number>
): Promise<PageVoteModel[]> {
  return client.request("vote_list", {
    type: "Page",
    id: pageId,
    deleted: false,
    disabled: false,
    start_id: 0,
    limit: 100
  })
}

/* ----- Page Vote Cast ----- */
export async function pageVoteCast(
  siteId: number,
  pageId: Optional<number>,
  userId: number,
  value: number
): Promise<Nullable<PageVoteModel>> {
  return client.request("vote_set", {
    page_id: pageId,
    user_id: userId,
    value
  })
}

/* ----- Page Vote Remove ----- */
export async function pageVoteRemove(
  siteId: number,
  pageId: Optional<number>,
  userId: number
): Promise<PageVoteModel> {
  return client.request("vote_remove", {
    page_id: pageId,
    user_id: userId
  })
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
  layout: Optional<Nullable<Layout>>
): Promise<void> {
  return client.request("page_set_layout", {
    site_id: siteId,
    page_id: pageId,
    user_id: userId,
    ip_address: userIpAddr,
    layout: Layout[layout?.toUpperCase() as keyof typeof Layout] ?? null
  })
}

/* ----- Page Parent Update ----- */
interface PageParentUpdate {
  added: Nullable<number[]>
  removed: Nullable<boolean[]>
}
export async function pageParentUpdate(
  siteId: number,
  pageId: number,
  userId: number,
  add: Optional<string[]>,
  remove: Optional<string[]>
): Promise<PageParentUpdate> {
  return client.request("parent_update", {
    site_id: siteId,
    child: pageId,
    user_id: userId,
    add,
    remove
  })
}

/* ----- Page Parent Get ----- */
export async function pageParentGet(
  siteId: number,
  pageId: Optional<number>,
  slug: string
): Promise<string[]> {
  return client.request("parent_get_all", {
    site_id: siteId,
    page: pageId ?? slug
  })
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
  siteId: number,
  slug: string
): Promise<PageDeletedGet[]> {
  return client.request("page_get_deleted", {
    site_id: siteId,
    slug
  })
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
  revisionComments: Optional<string>
): Promise<PageRestore> {
  return client.request("page_restore", {
    site_id: siteId,
    page_id: pageId,
    user_id: userId,
    ip_address: userIpAddr,
    revision_comments: revisionComments
  })
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
  reqContext: RequestContext
): Promise<PageLockModel[]> {
  return client.request("page_lock_get_history", {}, reqContext)
}

/* ----- Page Lock Remove ----- */
export async function pageLockRemove(
  userIpAddr: string,
  reqContext: RequestContext
): Promise<void> {
  return client.request(
    "page_lock_remove",
    {
      ip_address: userIpAddr
    },
    reqContext
  )
}

/* ----- Page Lock Create ----- */
export async function pageLockCreate(
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
export async function pageScore(
  siteId: number,
  pageId: Optional<number>,
  slug: string
): Promise<PageScore> {
  return client.request("page_get_score", {
    site_id: siteId,
    page: pageId ?? slug
  })
}
