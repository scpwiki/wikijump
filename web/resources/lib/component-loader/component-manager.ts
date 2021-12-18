/**
 * Object that maps component names to functions that asynchronously import
 * a component.
 */
// prettier-ignore
const COMPONENT_MAP = {
  "AccountControlWidget":    imp(() => import("../components/account/AccountControlWidget.svelte")),
  "ConfirmPasswordForm":     imp(() => import("../components/auth/ConfirmPasswordForm.svelte")),
  "ForgotPasswordForm":      imp(() => import("../components/auth/ForgotPasswordForm.svelte")),
  "LoginForm":               imp(() => import("../components/auth/LoginForm.svelte")),
  "RegisterForm":            imp(() => import("../components/auth/RegisterForm.svelte")),
  "ResendVerificationEmail": imp(() => import("../components/auth/ResendVerificationEmail.svelte")),
  "ResetPasswordForm":       imp(() => import("../components/auth/ResetPasswordForm.svelte")),
} as const

/** Valid component names. */
export type ComponentName = keyof typeof COMPONENT_MAP

/** Type function that returns the component type for a given component name. */
export type Component<K extends ComponentName> = ReturnType<typeof COMPONENT_MAP[K]>

/** @see {@link ComponentManager} */
class ComponentManagerInstance {
  /** Cache for already loaded components. */
  private cache = new Map<string, any>()

  /**
   * Checks if a component has already been loaded.
   *
   * @param name - The name of the component to check.
   */
  isLoaded(name: ComponentName) {
    return this.cache.has(name)
  }

  /**
   * Checks if the given name is a valid component name.
   *
   * @param name - The name to check.
   */
  isComponent(name: string): name is ComponentName {
    return COMPONENT_MAP.hasOwnProperty(name)
  }

  /**
   * Loads a component by name, returning the eventual component.
   *
   * @param name - The name of the component to load.
   */
  async load<N extends ComponentName>(name: N): Promise<Component<N>> {
    if (!this.isComponent(name)) throw new Error(`Component ${name} is not registered`)

    if (this.isLoaded(name)) return this.cache.get(name)

    const component = await COMPONENT_MAP[name]()
    this.cache.set(name, component)
    return component
  }
}

function imp<T>(importFunction: () => Promise<{ default: T }>) {
  return async () => (await importFunction()).default
}

/**
 * Manages the loading of components, using a mapping of component names to
 * functions that return the component. This is all done asynchronously.
 */
export const ComponentManager = new ComponentManagerInstance()

export default ComponentManager
