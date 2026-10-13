import { client } from "$lib/server/deepwell"

import type { Nullable } from "$lib/types"

export interface SiteIconData {
  site_id: number
  kind: string
  mime: string
  sha256_hex: string
  data: number[]
}

/** Fetch one site's stored icon bytes, or null when none are materialized. */
export async function getSiteIcon(
  siteId: number,
  kind: string
): Promise<Nullable<SiteIconData>> {
  return client.request("site_icon_get", { site: siteId, kind })
}
