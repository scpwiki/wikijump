import { pageEdit } from "$lib/server/deepwell/edit.ts"

export async function POST(event) {
  let data = await event.request.formData()
  let slug = event.params.slug

  let pageIdVal = data.get("page-id")?.toString()
  let pageId = pageIdVal ? parseInt(pageIdVal) : null
  let siteId = parseInt(data.get("site-id")?.toString() ?? "1")
  let comments = data.get("comments")?.toString() ?? ""
  let wikitext = data.get("wikitext")?.toString()
  let title = data.get("title")?.toString()
  let altTitle = data.get("alt-title")?.toString()
  let tagsStr = data.get("tags")?.toString().trim()
  let tags: string[] = []
  if (tagsStr?.length) tags = tagsStr.split(" ").filter(tag=>tag.length)

  let res = await pageEdit(siteId, pageId, slug, comments, wikitext, title, altTitle, tags)

  return new Response(JSON.stringify(res));
}
