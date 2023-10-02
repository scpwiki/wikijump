import { wellfetch } from "$lib/server/deepwell/index.ts"

export async function authLogout(
  sessionToken: string
): Promise<object> {
  const response = await wellfetch("/auth/logout", {
    method: "DELETE",
    headers: {
      "Content-Type": "text/plain"
    },
    body: sessionToken
  })

  if (!response.ok) {
    throw new Error("Unable to logout")
  }

  return response.ok
}
