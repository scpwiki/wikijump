import { client } from "$lib/server/deepwell/index.ts"
import type { Optional } from "$lib/types.ts"

export interface PageRoute {
  slug: string
  extra: string
}

export function pageView(
  domain: string,
  locale: string,
  route: Optional<PageRoute>,
  sessionToken: Optional<string>
): object {
  return client.request("page_view", {
    domain,
    locale,
    session_token: sessionToken,
    route
  })
}
