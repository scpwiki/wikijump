/**
 * @param {Record<string, unknown>} parentData
 * @param {Record<string, unknown>} viewData
 * @param {unknown} forms
 * @returns {Record<string, unknown>}
 */
export const buildPageLoadData = (parentData, viewData, forms) => {
  return {
    ...parentData,
    ...viewData,
    forms
  }
}
