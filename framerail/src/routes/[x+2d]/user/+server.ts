import { authGetSession } from "$lib/server/auth/getSession"
import { userEdit } from "$lib/server/deepwell/user"

export async function POST(event) {
  let data = await event.request.formData()
  let sessionToken = event.cookies.get("wikijump_token")

  try {
    let session = await authGetSession(sessionToken)

    let name = data.get("name")?.toString().trim()
    let email = data.get("email")?.toString().trim()
    let realName = data.get("real-name")?.toString().trim()
    let gender = data.get("gender")?.toString().trim()
    let birthday = data.get("birthday")?.toString().trim()
    let location = data.get("location")?.toString().trim()
    let biography = data.get("biography")?.toString().trim()
    let userPage = data.get("user-page")?.toString().trim()
    let locales = data.get("locales")?.toString().trim()

    let body: Record<string, any> = {
      name,
      email,
      real_name: realName,
      birthday,
      gender,
      location,
      biography,
      user_page: userPage,
      locales: locales?.split(" ").filter((v) => v.trim())
    }

    let res = await userEdit(session?.user_id, body)

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
