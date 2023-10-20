import { client } from "$lib/server/deepwell/index.ts"
import type { Optional } from "$lib/types.ts"

export async function pageDelete(
  siteId: number,
  pageId: Optional<number>,
  slug: string,
  revisionComments: Optional<string>
): Promise<object> {
  return client.request("page_delete", {
    site_id: siteId,
    page: pageId ?? slug,
    user_id: 1, // TODO: identify user session and pass the user to the API request
    revision_comments: revisionComments
  })
}

export async function pageEdit(
  siteId: number,
  pageId: Optional<number>,
  slug: string,
  revisionComments: Optional<string>,
  wikitext: string,
  title: string,
  altTitle: string,
  tags: string[]
): Promise<object> {
  return client.request(pageId ? "page_edit" : "page_create", {
    site_id: siteId,
    page: pageId ?? slug,
    user_id: 1, // TODO: identify user session and pass the user to the API request
    revision_comments: revisionComments,
    wikitext,
    title,
    alt_title: altTitle,
    tags
  })
}

export async function pageHistory(
  siteId: number,
  pageId: Optional<number>,
  revisionNumber: Optional<number>,
  limit: Optional<number>
): Promise<object> {
  return client.request("page_revision_range", {
    site_id: siteId,
    page_id: pageId,
    revision_number: revisionNumber,
    revision_direction: "before",
    limit
  })
}

export async function pageMove(
  siteId: number,
  pageId: Optional<number>,
  slug: string,
  newSlug: string,
  revisionComments: Optional<string>
): Promise<object> {
  return client.request("page_move", {
    site_id: siteId,
    page: pageId ?? slug,
    new_slug: newSlug,
    user_id: 1, // TODO: identify user session and pass the user to the API request
    revision_comments: revisionComments
  })
}