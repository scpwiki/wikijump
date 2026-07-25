import { loadLogoutPage, logoutAction } from "$lib/server/load/logout"

export async function load({ request, parent }) {
  return loadLogoutPage(request, parent)
}

export const actions = { logout: logoutAction }
