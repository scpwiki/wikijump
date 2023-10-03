import { client } from "$lib/server/deepwell/index.ts"

export async function authLogin(
  nameOrEmail: string,
  password: string,
  ipAddress: string,
  userAgent: string
): Promise<object> {
  return client.request("login", {
    name_or_email: nameOrEmail,
    password,
    ip_address: ipAddress,
    user_agent: userAgent
  })
}
