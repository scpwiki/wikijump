import { client } from "$lib/server/deepwell"

export async function authGetSession(sessionToken: string | undefined): Promise<object> {
  return client.request("session_get", [sessionToken ?? ""])
}
