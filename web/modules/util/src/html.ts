// so we can load this module in workers:
let domParser: DOMParser
try {
  domParser = new DOMParser()
} catch {}

/** Takes a string of HTML and creates a {@link DocumentFragment}. */
export function toFragment(html: string) {
  const parsed = domParser.parseFromString(html, "text/html")
  const fragment = document.createDocumentFragment()
  fragment.append(...Array.from(parsed.body.children))
  return fragment
}

/**
 * **DOES NOT ESCAPE INPUT**
 *
 * Template string tag that creates a {@link DocumentFragment}.
 */
export function html(strings: TemplateStringsArray, ...subs: (string | string[])[]) {
  const src = strings.raw.reduce((prev, cur, idx) => {
    let sub = subs[idx - 1]
    if (Array.isArray(sub)) sub = sub.join("")
    return prev + sub + cur
  })
  return toFragment(src)
}

/**
 * **DOES NOT ESCAPE INPUT**
 *
 * Template string tag for creating a CSS stylesheet.
 */
export function css(strings: TemplateStringsArray, ...subs: (string | string[])[]) {
  const src = strings.raw.reduce((prev, cur, idx) => {
    let sub = subs[idx - 1]
    if (Array.isArray(sub)) sub = sub.join("")
    return prev + sub + cur
  })
  const style = document.createElement("style")
  style.textContent = src
  return style
}
