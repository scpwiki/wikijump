import { pageActions } from "$lib/server/load/page/page-actions"
import { loadPage } from "$lib/server/load/page/page"
import { isReservedAdminAliasSlug } from "$lib/server/reserved-admin-alias.js"
import { redirect } from "@sveltejs/kit"

export async function load({ params, request, cookies, locals }) {
  if (isReservedAdminAliasSlug(params.slug)) {
    redirect(303, "/_admin")
  }

  return loadPage(params.slug, params.extra, request, cookies, locals)
}

export const actions = pageActions
