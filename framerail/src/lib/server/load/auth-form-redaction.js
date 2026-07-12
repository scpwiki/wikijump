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

/**
 * Remove submitted credentials from an action payload immediately before
 * it is returned to SvelteKit for serialization. Unrelated tokens,
 * including the MFA continuation token, are preserved.
 *
 * @template T
 * @param {T} payload
 * @param {string[]} secrets
 * @returns {T}
 */
export const redactAuthActionPayload = (payload, secrets) => {
  const submitted = secrets.filter((secret) => secret.length > 0)
  const visited = new WeakSet()

  /** @param {unknown} value */
  const redact = (value) => {
    if (typeof value === "string") {
      return submitted.reduce(
        (redacted, secret) => redacted.replaceAll(secret, ""),
        value
      )
    }
    if (value === null || typeof value !== "object" || visited.has(value)) {
      return value
    }

    visited.add(value)
    for (const [key, nested] of Object.entries(value)) {
      value[key] = key === "password" || key === "confirmPassword" ? "" : redact(nested)
    }
    return value
  }

  return /** @type {T} */ (redact(payload))
}
