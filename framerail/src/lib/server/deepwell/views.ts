import { wellfetch } from "$lib/server/deepwell/index.ts"
import type { Optional } from "$lib/types.ts"

export interface PageRoute {
  slug: string
  extra: string
}

export async function pageView(
  domain: string,
  route: Optional<PageRoute>,
  sessionToken: Optional<string>,
  language: string
): object {
  const response = await wellfetch("/view/page", {
    method: "PUT",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      domain,
      sessionToken,
      route
    })
  })

  if (!response.ok) {
    throw new Error("Unable to get view data from server")
  }

  return response.json()
}
