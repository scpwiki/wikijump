import { client } from "$lib/server/deepwell"
import type { Optional } from "$lib/types"

export interface PageRoute {
  slug: string
  extra: string
}

export async function pageView(
  domain: string,
  locales: string[],
  route: Optional<PageRoute>,
  sessionToken: Optional<string>
): Promise<object> {
  return client.request("page_view", {
    domain,
    locales,
    session_token: sessionToken,
    route
  })
}

export async function adminView(
  domain: string,
  locales: string[],
  sessionToken: Optional<string>
): Promise<object> {
  return client.request("admin_view", {
    domain,
    locales,
    session_token: sessionToken
  })
}
