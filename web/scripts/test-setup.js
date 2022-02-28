// jsdom/happy-dom doesn't seem to have matchMedia
// the HTMLElement interface satisfies enough of it to make it work
window.matchMedia = () => document.createElement("div")
