import { loadPage } from "$lib/server/load/page.ts"

export async function load({ request, cookies }) {
  return loadPage(null, null, request, cookies)
}
