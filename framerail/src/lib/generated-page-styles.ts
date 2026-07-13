const escapeStyleRawText = (css: string) => css.replaceAll("<", "\\3C ")

export function buildGeneratedPageStylesHead(styles: readonly string[]): string {
  return styles
    .map(
      (css, index) =>
        `<style type="text/css" data-wikijump-generated-css="${index}">${escapeStyleRawText(css)}</style>`
    )
    .join("")
}
