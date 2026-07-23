import { client } from "$lib/server/deepwell"

export async function authGetSession(sessionToken: string | undefined): Promise<{
  session_token: string
  user_id: number
  created_at: string
  expires_at: string
  ip_address: string
  user_agent: string
  restricted: boolean
}> {
  return client.request("session_get", [sessionToken ?? ""])
}
