import { loadUser } from "$lib/server/load/user"

export async function load({ params, request, cookies }) {
  return loadUser(null, request, cookies)
}
