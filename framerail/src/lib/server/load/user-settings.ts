import { authGetSession } from "$lib/server/auth/get-session"
import { translate } from "$lib/server/deepwell/translate"
import { userEdit } from "$lib/server/deepwell/user"
import {
  failForActionError,
  failForMissingSession,
  requireActionSession
} from "$lib/server/load/action-error"
import { getRequestContext } from "$lib/server/request-context"
import { parseUserLocalePreferences } from "$lib/user-settings.js"
import { uniqueLocales } from "$lib/locales.js"
import {
  USER_INTERFACE_LOCALES,
  isSupportedUserInterfaceLocale
} from "$lib/user-interface-locales"
import { fail, redirect } from "@sveltejs/kit"
import { superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import { array, maxLength, minLength, object, pipe, string } from "valibot"

import type { PreloadDataAsync } from "$lib/server/deepwell/views"
import type { RequestEvent } from "@sveltejs/kit"

export async function loadUserSettings(parent: PreloadDataAsync) {
  const parentData = await parent()
  if (!parentData.user_session?.user.user_id) {
    redirect(303, "/-/login")
  }

  const savedLocales = uniqueLocales(parentData.user_session.user.locales ?? ["en"])
  const locales = savedLocales.length > 0 ? savedLocales : ["en"]
  const userInterfaceLocales = [
    ...USER_INTERFACE_LOCALES,
    ...locales
      .filter((locale) => !isSupportedUserInterfaceLocale(locale))
      .map((value) => ({ value, label: `${value} (current)` }))
  ]
  const displaySettingsForm = await superValidate(
    {
      locales,
      signature: parentData.user_session.user.forum_signature ?? ""
    },
    valibot(userDisplaySettingsSchema)
  )
  const internationalization = await translate(
    parentData.locales,
    {
      settings: {},
      save: {},
      cancel: {},
      "user-profile-info.locales": {}
    },
    []
  )

  return {
    ...parentData,
    displaySettingsForm,
    internationalization,
    userInterfaceLocales
  }
}

export async function userDisplaySettingsAction({
  request,
  cookies,
  getClientAddress,
  locals
}: RequestEvent) {
  const form = await superValidate(request, valibot(userDisplaySettingsSchema))
  if (!form.valid) {
    return fail(400, { form })
  }
  const signature = form.data.signature.replace(/\r\n?/g, "\n")
  if (signature.split("\n").length > 4) {
    return fail(400, { form, message: "Forum signatures are limited to four lines." })
  }

  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) return failForMissingSession({ form })

  try {
    const session = requireActionSession(await authGetSession(sessionToken))
    const locales = parseUserLocalePreferences(form.data.locales.join(" "))
    if (locales.length === 0) {
      return fail(400, { form, message: "At least one display language is required." })
    }
    form.data.locales = locales

    await userEdit(
      session.user_id,
      getClientAddress(),
      { locales, forumSignature: signature || null },
      getRequestContext(locals)
    )
    return { form }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const userDisplaySettingsSchema = object({
  locales: pipe(
    array(string()),
    minLength(1, "At least one display language is required.")
  ),
  signature: pipe(
    string(),
    maxLength(400, "Forum signatures are limited to 400 characters.")
  )
})
