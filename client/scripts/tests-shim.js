const { GlobalRegistrator } = require("@happy-dom/global-registrator")

GlobalRegistrator.register()

// this is a filthy hack
// but this _should_ satisfy enough of the interface not to break things
// Element implements a bunch of stuff matchMedia does, so that's our hack
globalThis.matchMedia = str => {
  const element = new Element()
  element.matches = true
  return element
}
