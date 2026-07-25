import { loadRegisterPage, registerAction } from "$lib/server/load/register"

export async function load({ request, parent }) {
  return loadRegisterPage(request, parent)
}

export const actions = { default: registerAction }
