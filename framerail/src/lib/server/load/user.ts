import defaults from "$lib/defaults"
import { parseAcceptLangHeader } from "$lib/locales"
import { getFileByHash } from "$lib/server/deepwell/getFile"
import { translate } from "$lib/server/deepwell/translate"
import { userView } from "$lib/server/deepwell/user"
import type { TranslateKeys } from "$lib/types"
import { error, redirect } from "@sveltejs/kit"

export async function loadUser(username?: string, request, cookies) {
  const url = new URL(request.url)
  const domain = url.hostname
  const sessionToken = cookies.get("wikijump_token")
  let locales = parseAcceptLangHeader(request)

  if (!locales.includes(defaults.fallbackLocale)) locales.push(defaults.fallbackLocale)

  const response = await userView(domain, sessionToken, locales, username)

  let translateKeys: TranslateKeys = {
    ...defaults.translateKeys
  }

  const viewData = response.data
  viewData.view = response.type

  let errorStatus = null

  switch (response.type) {
    case "user_found":
      break
    case "user_missing":
      viewData.user = null
      errorStatus = 404
      break
    case "site_missing":
      errorStatus = 404
  }

  if (errorStatus === null && username && viewData.user.slug !== username) {
    throw redirect(308, `/-/user/${viewData.user.slug}`)
  }

  if (errorStatus !== null) {
    translateKeys = {
      ...translateKeys,
      "user-not-exist": {}
    }
  } else {
    // Remove sensitive information
    let sensitiveKeys = ["password", "multi_factor_secret", "multi_factor_recovery_codes"]
    if (viewData.user_session?.user?.user_id !== viewData.user.user_id) {
      // Currently viewing another user's profile
      sensitiveKeys = [...sensitiveKeys, "email", "email_is_alias", "email_verified_at"]
    }
    for (let i = 0; i < sensitiveKeys.length; i++) {
      delete viewData.user[sensitiveKeys[i]]
    }

    // Get user avatar image
    if (viewData.user.avatar_s3_hash !== null) {
      let avatar = await getFileByHash(new Uint8Array(viewData.user.avatar_s3_hash))
      let dataurl = `data:${avatar.type};base64,${Buffer.from(
        await avatar.arrayBuffer()
      ).toString("base64")}`
      viewData.user.avatar = dataurl
    }

    translateKeys = {
      ...translateKeys,

      // Edit actions
      "edit": {},
      "save": {},
      "cancel": {},

      // User profile attributes
      "avatar": {},
      "user-profile-info.name": {},
      "user-profile-info.real-name": {},
      "user-profile-info.email": {},
      "user-profile-info.avatar": {},
      "user-profile-info.gender": {},
      "user-profile-info.birthday": {},
      "user-profile-info.location": {},
      "user-profile-info.biography": {},
      "user-profile-info.user-page": {},
      "user-profile-info.locales": {}
    }
  }

  const translated = await translate(locales, translateKeys)

  viewData.internationalization = translated

  if (errorStatus !== null) {
    throw error(errorStatus, viewData)
  }

  return viewData
}
