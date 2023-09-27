import { wellfetch } from "$lib/server/deepwell/index.ts"
import type { Optional } from "$lib/types.ts"

export async function pageMove(
  siteId: number,
  pageId: Optional<number>,
  slug: string,
  newSlug: string,
  revisionComments: Optional<string>
): Promise<object> {
  const response = await wellfetch("/page/move", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      siteId,
      page: pageId ?? slug,
      newSlug,
      userId: 1, // TODO: identify user session and pass the user to the API request
      revisionComments
    })
  })

  if (!response.ok) {
    throw new Error("Unable to send data to server")
  }

  return response.json()
}
