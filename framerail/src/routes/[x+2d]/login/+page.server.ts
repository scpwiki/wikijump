import { loadLoginPage } from "$lib/server/load/login.ts"

export async function load({ request, cookies }) {
  return loadLoginPage(request, cookies)
}
