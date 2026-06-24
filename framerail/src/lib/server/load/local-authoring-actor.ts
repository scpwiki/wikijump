const LOCAL_ADMIN_USER_ID = -1
const EDITABLE_LOCAL_SITE_SLUG = "scpaiueouiuiuiui"

export function resolvePageMutationUserId(
  sessionUserId: number | undefined,
  siteSlug: string
): number | undefined {
  if (sessionUserId !== undefined) {
    return sessionUserId
  }

  if (siteSlug === EDITABLE_LOCAL_SITE_SLUG) {
    return LOCAL_ADMIN_USER_ID
  }

  return undefined
}
