import { loadAdminPage } from "$lib/server/load/admin"

export async function load({ request, cookies }) {
  return loadAdminPage(request, cookies)
}
