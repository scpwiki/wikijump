import { loadPage } from "$lib/server/load/page"
import { actions as pageActions } from "./[slug]/[...extra]/+page.server"

export async function load({ request, cookies, locals }) {
  return loadPage(undefined, undefined, request, cookies, locals)
}

export const actions = pageActions
