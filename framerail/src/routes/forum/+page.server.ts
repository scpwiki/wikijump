import { loadForumIndex } from "$lib/server/load/forum"

import type { PageServerLoad } from "./$types"

export const load: PageServerLoad = async ({ parent }) => {
  return loadForumIndex(await parent())
}
