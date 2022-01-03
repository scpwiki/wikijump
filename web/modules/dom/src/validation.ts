/**
 * Checks if all of the given form inputs are valid via ordinary HTML validation.
 *
 * @param inputs - The form inputs to check. Can accept `undefined` or
 *   `null` values, which will count as "invalid" inputs.
 */
export function inputsValid(
  ...inputs: (
    | HTMLInputElement
    | HTMLSelectElement
    | HTMLTextAreaElement
    | HTMLButtonElement
    | HTMLFieldSetElement
    | undefined
    | null
  )[]
) {
  for (const input of inputs) {
    if (input instanceof HTMLFieldSetElement) continue
    if (input instanceof HTMLButtonElement) continue
    if (
      !input ||
      input.disabled ||
      !input.validity.valid ||
      (input.required && input.value.length === 0) ||
      (input as any).readOnly
    ) {
      return false
    }
  }

  return true
}
