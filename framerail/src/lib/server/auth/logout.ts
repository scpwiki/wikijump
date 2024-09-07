import { client } from "$lib/server/deepwell"

export async function authLogout(sessionToken: string): Promise<object> {
  return client.request("logout", [sessionToken])
}
