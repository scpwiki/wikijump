/**
 * @file Type declarations used in UI components.
 */

export declare global {
  /**
   * A Svelte action, used in the `use:` directive.
   *
   * Derived from sveltejs/language-tools, MIT
   */
  type SvelteAction<U extends any[] = never[], El = HTMLElement> = (
    node: El,
    ...args: U
  ) => {
    update?: (...args: U) => void
    destroy?: () => void
  } | void
}
