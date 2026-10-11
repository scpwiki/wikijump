/**
 * The site manager owns the reserved `_admin` route. Page-view aliases
 * must return to that route so its server-side authorization runs before
 * article content is loaded.
 */
export const isReservedAdminAliasSlug = (slug) => {
  if (typeof slug !== "string") return false

  return slug.replace(/^\/+|\/+$/gu, "").toLowerCase() === "_admin"
}
