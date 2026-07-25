import { parseDeploymentEnvironment } from "../deployment-environment.js"

const LOCAL_ADMIN_USER_ID = -1
const EDITABLE_LOCAL_SITE_SLUG = "scpaiueouiuiuiui"

// This local-only exception is intentionally limited to the page edit, page delete, file restore, and file rollback actions that support the editable local mirror. Do not extend it to sibling mutations without equivalent deployment, host, and site binding plus local-parity evidence.
export function resolvePageMutationUserId(
  sessionUserId: number | undefined,
  siteSlug: string,
  requestSiteId: number,
  mutationSiteId: number,
  deploymentEnvironment = parseDeploymentEnvironment()
): number | undefined {
  if (sessionUserId !== undefined) {
    return sessionUserId
  }

  if (
    deploymentEnvironment === "local" &&
    siteSlug === EDITABLE_LOCAL_SITE_SLUG &&
    mutationSiteId === requestSiteId
  ) {
    return LOCAL_ADMIN_USER_ID
  }

  return undefined
}
