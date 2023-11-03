import { authLogin } from "$lib/server/auth/login"

export async function POST(event) {
  let data = await event.request.formData()

  let userAgent = event.request.headers.get("User-Agent")
  let ipAddress = event.getClientAddress()

  let nameOrEmail = data.get("name-or-email")?.toString()
  let password = data.get("password")?.toString()

  let res = await authLogin(nameOrEmail, password, ipAddress, userAgent)

  event.cookies.set("wikijump_token", res.session_token, {
    path: "/"
  })

  return new Response(JSON.stringify(res))
}
