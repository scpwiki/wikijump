import { wellfetch } from "$lib/server/deepwell/index.ts"

export async function authLogin(
  nameOrEmail: string,
  password: string,
  ipAddress: string,
  userAgent: string
): Promise<object> {
  const response = await wellfetch("/auth/login", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      nameOrEmail,
      password,
      ipAddress,
      userAgent
    })
  })

  if (!response.ok) {
    throw new Error("Unable to login")
  }

  return response.json()
}
