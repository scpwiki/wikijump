/**
 * @param {{
 *   category_id: number
 *   slug: string
 *   per_page_discussion: boolean | null
 * }} category
 */
export const discussionFormValues = (category) => ({
  categoryId: category.category_id,
  state:
    category.per_page_discussion === null && category.slug !== "_default"
      ? "default"
      : category.per_page_discussion
        ? "enable"
        : "disable"
})

/** @param {{ state: "default" | "enable" | "disable" }} form */
export const discussionUpdateValue = (form) => {
  if (form.state === "default") return null
  return form.state === "enable"
}
