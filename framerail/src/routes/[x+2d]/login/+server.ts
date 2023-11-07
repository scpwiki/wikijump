import { authLogin } from "$lib/server/auth/login"

export async function POST(event) {
  let data = await event.request.formData()

  let userAgent = event.request.headers.get("User-Agent")
  let ipAddress = event.getClientAddress()

  let nameOrEmail = data.get("name-or-email")?.toString()
  let password = data.get("password")?.toString()

  try {
    let res = await authLogin(nameOrEmail, password, ipAddress, userAgent)

    if (res.session_token) {
      event.cookies.set("wikijump_token", res.session_token, {
        path: "/",
        httpOnly: true,
        secure: true,
        sameSite: "lax"
        // TODO made deepwell return the cookie expiration for setting maxAge
        // maxAge: someValue
      })
    }

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
