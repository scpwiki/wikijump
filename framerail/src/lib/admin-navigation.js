/**
 * @param {{
 *   category_id: number
 *   top_bar_page: string | null
 *   side_bar_page: string | null
 * }} category
 * @param {{
 *   top_bar_page: string | null
 *   side_bar_page: string | null
 * }} site
 */
export const navigationFormValues = (category, site) => ({
  categoryId: category.category_id,
  inherit: category.top_bar_page === null && category.side_bar_page === null,
  topBarPage: category.top_bar_page ?? site.top_bar_page ?? "",
  sideBarPage: category.side_bar_page ?? site.side_bar_page ?? ""
})

/**
 * @param {{
 *   inherit: boolean
 *   topBarPage: string
 *   sideBarPage: string
 * }} form
 */
export const navigationUpdateValues = (form) => ({
  topBarPage: form.inherit ? null : form.topBarPage.trim(),
  sideBarPage: form.inherit ? null : form.sideBarPage.trim()
})
