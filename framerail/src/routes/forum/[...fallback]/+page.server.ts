import type { PageServerLoad } from "./$types"

import { loadForumFallback } from "$lib/server/load/forum"

export const load: PageServerLoad = async ({ parent }) => {
  return loadForumFallback(await parent())
}
