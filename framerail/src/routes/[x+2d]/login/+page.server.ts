import { loadLoginPage } from "$lib/server/load/login"

export async function load({ request, cookies }) {
  return loadLoginPage(request, cookies)
}
