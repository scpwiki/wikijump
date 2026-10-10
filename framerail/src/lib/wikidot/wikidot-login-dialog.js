/**
 * Helpers for the in-place Sign in surface on imported Wikidot-layout
 * pages.
 *
 * The surface posts the native `/-/login` form action; credential
 * validation, MFA, and session creation stay server-authoritative in that
 * action.
 */

export const WIKIDOT_LOGIN_DIALOG_LABELS = Object.freeze({
  title: "Sign in",
  specifier: "Email or Username",
  password: "Password",
  submit: "Login",
  cancel: "Cancel",
  failed: "Sign in failed. Check your details and try again.",
  mfaRequired:
    "Two-step verification is required. Continue on the sign-in page to enter your code.",
  signInPage: "Open the sign-in page",
  unavailable: "Sign in is unavailable right now. Try again later."
})

/**
 * Map a SvelteKit enhanced-form result from the native `/-/login` action
 * to the dialog outcome. The MFA session token is intentionally ignored
 * here: the in-place surface does not hold it, so MFA continues on the
 * native page.
 *
 * @param {{ type?: string; data?: unknown } | null | undefined} result
 * @returns {{ kind: "success" }
 *   | { kind: "mfa" }
 *   | { kind: "failure"; message: string | null }
 *   | { kind: "error" }}
 */
export const interpretLoginActionResult = (result) => {
  if (result?.type === "success") {
    const data = /** @type {{ needsMfa?: unknown; isLoggedIn?: unknown }} */ (
      result.data ?? {}
    )
    if (data.needsMfa === true) return { kind: "mfa" }
    if (data.isLoggedIn === true) return { kind: "success" }
    return { kind: "failure", message: null }
  }

  if (result?.type === "failure") {
    const data = /** @type {{ message?: unknown }} */ (result.data ?? {})
    return {
      kind: "failure",
      message: typeof data.message === "string" ? data.message : null
    }
  }

  return { kind: "error" }
}

/**
 * Only a plain primary-button activation opens the in-place surface.
 * Modified clicks, middle clicks, and other non-primary activations keep
 * the native link behaviour.
 *
 * @param {{
 *   button: number
 *   metaKey: boolean
 *   ctrlKey: boolean
 *   shiftKey: boolean
 *   altKey: boolean
 * }} event
 */
export const isPlainPrimaryActivation = (event) =>
  event.button === 0 &&
  !event.metaKey &&
  !event.ctrlKey &&
  !event.shiftKey &&
  !event.altKey
