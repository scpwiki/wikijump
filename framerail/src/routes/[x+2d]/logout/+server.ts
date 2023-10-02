import { authLogout } from "$lib/server/auth/logout"

export async function DELETE(event) {
  let sessionToken = event.cookies.get("wikijump_token")

  let res = await authLogout(sessionToken)

  event.cookies.set("wikijump_token", "", {
    path: "/",
    maxAge: 0
  })

  return new Response(JSON.stringify(res))
}