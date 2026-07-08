import { loadPage } from "$lib/server/load/page"
import { actions as pageActions } from "./[slug]/[...extra]/+page.server"

export async function load({ request, cookies }) {
  return loadPage(undefined, undefined, request, cookies)
}

export const actions = pageActions
