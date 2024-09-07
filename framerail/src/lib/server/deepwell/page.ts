import defaults from "$lib/defaults.ts"
import { client } from "$lib/server/deepwell/index.ts"
import type { Optional } from "$lib/types.ts"

export async function pageDelete(
  siteId: number,
  pageId: Optional<number>,
  userId: number,
  slug: string,
  revisionComments: Optional<string>
): Promise<object> {
  return client.request("page_delete", {
    site_id: siteId,
    page: pageId ?? slug,
    user_id: userId,
    revision_comments: revisionComments
  })
}

export async function pageEdit(
  siteId: number,
  pageId: Optional<number>,
  userId: number,
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
    user_id: userId,
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
    revision_number: revisionNumber ?? defaults.page.history.revisionNumber,
    revision_direction: "before",
    limit: limit ?? defaults.page.history.limit
  })
}

export async function pageMove(
  siteId: number,
  pageId: Optional<number>,
  userId: number,
  slug: string,
  newSlug: string,
  revisionComments: Optional<string>
): Promise<object> {
  return client.request("page_move", {
    site_id: siteId,
    page: pageId ?? slug,
    new_slug: newSlug,
    user_id: userId,
    revision_comments: revisionComments
  })
}

export async function pageRevision(
  siteId: number,
  pageId: Optional<number>,
  revisionNumber: Optional<number>,
  compiledHtml?: boolean,
  wikitext?: boolean
): Promise<object> {
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

export async function pageVote(
  siteId: number,
  pageId: Optional<number>,
  userId: number,
  action: String,
  value: number
) {
  let actionLower = action.toLowerCase()
  if (actionLower === "set") {
    return client.request("vote_set", {
      page_id: pageId,
      user_id: userId,
      value
    })
  } else if (actionLower === "remove") {
    return client.request("vote_remove", {
      page_id: pageId,
      user_id: userId
    })
  } else if (actionLower === "get_list") {
    return client.request("vote_list", {
      type: "Page",
      id: pageId,
      deleted: false,
      disabled: false,
      start_id: 0,
      limit: 100
    })
  }
}
