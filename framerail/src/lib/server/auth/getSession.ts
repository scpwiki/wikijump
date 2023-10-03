import { client } from "$lib/server/deepwell/index.ts"

export async function authGetSession(
  sessionToken: string
): Promise<object> {
  return client.request("session_get", {
    body: [sessionToken]
  })
}
