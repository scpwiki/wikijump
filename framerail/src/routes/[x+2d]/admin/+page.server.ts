import {
  adminAction,
  discussionAction,
  forumNestingAction,
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
  navigation: navigationAction,
  license: licenseAction,
  rating: ratingAction,
  template: templateAction
}
