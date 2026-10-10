import defaults from "$lib/defaults"

import { limitLocalePreferences } from "$lib/locales"
import { authGetSession } from "$lib/server/auth/get-session"
import { getFileByHash } from "$lib/server/deepwell/file"
import { translate } from "$lib/server/deepwell/translate"
import { userEdit, userView } from "$lib/server/deepwell/user"
import {
  failForActionError,
  failForMissingSession,
  requireActionSession
} from "$lib/server/load/action-error"
import { getRequestContext } from "$lib/server/request-context"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { error, redirect } from "@sveltejs/kit"
import { fail, superValidate, withFiles } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import { file, object, optional, string } from "valibot"

import type { PreloadData, PreloadDataAsync } from "$lib/server/deepwell/views"
import type { TranslateKeys, TranslatedKeys, UserModel } from "$lib/types"
import type { Cookies, RequestEvent } from "@sveltejs/kit"

type UserPageData = {
  user?: Partial<UserModel & { avatar: string }>
  view: string
  internationalization: TranslatedKeys
}

type PublicUserPageData = {
  site: Pick<PreloadData["site"], "slug" | "name" | "locale">
  site_file_domain: PreloadData["site_file_domain"]
  license_name: PreloadData["license_name"]
  license_url: PreloadData["license_url"]
  license_kind: PreloadData["license_kind"]
  license_html: PreloadData["license_html"]
} & UserPageData

type OwnUserPageData = PreloadData &
  UserPageData & { userEditForm: Awaited<ReturnType<typeof makeUserEditForm>> }

export function loadUser(
  request: Request,
  cookies: Cookies,
  preloadData: PreloadDataAsync,
  username: string
): Promise<PublicUserPageData>
export function loadUser(
  request: Request,
  cookies: Cookies,
  preloadData: PreloadDataAsync
): Promise<OwnUserPageData>
export async function loadUser(
  request: Request,
  cookies: Cookies,
  preloadData: PreloadDataAsync,
  username?: string
): Promise<PublicUserPageData | OwnUserPageData> {
  const { siteId } = loadSiteInfo(request.headers)
  const sessionToken = cookies.get("wikijump_token")

  let parentData = await preloadData()
  if (username && !parentData.site) {
    const { loadPreload } = await import("$lib/server/load/preload")
    parentData = await loadPreload(request, cookies)
  }
  const locales = parentData.locales

  const response = await userView(siteId, locales, sessionToken, username)
  const unsupportedImportedUser =
    response.type === "user_found" && response.data.user.user_type === "wikidot"

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
      if (unsupportedImportedUser) errorStatus = 404
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
  } = {}

  if (
    (errorStatus !== null && response.type === "user_missing") ||
    unsupportedImportedUser
  ) {
    translateKeys = {
      ...translateKeys,
      "user-not-exist": {},
      "user-not-logged-in": {}
    }
  } else if (
    errorStatus === null &&
    response.type === "user_found" &&
    response.data.user.user_type !== "wikidot"
  ) {
    const isViewingAnotherUser =
      username !== undefined ||
      parentData.user_session?.user?.user_id !== response.data.user.user_id

    const user = response.data.user
    viewData.user = sanitizeUserData(user, isViewingAnotherUser)

    // Get user avatar image
    if (user.avatar_s3_hash !== null) {
      const avatar = await getFileByHash(new Uint8Array(user.avatar_s3_hash))
      const dataurl = `data:${avatar.type};base64,${Buffer.from(
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
      "user-profile-info": {},
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
      "user-profile-info.locales": {}
    }
  }

  const internationalization = await translate(locales, translateKeys)

  if (username) {
    const pageData = {
      site: {
        slug: parentData.site.slug,
        name: parentData.site.name,
        locale: parentData.site.locale
      },
      site_file_domain: parentData.site_file_domain,
      license_name: parentData.license_name,
      license_url: parentData.license_url,
      license_kind: parentData.license_kind,
      license_html: parentData.license_html,
      ...viewData,
      view: unsupportedImportedUser ? "user_missing" : response.type,
      internationalization
    }

    if (errorStatus !== null) {
      error(errorStatus, pageData as App.Error)
    }

    return pageData
  }

  const pageData = {
    ...parentData,
    ...viewData,
    view: unsupportedImportedUser ? "user_missing" : response.type,
    internationalization
  }

  if (errorStatus !== null) {
    error(errorStatus, pageData)
  }

  const userEditForm = await makeUserEditForm(request)
  return { ...pageData, userEditForm }
}

async function makeUserEditForm(request: Request) {
  return superValidate(request, valibot(userEditSchema))
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
      "forum_signature",
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
  getClientAddress,
  locals
}: RequestEvent) {
  const form = await superValidate(request, valibot(userEditSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) return failForMissingSession({ form })

  const ipAddress = getClientAddress()

  try {
    const session = requireActionSession(await authGetSession(sessionToken))
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

    await userEdit(
      session.user_id,
      ipAddress,
      {
        name,
        email,
        locales: locales
          ? limitLocalePreferences(
              locales.replaceAll("_", "-").replaceAll(",", " ").split(" ")
            )
          : undefined,
        avatar,
        realName,
        gender,
        birthday,
        location,
        biography,
        website,
        userPage,
        bypassFilter: false
      },
      getRequestContext(locals)
    )

    return withFiles({ form })
  } catch (error) {
    return failForActionError(error, { form })
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
