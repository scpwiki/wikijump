/**
 * The public Wikidot profile action row opens this permission message for
 * anonymous viewers. Retained live evidence:
 *
 * - `install/local/wikidot-verification/artifacts/userinfo-target-routes-live-20260810.json`
 *   retains the anonymous `Add to contacts` and `flag user as abusive`
 *   controls and their stable numeric target user IDs.
 * - `commentary.md` 2026-10-08: activating either control while anonymous
 *   keeps the profile URL in place, opens `Permission error` / `Please
 *   note:` / `Please create a Wikidot account and/or sign in first`, and
 *   creates no contact relation and no abuse report.
 *
 * Keep the dialog state separate from any contact, report, or message
 * mutation: opening this guard never creates one, so an anonymous viewer
 * cannot produce a write by merely opening the control.
 *
 * @param {{
 *   current: {
 *     state: boolean
 *     title?: string | null
 *     message: string | null
 *     data: unknown | null
 *   }
 * }} popupState
 */
export const showUserProfileActionAuthenticationGuard = (popupState) => {
  popupState.current = {
    state: true,
    title: "Permission error",
    message: "Please note:",
    data: "Please create a Wikidot account and/or sign in first"
  }
}
