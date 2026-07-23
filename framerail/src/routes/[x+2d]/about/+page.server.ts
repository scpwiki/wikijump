import { loadInfo } from "$lib/server/load/info"

export async function load({ parent }) {
  return loadInfo(parent)
}
