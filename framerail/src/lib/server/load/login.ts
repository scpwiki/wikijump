import defaults from "$lib/defaults"

import { authGetSession } from "$lib/server/auth/get-session"
import { authLogin } from "$lib/server/auth/login"
import { authMfaVerify } from "$lib/server/auth/mfa"
import { translate } from "$lib/server/deepwell/translate"
import {
  clearLoginPassword,
  redactAuthActionPayload
} from "$lib/server/load/auth-form-redaction.js"
import { failForActionError } from "$lib/server/load/action-error"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { fail } from "@sveltejs/kit"
import { superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import { minLength, object, pipe, string } from "valibot"

import type { PreloadDataAsync } from "$lib/server/deepwell/views"
import type { TranslateKeys } from "$lib/types"
import type { Cookies, RequestEvent } from "@sveltejs/kit"

export async function loadLoginPage(request: Request, preloadData: PreloadDataAsync) {
  loadSiteInfo(request.headers)

  const parentData = await preloadData()
  const locales = parentData.locales

  const isLoggedIn = Boolean(parentData.user_session)

  const translateKeys: TranslateKeys = {
    ...defaults.translateKeys,

    // Page actions
    "cancel": {},
    "login": {},

    // misc
    "specifier": {},
    "password": {},
    "login.toast": {},
    "forgot-password": {},
    "remember-me": {},
    "create-account": {}
  }

  const internationalization = await translate(locales, translateKeys)

  // superform
  const loginForm = await superValidate(valibot(loginSchema))

  // Return to page for rendering
  return { isLoggedIn, internationalization, loginForm }
}

export async function loginAction({ request, getClientAddress, cookies }: RequestEvent) {
  const userAgent: string = request.headers.get("User-Agent") ?? ""
  const ipAddress: string = getClientAddress()
  const formData = await request.formData()
  const mfaSessionToken = formData.get("mfaSessionToken")
  const totpOrCode = formData.get("totpOrCode")

  if (mfaSessionToken !== null || totpOrCode !== null) {
    const form = await superValidate(formData, valibot(loginSchema))
    const submittedPassword = form.data.password

    if (
      typeof mfaSessionToken !== "string" ||
      mfaSessionToken.length === 0 ||
      typeof totpOrCode !== "string" ||
      totpOrCode.trim().length === 0
    ) {
      return fail(
        400,
        redactAuthActionPayload(
          {
            form: clearLoginPassword(form),
            message: "MFA code is required"
          },
          [submittedPassword]
        )
      )
    }

    try {
      const sessionToken = await authMfaVerify(
        mfaSessionToken,
        totpOrCode,
        ipAddress,
        userAgent
      )
      await setSessionCookie(cookies, sessionToken)

      return redactAuthActionPayload(
        {
          form: clearLoginPassword(form),
          session_token: undefined,
          needsMfa: false,
          isLoggedIn: true
        },
        [submittedPassword]
      )
    } catch (error) {
      return failForActionError(
        error,
        redactAuthActionPayload({ form: clearLoginPassword(form) }, [submittedPassword])
      )
    }
  }

  const form = await superValidate(formData, valibot(loginSchema))
  const submittedPassword = form.data.password

  if (!form.valid) {
    return fail(
      400,
      redactAuthActionPayload({ form: clearLoginPassword(form) }, [submittedPassword])
    )
  }

  try {
    const { data } = form
    const res = await authLogin(data.nameOrEmail, data.password, ipAddress, userAgent)

    if (res.session_token && !res.needs_mfa) {
      await setSessionCookie(cookies, res.session_token)
    }

    return redactAuthActionPayload(
      {
        form: clearLoginPassword(form),
        session_token: res.needs_mfa ? res.session_token : undefined,
        needsMfa: res.needs_mfa,
        isLoggedIn: !res.needs_mfa
      },
      [submittedPassword]
    )
  } catch (error) {
    return failForActionError(
      error,
      redactAuthActionPayload({ form: clearLoginPassword(form) }, [submittedPassword])
    )
  }
}

const loginSchema = object({
  nameOrEmail: pipe(string(), minLength(1)),
  password: pipe(string(), minLength(1))
})

async function setSessionCookie(cookies: Cookies, sessionToken: string) {
  const session = await authGetSession(sessionToken)
  cookies.set("wikijump_token", sessionToken, {
    path: "/",
    httpOnly: true,
    secure: true,
    sameSite: "lax",
    expires: new Date(session.expires_at)
  })
}
