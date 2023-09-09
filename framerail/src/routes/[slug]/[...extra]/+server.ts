import { pageEdit } from "$lib/server/deepwell/edit.ts"

export async function POST(event) {
  let data = await event.request.formData()

  let page = parseInt(data.get("page-id")?.toString() ?? "1")
  let siteId = parseInt(data.get("site-id")?.toString() ?? "1")
  let comments = data.get("comments")?.toString() ?? ""
  let wikitext = data.get("wikitext")?.toString()
  let title = data.get("title")?.toString()
  let altTitle = data.get("alt-title")?.toString()
  let tags = data.get("tags")?.toString().split(" ") ?? []

  let res = await pageEdit(siteId, page, comments, wikitext, title, altTitle, tags)

  return new Response(JSON.stringify(res));
}
