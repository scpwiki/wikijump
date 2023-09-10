import { wellfetch } from "$lib/server/deepwell/index.ts"
import type { Optional } from "$lib/types.ts"

export async function pageEdit(
  siteId: number,
  pageId: Optional<number>,
  slug: string,
  revisionComments: Optional<string>,
  wikitext: string,
  title: string,
  altTitle: string,
  tags: string[],
): object {
  let endpoint = "/page"
  if (!pageId) {
    // Assume page creation
    endpoint += "/create"
  }
  const response = await wellfetch(endpoint, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      siteId,
      page: pageId ?? slug,
      slug,
      userId: 1, // TODO: identify user session and pass the user to the API request
      revisionComments,
      wikitext,
      title,
      altTitle,
      tags,
    })
  })

  if (!response.ok) {
    throw new Error("Unable to save data to server")
  }

  return response.json()
}
