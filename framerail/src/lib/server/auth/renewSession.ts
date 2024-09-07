import { client } from "$lib/server/deepwell"

export async function authRenewSession(
  sessionToken: string,
  userId: number,
  ipAddress: string,
  userAgent: string
): Promise<object> {
  return client.request("session_renew", {
    old_session_token: sessionToken,
    user_id: userId,
    ip_address: ipAddress,
    user_agent: userAgent
  })
}
