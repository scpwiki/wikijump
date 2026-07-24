/**
 * @typedef {"registered" | "members"} PageRatingPermission
 *
 * @typedef {"visible" | "anonymous"} PageRatingVisibility
 *
 * @typedef {"plus" | "plus_minus" | "stars"} PageRatingType
 *
 * @typedef {{
 *   category_id: number
 *   slug: string
 *   rating_enabled: boolean | null
 *   rating_permission: PageRatingPermission | null
 *   rating_visibility: PageRatingVisibility | null
 *   rating_type: PageRatingType | null
 * }} RatingCategory
 */

const WIKIDOT_RATING_DEFAULTS = Object.freeze({
  enabled: true,
  permission: "registered",
  visibility: "visible",
  ratingType: "plus_minus"
})

/**
 * @param {RatingCategory} category
 * @param {RatingCategory | undefined} defaultCategory
 */
export const ratingFormValues = (category, defaultCategory) => ({
  categoryId: category.category_id,
  inherit:
    category.slug !== "_default" &&
    category.rating_enabled === null &&
    category.rating_permission === null &&
    category.rating_visibility === null &&
    category.rating_type === null,
  enabled:
    category.rating_enabled ??
    defaultCategory?.rating_enabled ??
    WIKIDOT_RATING_DEFAULTS.enabled,
  permission:
    category.rating_permission ??
    defaultCategory?.rating_permission ??
    WIKIDOT_RATING_DEFAULTS.permission,
  visibility:
    category.rating_visibility ??
    defaultCategory?.rating_visibility ??
    WIKIDOT_RATING_DEFAULTS.visibility,
  ratingType:
    category.rating_type ??
    defaultCategory?.rating_type ??
    WIKIDOT_RATING_DEFAULTS.ratingType
})
