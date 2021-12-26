// neither JSDOM or happy-dom set a default language
navigator.languages = ["en"]
navigator.langauge = "en"

// jsdom/happy-dom doesn't seem to have matchMedia
// the HTMLElement interface satisfies enough of it to make it work
window.matchMedia = () => document.createElement("div")
