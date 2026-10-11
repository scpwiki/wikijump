/**
 * The public Wikidot profile action opens this permission message for
 * anonymous viewers. Keep the dialog state separate from contact mutation:
 * no relationship request is made by this guard.
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
export const showUserContactAuthenticationGuard = (popupState) => {
  popupState.current = {
    state: true,
    title: "Permission error",
    message: "Please note:",
    data: "Please create a Wikidot account and/or sign in first"
  }
}
