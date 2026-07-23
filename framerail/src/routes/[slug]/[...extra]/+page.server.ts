import { pageActions } from "$lib/server/load/page/page-actions"
import { loadPage } from "$lib/server/load/page/page"

export async function load({ params, request, cookies, locals }) {
  return loadPage(params.slug, params.extra, request, cookies, locals)
}

export const actions = pageActions
