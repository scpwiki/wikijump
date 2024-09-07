import { loadUser } from "$lib/server/load/user"

export async function load({ params, request, cookies }) {
  return loadUser(params.slug, request, cookies)
}
