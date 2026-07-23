import { loadLoginPage, loginAction } from "$lib/server/load/login"

export async function load({ request, parent }) {
  return loadLoginPage(request, parent)
}

export const actions = { default: loginAction }
