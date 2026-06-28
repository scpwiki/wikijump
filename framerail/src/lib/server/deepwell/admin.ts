import { client } from "$lib/server/deepwell"
import { Layout } from "$lib/types"

import type { Nullable, Optional, SiteModel } from "$lib/types"

type SiteUpdateRequestContext = {
  sessionToken: string
  siteId: number
}

export async function siteUpdate(
  siteId: number,
  userId: number,
  userIpAddr: string,
  name: Optional<string>,
  slug: Optional<string>,
  tagline: Optional<string>,
  description: Optional<string>,
  defaultPage: Optional<string>,
  locale: Optional<string>,
  layout: Optional<Nullable<Layout>>,
  requestContext: SiteUpdateRequestContext
): Promise<SiteModel> {
  return client.request(
    "site_update",
    {
      site: siteId,
      user_id: userId,
      name,
      slug,
      tagline,
      description,
      default_page: defaultPage,
      locale,
      layout:
        layout !== undefined
          ? (Layout[layout?.toUpperCase() as keyof typeof Layout] ?? null)
          : undefined,
      ip_address: userIpAddr
    },
    requestContext
  )
}
