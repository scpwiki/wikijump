import * as page from "$lib/server/deepwell/page"

// Handling of server events from client

export async function POST(event) {
  let data = await event.request.formData()
  let slug = event.params.slug

  let extra = event.params.extra
    ?.toLowerCase()
    .split("/")
    .filter((flag) => flag.length)

  let pageIdVal = data.get("page-id")?.toString()
  let pageId = pageIdVal ? parseInt(pageIdVal) : null
  let siteIdVal = data.get("site-id")?.toString()
  let siteId = siteIdVal ? parseInt(siteIdVal) : null

  let res: object = {}

  if (extra.includes("edit")) {
    /** Edit or create page. */
    let comments = data.get("comments")?.toString() ?? ""
    let wikitext = data.get("wikitext")?.toString()
    let title = data.get("title")?.toString()
    let altTitle = data.get("alt-title")?.toString()
    let tagsStr = data.get("tags")?.toString().trim()
    let tags: string[] = []
    if (tagsStr?.length) tags = tagsStr.split(" ").filter((tag) => tag.length)

    res = await page.pageEdit(
      siteId,
      pageId,
      slug,
      comments,
      wikitext,
      title,
      altTitle,
      tags
    )
  } else if (extra.includes("history")) {
    /** Retrieve page revision list. */
    let revisionNumberStr = data.get("revision-number")?.toString()
    let revisionNumber = revisionNumberStr ? parseInt(revisionNumberStr) : null
    let limitStr = data.get("limit")?.toString()
    let limit = limitStr ? parseInt(limitStr) : null

    res = await page.pageHistory(siteId, pageId, revisionNumber, limit)
  } else if (extra.includes("move")) {
    /** Move page to new slug. */
    let comments = data.get("comments")?.toString() ?? ""
    let newSlug = data.get("new-slug")?.toString()

    res = await page.pageMove(siteId, pageId, slug, newSlug, comments)
  }

  return new Response(JSON.stringify(res))
}

/** Delete page. */
export async function DELETE(event) {
  let data = await event.request.formData()
  let slug = event.params.slug

  let pageIdVal = data.get("page-id")?.toString()
  let pageId = pageIdVal ? parseInt(pageIdVal) : null
  let siteId = parseInt(data.get("site-id")?.toString() ?? "1")
  let comments = data.get("comments")?.toString() ?? ""

  let res = await page.pageDelete(siteId, pageId, slug, comments)
  return new Response(JSON.stringify(res))
}
