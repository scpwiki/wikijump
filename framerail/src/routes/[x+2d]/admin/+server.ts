import { authGetSession } from "$lib/server/auth/getSession"
import { siteUpdate } from "$lib/server/deepwell/admin.js"

// Handling of server events from client

export async function POST(event) {
  let data = await event.request.formData()

  let sessionToken = event.cookies.get("wikijump_token")
  let ipAddr = event.getClientAddress()
  let userAgent = event.cookies.get("User-Agent")

  let session = await authGetSession(sessionToken)

  let action = data.get("action")?.toString().toLowerCase()

  let siteIdVal = data.get("site-id")?.toString()
  let siteId = siteIdVal ? parseInt(siteIdVal) : null

  let res: object = {}

  try {
    if (action === "edit") {
      /** Edit site settings. */
      let name = data.get("name")?.toString()
      let slug = data.get("slug")?.toString()
      let tagline = data.get("tagline")?.toString()
      let description = data.get("description")?.toString()
      let locale = data.get("locale")?.toString()
      let layout = data.get("layout")?.toString().trim()

      res = await siteUpdate(
        siteId,
        session?.user_id,
        name,
        slug,
        tagline,
        description,
        locale,
        layout
      )
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
