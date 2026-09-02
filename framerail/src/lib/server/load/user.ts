import defaults from "$lib/defaults"

import { authGetSession } from "$lib/server/auth/getSession"
import { translate } from "$lib/server/deepwell/translate"
import { userEdit, userView } from "$lib/server/deepwell/user"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { error, redirect } from "@sveltejs/kit"
import { fail, superValidate, withFiles } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import { file, object, optional, string } from "valibot"

import type { PreloadDataAsync } from "$lib/server/deepwell/views"
import type { TranslateKeys, UserModel } from "$lib/types"
import type { Cookies, RequestEvent } from "@sveltejs/kit"

export async function loadUser(
  request: Request,
  cookies: Cookies,
  preloadData: PreloadDataAsync,
  username?: string
) {
  const { siteId } = loadSiteInfo(request.headers)
  const sessionToken = cookies.get("wikijump_token")

  const parentData = await preloadData()
  const locales = parentData.locales

  const response = await userView(siteId, locales, sessionToken, username)

  let translateKeys: TranslateKeys = {
    ...defaults.translateKeys,
    "footer-license-unless": {
      license: parentData.license_name,
      "license_url": parentData.license_url
    }
  }

  let errorStatus = null

  switch (response.type) {
    case "user_found":
      break
    case "user_missing":
      errorStatus = 404
      break
    default:
      // Unexpected response type!
      // There is an inconsistency between here / DEEPWELL
      errorStatus = 500
  }

  // If the username is not the same as the slug, redirect to the slug
  if (
    errorStatus === null &&
    username &&
    response.type === "user_found" &&
    response.data.user.slug !== username
  ) {
    redirect(308, `/-/user/${response.data.user.slug}`)
  }

  const viewData: {
    user?: Partial<UserModel & { avatar: string }>
  } = response.data ?? {}

  if (errorStatus !== null && response.type === "user_missing") {
    translateKeys = {
      ...translateKeys,
      "user-not-exist": {},
      "user-not-logged-in": {}
    }
  } else if (errorStatus === null && response.type === "user_found") {
    const isViewingAnotherUser =
      parentData.user_session?.user?.user_id !== response.data.user.user_id

    viewData.user = sanitizeUserData(response.data.user, isViewingAnotherUser)

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
      "user-profile-info.website": {},
      "user-profile-info.user-page": {},
      "user-profile-info.locales": {},
      "user-profile-info.toast": {}
    }
  }

  const internationalization = await translate(locales, translateKeys)

  const userEditForm = await superValidate(request, valibot(userEditSchema))

  if (errorStatus !== null) {
    error(errorStatus, { ...viewData, view: response.type, internationalization })
  }

  return { ...viewData, view: response.type, internationalization, userEditForm }
}

export function sanitizeUserData(
  user: UserModel,
  isViewingAnotherUser: boolean
): Partial<UserModel> {
  const baseSafeKeys: (keyof UserModel)[] = [
    "user_id",
    "user_type",
    "created_at",
    "updated_at",
    "deleted_at",
    "name",
    "slug",
    "avatar_s3_hash",
    "website",
    "user_page"
  ]
  if (isViewingAnotherUser) {
    // the whitelist for viewing other user profiles should be a subset of that for
    // viewing their own.
    return Object.fromEntries(
      baseSafeKeys.filter((key) => key in user).map((key) => [key, user[key]])
    )
  } else {
    const safeKeys: (keyof UserModel)[] = [
      ...baseSafeKeys,
      "name_changes_left",
      "last_name_change_added_at",
      "last_renamed_at",
      "email",
      "email_verified_at",
      "email_validation_info",
      "email_validation_at",
      "locales",
      "real_name",
      "gender",
      "birthday",
      "location",
      "biography"
    ]
    return Object.fromEntries(
      safeKeys.filter((key) => key in user).map((key) => [key, user[key]])
    )
  }
}

export async function userEditAction({
  request,
  cookies,
  getClientAddress
}: RequestEvent) {
  const form = await superValidate(request, valibot(userEditSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)

  const ipAddress = getClientAddress()

  try {
    const {
      name,
      realName,
      email,
      avatar,
      gender,
      birthday,
      location,
      biography,
      website,
      userPage,
      locales
    } = form.data

    const res = await userEdit(session?.user_id, ipAddress, {
      name,
      email,
      locales: locales
        ?.replaceAll("_", "-")
        .replaceAll(",", " ")
        .split(" ")
        .filter((v) => v.trim()),
      avatar,
      realName,
      gender,
      birthday,
      location,
      biography,
      website,
      userPage,
      bypassFilter: false
    })

    return withFiles({ form, res })
  } catch (error) {
    return fail(500, {
      form,
      message: error.message,
      code: error.code,
      data: error.data
    })
  }
}

export const userEditSchema = object({
  name: optional(string()),
  realName: optional(string()),
  email: optional(string()),
  avatar: optional(file()),
  gender: optional(string()),
  birthday: optional(string()),
  location: optional(string()),
  website: optional(string()),
  userPage: optional(string()),
  biography: optional(string()),
  locales: optional(string())
})
