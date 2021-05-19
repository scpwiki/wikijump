// organize-imports-ignore
import * as i18n from "svelte-i18n"

// register languages here
i18n.register("en", () => import("../../../../locales/en.yaml"))

// init the i18n system
i18n.init({
  fallbackLocale: "en",
  initialLocale: i18n.getLocaleFromNavigator()
})

// reexport library so that it does not need to be added as a dependency
// in every other package

// observables
export { date, json, number, t, time } from "svelte-i18n"

// low-level formatters
export {
  getDateFormatter,
  getMessageFormatter,
  getTimeFormatter,
  getNumberFormatter
} from "svelte-i18n"

// library itself in-case the above exports aren't enough
export { i18n }
