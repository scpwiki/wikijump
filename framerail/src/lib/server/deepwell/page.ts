import defaults from "$lib/defaults"

import { client } from "$lib/server/deepwell"
import { Layout } from "$lib/types"

import type {
  Nullable,
  Optional,
  PageRevisionType,
  PageVoteModel,
  ParseError
} from "$lib/types"
import type { RequestContext } from "../request-context"

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
  revision_id: number
  revision_number: number
  parser_errors: Nullable<ParseError[]>
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
  return client.request(
    pageId ? "page_edit" : "page_create",
    {
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
    },
    requestContext
  )
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
  compiled_body_styles: Nullable<string[]>
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
  limit: Optional<number>,
  requestContext: RequestContext = {}
): Promise<PageRevisionModelFiltered[]> {
  return client.request(
    "page_revision_range",
    {
      site_id: siteId,
      page_id: pageId,
      revision_number: revisionNumber ?? defaults.page.history.revisionNumber,
      revision_direction: "before",
      limit: limit ?? defaults.page.history.limit
    },
    requestContext
  )
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
  userId: number,
  value: number,
  requestContext: RequestContext
): Promise<Nullable<PageVoteModel>> {
  return client.request(
    "vote_set",
    {
      page_id: pageId,
      user_id: userId,
      value
    },
    requestContext
  )
}

/* ----- Page Vote Remove ----- */
export async function pageVoteRemove(
  pageId: Optional<number>,
  userId: number,
  requestContext: RequestContext
): Promise<PageVoteModel> {
  return client.request(
    "vote_remove",
    {
      page_id: pageId,
      user_id: userId
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
interface PageParentUpdate {
  added: Nullable<number[]>
  removed: Nullable<boolean[]>
}
export async function pageParentUpdate(
  siteId: number,
  pageId: number,
  userId: number,
  add: Optional<string[]>,
  remove: Optional<string[]>,
  requestContext: RequestContext
): Promise<PageParentUpdate> {
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
  siteId: number,
  slug: string,
  requestContext: RequestContext
): Promise<PageDeletedGet[]> {
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

/* ----- Page Score ----- */
export interface PageScore {
  page_id: number
  score: number
}
export async function pageScore(
  siteId: number,
  pageId: Optional<number>,
  slug: string,
  requestContext: RequestContext
): Promise<PageScore> {
  return client.request(
    "page_get_score",
    {
      site_id: siteId,
      page: pageId ?? slug
    },
    requestContext
  )
}
