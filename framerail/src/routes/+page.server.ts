import { pageActions } from "$lib/server/load/page/page-actions"
import { loadPage } from "$lib/server/load/page/page"

export async function load({ request, cookies, locals }) {
  return loadPage(undefined, undefined, request, cookies, locals)
}

export const actions = pageActions
