import { loadPage } from "$lib/server/load/page.ts"

export async function load({ params, request, cookies }) {
  return loadPage(params.slug, params.extra, request, cookies)
}
