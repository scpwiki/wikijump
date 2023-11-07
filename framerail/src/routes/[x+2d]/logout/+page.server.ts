import { loadLogoutPage } from "$lib/server/load/logout.ts"

export async function load({ request, cookies }) {
  return loadLogoutPage(request, cookies)
}
