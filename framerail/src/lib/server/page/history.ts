import { wellfetch } from "$lib/server/deepwell/index.ts"
import type { Optional } from "$lib/types.ts"

export async function pageHistory(
  siteId: number,
  pageId: Optional<number>,
  revisionNumber: Optional<Number>,
  limit: Optional<Number>
): Promise<object> {
  const response = await wellfetch("/page/revision/range", {
    method: "PUT",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      siteId,
      pageId,
      revisionNumber,
      revisionDirection: "before",
      limit
    })
  })

  if (!response.ok) {
    throw new Error("Unable to get data from server")
  }

  return response.json()
}
