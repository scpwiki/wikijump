import { client } from "$lib/server/deepwell"

export async function authMfaVerify(
  sessionToken: string,
  totpOrCode: string,
  ipAddress: string,
  userAgent: string
): Promise<string> {
  return client.request("mfa_verify", {
    session_token: sessionToken,
    totp_or_code: totpOrCode,
    ip_address: ipAddress,
    user_agent: userAgent
  })
}
