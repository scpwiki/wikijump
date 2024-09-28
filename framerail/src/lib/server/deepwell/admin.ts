import { client } from "$lib/server/deepwell"
import type { Nullable, Optional } from "$lib/types"
import { Layout } from "$lib/types"

export async function siteUpdate(
  siteId: number,
  userId: number,
  name: Optional<String>,
  slug: Optional<String>,
  tagline: Optional<String>,
  description: Optional<String>,
  locale: Optional<String>,
  layout: Optional<Nullable<Layout>>
): Promise<object> {
  return client.request("site_update", {
    site: siteId,
    user_id: userId,
    name,
    slug,
    tagline,
    description,
    locale,
    layout:
      layout !== undefined
        ? Layout[layout?.toUpperCase() as keyof typeof Layout] ?? null
        : undefined
  })
}
