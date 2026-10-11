export function pageErrorStatus(type, slug) {
  if (
    type === "permissions" &&
    slug?.replace(/^\/+|\/+$/gu, "").toLowerCase() === "_admin"
  ) {
    return 401
  }
  return type === "missing" ? 404 : 403
}
