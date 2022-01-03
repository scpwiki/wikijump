<!--
  @component Smart form wrapper.
  Form input elements with the `name` attribute are automatically provided to
  an `onsubmit` function.
-->
<script lang="ts">
  import { getFoci, inputsValid } from "@wikijump/dom"
  import { format as t } from "@wikijump/fluent"

  type Values = Record<
    string,
    Arrayable<string | number | boolean> | Date | FileList | null
  >

  // this is how generics work in Svelte
  // stands for <T extends ...>
  type T = $$Generic<(values: Values) => Promisable<void>>

  /**
   * Callback to fire when the form is submitted. It is provided the
   * resolved values for every named input of the form.
   */
  export let onsubmit: T = (() => {}) as any

  /**
   * Callback to fire when an error occurs during submission. It is
   * provided the error. Returning a string will set the slots error variable.
   */
  export let onerror: (error: unknown) => void | string = () => {}

  let form: HTMLFormElement

  // passed to slot

  /** True if the form is currently submitting. */
  let busy = false

  /** True if the form successfully fired. */
  let fired = false

  /** The current form error. This will be an empty string if there isn't any error. */
  let error = ""

  /** Submits the form. */
  function submit() {
    if (busy) return
    wrapper()
  }

  /**
   * When fired, the next focusable form element after the current one will
   * be focused.
   */
  function focusnext() {
    const focused = document.activeElement as HTMLElement | null
    if (!focused) return
    const children = getFoci(form)
    const index = children.indexOf(focused)
    if (index === -1) return
    const next = children[index + 1]
    if (!next) return
    next.focus()
  }

  // functions

  /** Gets all of the forms inputs. */
  function getElements() {
    return (
      Array.from(form.elements) as (
        | HTMLInputElement
        | HTMLSelectElement
        | HTMLTextAreaElement
        | HTMLButtonElement
        | HTMLFieldSetElement
      )[]
    ).filter(
      input =>
        input.name &&
        input.disabled === false &&
        input.type !== "submit" &&
        input.type !== "reset" &&
        input.type !== "button"
    )
  }

  /** Wrapper around the onsubmit callback. */
  async function wrapper(evt?: Event) {
    if (evt) evt.preventDefault()

    if (busy) return

    busy = true
    fired = false
    error = ""

    const inputs = getElements()

    if (!inputsValid(...inputs)) {
      error = t("error-form.missing-fields")
      busy = false
      return
    }

    const values: Values = {}

    for (let input of inputs) {
      const name = input.name

      if (input instanceof HTMLSelectElement) {
        const selected = Array.from(input.selectedOptions)
        values[name] = selected.length ? selected.map(option => option.value) : null
      } else if (input instanceof HTMLInputElement) {
        // prettier-ignore
        switch (input.type) {
          case "checkbox": values[name] = input.checked;       break
          case "number":   values[name] = input.valueAsNumber; break
          case "date":     values[name] = input.valueAsDate;   break
          case "file":     values[name] = input.files;         break
          default:         values[name] = input.value;         break
        }
      } else if (input instanceof HTMLTextAreaElement) {
        values[name] = input.value
      }
    }

    try {
      await onsubmit(values)
      fired = true
    } catch (err) {
      error = onerror(err) ?? ""
    } finally {
      busy = false
    }
  }
</script>

<form bind:this={form} on:submit={wrapper}>
  <slot {busy} {fired} {error} {submit} {focusnext} />
</form>
