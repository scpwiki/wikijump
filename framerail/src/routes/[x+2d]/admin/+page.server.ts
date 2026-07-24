import {
  adminAction,
  discussionAction,
  forumNestingAction,
  siteIconsAction,
  licenseAction,
  loadAdminPage,
  navigationAction,
  ratingAction,
  templateAction
} from "$lib/server/load/admin"

export async function load({ request, cookies, parent }) {
  return loadAdminPage(request, cookies, parent)
}

export const actions = {
  site: adminAction,
  discussion: discussionAction,
  forumNesting: forumNestingAction,
  siteIcons: siteIconsAction,
  navigation: navigationAction,
  license: licenseAction,
  rating: ratingAction,
  template: templateAction
}
