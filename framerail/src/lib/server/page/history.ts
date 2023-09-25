import { wellfetch } from "$lib/server/deepwell/index.ts"
import type { Optional } from "$lib/types.ts"

export async function pageHistory(
  siteId: number,
  pageId: Optional<number>,
  slug: string
): Promise<object> {
  const response = await wellfetch("/page/revision/range", {
    method: "PUT",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      siteId,
      pageId: pageId,
      revisionNumber: -1,
      revisionDirection: "before",
      limit: 200
    })
  })

  if (!response.ok) {
    throw new Error("Unable to get data from server")
  }

  return response.json()
}
