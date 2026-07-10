/**
 * @template {{ data: { password?: string } }} T
 * @param {T} form
 * @returns {T}
 */
export const clearLoginPassword = (form) => {
  form.data.password = ""
  return form
}

/**
 * @template {{ data: { password?: string; confirmPassword?: string } }} T
 * @param {T} form
 * @returns {T}
 */
export const clearRegisterPasswords = (form) => {
  form.data.password = ""
  form.data.confirmPassword = ""
  return form
}
