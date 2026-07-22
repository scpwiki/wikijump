/**
 * Error route data is the authoritative shell input when present. Parent
 * layout data can remain populated during error rendering and must not be
 * mixed into the error shell.
 */
export const resolveCanonicalViewData = (errorData, pageData) =>
  errorData ?? pageData ?? null

export const resolveCanonicalViewMetadata = (errorData, pageData) => {
  const viewData = resolveCanonicalViewData(errorData, pageData)
  return {
    viewData,
    locale: viewData?.site?.locale,
    licenseName: viewData?.license_name,
    licenseUrl: viewData?.license_url,
    licenseKind: viewData?.license_kind,
    licenseHtml: viewData?.license_html,
    sourceSite: viewData?.wikidot_snapshot?.source_site
  }
}
