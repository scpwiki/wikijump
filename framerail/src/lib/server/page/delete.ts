import { wellfetch } from "$lib/server/deepwell/index.ts"
import type { Optional } from "$lib/types.ts"

export async function pageDelete(
  siteId: number,
  pageId: Optional<number>,
  slug: string,
  revisionComments: Optional<string>
): object {
  const response = await wellfetch("/page", {
    method: "DELETE",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      siteId,
      page: pageId ?? slug,
      userId: 1, // TODO: identify user session and pass the user to the API request
      revisionComments
    })
  })

  if (!response.ok) {
    throw new Error("Unable to delete page")
  }

  return response.json()
}
