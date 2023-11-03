import { client } from "$lib/server/deepwell/index.ts"

export async function authLogout(sessionToken: string): Promise<object> {
  return client.request("logout", [sessionToken])
}
