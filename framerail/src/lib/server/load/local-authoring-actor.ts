const LOCAL_ADMIN_USER_ID = -1
const EDITABLE_LOCAL_SITE_SLUG = "scpaiueouiuiuiui"

export function resolvePageMutationUserId(
  sessionUserId: number | undefined,
  siteSlug: string,
  requestSiteId: number,
  mutationSiteId: number
): number | undefined {
  if (sessionUserId !== undefined) {
    return sessionUserId
  }

  if (siteSlug === EDITABLE_LOCAL_SITE_SLUG && mutationSiteId === requestSiteId) {
    return LOCAL_ADMIN_USER_ID
  }

  return undefined
}
