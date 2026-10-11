/**
 * Audited external asset origins for the Framerail CSP `style-src` and
 * `font-src` directives.
 *
 * Imported Wikidot pages routinely `@import` stylesheets and `@font-face`
 * font files that live on third-party origins, and Wikidot serves those
 * assets from the origin the author wrote. Reproducing that behavior means
 * the browser must be allowed to fetch them, so the origins an imported
 * page legitimately depends on have to be named in the policy in advance.
 *
 * This registry is the closed, single source of truth for those origins.
 * Two rules make it a real boundary rather than a growing hole:
 *
 * 1. Every entry names the retained repository-owned evidence that authorizes
 *    it. An entry is a repository decision, never something a rendered
 *    page can extend on its own: an arbitrary author-supplied `@import`
 *    URL has to fail closed (blocked by the policy) instead of silently
 *    becoming a global exception for every site.
 * 2. An entry may only grant `styles` and/or `fonts`. No audited asset origin
 *    is ever added to `script-src`, `connect-src`, `frame-src` or any
 *    other directive, so an allowlisted CSS/font origin cannot become an
 *    executable or data-exfiltration dependency of a rendered page.
 *
 * See `install/local/wikidot-verification/README.md` for the external
 * acquisition rules that govern refreshing the evidence for an entry, and
 * `framerail/tests/wikidot-csp-audited-origins.test.js` for the invariants
 * this registry is held to.
 */

/**
 * @typedef {object} AuditedExternalAssetOrigin
 * @property {string} origin Exact CSP host-source, always an
 *   `https://<host>` origin. Never a scheme-wide source such as `https:`
 *   and never a bare wildcard host.
 * @property {boolean} styles Whether the origin is allowlisted in
 *   `style-src`.
 * @property {boolean} fonts Whether the origin is allowlisted in
 *   `font-src`.
 * @property {string} evidence Where the retained observation of this
 *   origin serving a rendered page's CSS or font asset lives.
 */

/** @type {readonly AuditedExternalAssetOrigin[]} */
export const AUDITED_EXTERNAL_ASSET_ORIGINS = Object.freeze([
  {
    origin: "https://*.wdfiles.com",
    styles: true,
    fonts: true,
    evidence:
      "Wikidot's own file host. Serves imported `local--files` stylesheet and font attachments, e.g. `component:theme`'s `font-bauhaus.css` in the retained shared-corpus copies of `component:classified-decoration-base`."
  },
  {
    origin: "https://cdn.scpwiki.com",
    styles: true,
    fonts: true,
    evidence:
      "SCP Wiki CDN, cited by the retained Theme Lab port manifests (e.g. `install/local/theme-lab/ports/basalt/manifest.json`)."
  },
  {
    origin: "https://cdn.jsdelivr.net",
    styles: true,
    fonts: true,
    evidence:
      "jsDelivr CDN, cited by the retained Theme Lab port manifests (e.g. `install/local/theme-lab/ports/cosmonaut/manifest.json`)."
  },
  {
    origin: "https://d3g0gp89917ko0.cloudfront.net",
    styles: true,
    fonts: true,
    evidence:
      "Legacy SCP Wiki CloudFront distribution, cited by the retained Theme Lab port manifests (e.g. `install/local/theme-lab/ports/site/manifest.json`)."
  },
  {
    origin: "https://fonts.bunny.net",
    styles: true,
    fonts: true,
    evidence:
      "Bunny Fonts, cited by the retained Theme Lab port manifests (e.g. `install/local/theme-lab/ports/al-slop/manifest.json`)."
  },
  {
    origin: "https://fonts.googleapis.com",
    styles: true,
    fonts: false,
    evidence:
      "Google Fonts stylesheet host. Still loading with HTTP 200 on both origins for `component:classified-decoration-base`'s `Noto Serif KR` import (the #2213 control), and cited by the Theme Lab port manifests (e.g. `install/local/theme-lab/ports/foxtrot/manifest.json`)."
  },
  {
    origin: "https://fonts.gstatic.com",
    styles: false,
    fonts: true,
    evidence: "Google Fonts font-file host for the stylesheets that Google Fonts serves."
  },
  {
    origin: "https://maxcdn.bootstrapcdn.com",
    styles: true,
    fonts: true,
    evidence:
      "Legacy Bootstrap CDN, cited by the retained Theme Lab port manifests (e.g. `install/local/theme-lab/ports/site/manifest.json`)."
  },
  {
    origin: "https://nu-scptheme.github.io",
    styles: true,
    fonts: true,
    evidence:
      "Nu SCP Theme GitHub Pages host for the Black Highlighter component, cited by the retained Theme Lab port manifests (e.g. `install/local/theme-lab/ports/hansarp/manifest.json`)."
  },
  {
    origin: "https://rsms.me",
    styles: true,
    fonts: true,
    evidence:
      "Rasmus Andersson's font host (Inter), cited by the retained Theme Lab port manifests (e.g. `install/local/theme-lab/ports/foxtrot/manifest.json`)."
  },
  {
    origin: "https://scp-wiki-cdn.nyc3.cdn.digitaloceanspaces.com",
    styles: true,
    fonts: true,
    evidence:
      "SCP Wiki DigitalOcean Spaces CDN, cited by the retained Theme Lab port assets (e.g. `install/local/theme-lab/ports/basalt/assets.json`)."
  },
  {
    origin: "https://indestructibletype-fonthosting.github.io",
    styles: true,
    fonts: true,
    evidence:
      "Indestructible Type's font-hosting GitHub Pages site. Both retained shared-corpus copies of `component:classified-decoration-base` `@import` `https://indestructibletype-fonthosting.github.io/renner.css` (EN capture b498634ff254edb806041276dbd088d8c9ed415d7fac1f05ff3dada1c21501d9, JP capture d0230d2f49ab3776d8d503367e06ba5d168a5fadcca7c8036af2331b5a5112e5), and current Wikidot loads both that stylesheet and its WOFF2 with HTTP 200 while Wikijump's `style-src` blocked them (issue #2213)."
  }
])

/**
 * @param {"styles" | "fonts"} directive
 * @returns {string[]} Origins allowlisted for the given CSP directive.
 */
export const auditedExternalAssetOrigins = (directive) =>
  AUDITED_EXTERNAL_ASSET_ORIGINS.filter((entry) => entry[directive]).map(
    (entry) => entry.origin
  )
