import { loadLogoutPage } from "$lib/server/load/logout"

export async function load({ request, cookies }) {
  return loadLogoutPage(request, cookies)
}
