import "../vendor/prism"

// Re-export a reference to Prism so that there is actually a half-decent
// way of accessing it
/**
 * Reference to the Prism syntax highlighter.
 * @namespace
 * @external
 */
export const Prism: typeof import("prismjs") = globalThis.Prism

// set prism class prefix
Prism.plugins.customClass.prefix("code-")

// HTML escape function taken from Svelte
// https://github.com/sveltejs/svelte/blob/master/src/compiler/compile/utils/stringify.ts

const escaped: Record<string, string> = {
  '"': "&quot;",
  "'": "&#39;",
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;"
}

/** Escapes a string of text so that it is safe to insert into HTML. */
function escapeHTML(src: string) {
  return src.replace(/["'&<>]/g, match => escaped[match])
}

/**
 * Highlights a string of code and returns HTML, given a specified language.
 * If the language specified isn't known by Prism, the string of code will
 * be escaped and returned with no syntax highlighting.
 *
 * @param code The string to be highlighted.
 * @param lang The language to highlight the code with.
 */
export function highlight(code: string, lang: string) {
  if (!(lang in Prism.languages)) return escapeHTML(code)
  try {
    const grammar = Prism.languages[lang]
    const html = Prism.highlight(code, grammar, lang)
    return html
  } catch {
    return escapeHTML(code)
  }
}
