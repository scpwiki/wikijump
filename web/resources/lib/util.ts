/**
 * Checks if all of the given form inputs are valid.
 *
 * @param inputs - The form inputs to check.
 */
export function inputsValid(...inputs: (HTMLInputElement | undefined | null)[]) {
  for (const input of inputs) {
    if (
      !input ||
      !input.validity.valid ||
      input.value.length === 0 ||
      input.disabled ||
      input.readOnly
    ) {
      return false
    }
  }

  return true
}
