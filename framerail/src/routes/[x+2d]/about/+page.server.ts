import { loadInfo } from "$lib/server/load/info"

export async function load({ request, cookies, parent }) {
  return loadInfo(request, cookies, parent)
}
