import { authLogout } from "$lib/server/auth/logout"

export async function DELETE(event) {
  let sessionToken = event.cookies.get("wikijump_token")

  try {
    let res = await authLogout(sessionToken)

    event.cookies.set("wikijump_token", "", {
      path: "/",
      maxAge: 0
    })

    return new Response(JSON.stringify(res))
  } catch (error) {
    return new Response(
      JSON.stringify({
        message: error.message,
        code: error.code,
        data: error.data
      })
    )
  }
}
