/**
 * @template TParentData
 * @template TViewData
 * @template TForms
 * @param {TParentData} parentData
 * @param {TViewData} viewData
 * @param {TForms} forms
 * @returns {TParentData & TViewData & { forms: TForms }}
 */
export const buildPageLoadData = (parentData, viewData, forms) => {
  return /** @type {TParentData & TViewData & { forms: TForms }} */ ({
    ...parentData,
    ...viewData,
    forms
  })
}
