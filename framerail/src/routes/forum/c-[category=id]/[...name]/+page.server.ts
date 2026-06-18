import { loadForumCategory, parseForumRouteId } from "$lib/server/load/forum"

import type { PageServerLoad } from "./$types"

export const load: PageServerLoad = async ({ params, parent }) => {
  return loadForumCategory(parseForumRouteId(params.category), await parent())
}
