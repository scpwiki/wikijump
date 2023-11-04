import { authLogout } from "$lib/server/auth/logout"

export async function DELETE(event) {
  let sessionToken = event.cookies.get("wikijump_token")

  try {
    let res = await authLogout(sessionToken)

    event.cookies.delete("wikijump_token", {
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "lax",
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
