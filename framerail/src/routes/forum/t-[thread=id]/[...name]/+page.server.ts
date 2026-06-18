import { loadForumThread, parseForumRouteId } from "$lib/server/load/forum"

import type { PageServerLoad } from "./$types"

export const load: PageServerLoad = async ({ params, parent }) => {
  return loadForumThread(parseForumRouteId(params.thread), await parent())
}
