import defaults from "$lib/defaults"

import { translate } from "$lib/server/deepwell/translate"
import { userCreate } from "$lib/server/deepwell/user"
import {
  clearRegisterPasswords,
  redactAuthActionPayload
} from "$lib/server/load/auth-form-redaction.js"
import { failForActionError } from "$lib/server/load/action-error"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { fail } from "@sveltejs/kit"
import { superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import {
  array,
  email,
  forward,
  minLength,
  object,
  optional,
  partialCheck,
  pipe,
  string
} from "valibot"

import type { PreloadDataAsync } from "$lib/server/deepwell/views"
import { UserType, type TranslateKeys } from "$lib/types"
import type { RequestEvent } from "@sveltejs/kit"

export async function loadRegisterPage(request: Request, preloadData: PreloadDataAsync) {
  loadSiteInfo(request.headers)

  const parentData = await preloadData()
  const locales = parentData.locales

  const isLoggedIn = Boolean(parentData.user_session)

  const translateKeys: TranslateKeys = {
    ...defaults.translateKeys,

    // Page actions
    "cancel": {},
    "register": {},

    // misc
    "username": {},
    "username.placeholder": {},
    "username.info": {},
    "email": {},
    "email.placeholder": {},
    "email.info": {},
    "password": {},
    "password.placeholder": {},
    "confirm-password": {},
    "register.toast": {},
    "create-account": {},

    // errors
    "error-form.password-mismatch": {}
  }

  const internationalization = await translate(locales, translateKeys)

  // superform
  const registerForm = await superValidate(valibot(registerSchema))

  // Return to page for rendering
  return { isLoggedIn, internationalization, registerForm }
}

export async function registerAction({ request, getClientAddress }: RequestEvent) {
  const form = await superValidate(request, valibot(registerSchema))
  const submittedPasswords = [form.data.password, form.data.confirmPassword]

  if (!form.valid) {
    return fail(
      400,
      redactAuthActionPayload({ form: clearRegisterPasswords(form) }, submittedPasswords)
    )
  }

  const ipAddress = getClientAddress()
  const { data } = form

  try {
    const res = await userCreate({
      userType: UserType.Regular,
      name: data.username,
      email: data.email,
      locales: data.locale,
      password: data.password,
      ipAddress
    })

    return redactAuthActionPayload(
      { form: clearRegisterPasswords(form), res, isRegistered: true },
      submittedPasswords
    )
  } catch (error) {
    return failForActionError(
      error,
      redactAuthActionPayload({ form: clearRegisterPasswords(form) }, submittedPasswords)
    )
  }
}

const registerSchema = pipe(
  object({
    username: pipe(string(), minLength(1)),
    email: pipe(string(), email(), minLength(1)),
    password: pipe(string(), minLength(1)),
    confirmPassword: pipe(string(), minLength(1)),
    locale: pipe(optional(array(string()), ["en"]), minLength(1))
  }),
  forward(
    partialCheck(
      [["password"], ["confirmPassword"]],
      ({ password, confirmPassword }) => password === confirmPassword
    ),
    ["confirmPassword"]
  )
)
