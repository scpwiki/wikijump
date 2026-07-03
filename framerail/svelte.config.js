import adapter from "@sveltejs/adapter-node"
import { statSync } from "fs"
import { dirname, resolve } from "path"
import { sveltePreprocess } from "svelte-preprocess"
import { fileURLToPath } from "url"

// The former only works on node 20.11+
const __dirname = import.meta.dirname ?? dirname(fileURLToPath(import.meta.url))

function resolveAssets() {
  try {
    let globalAssets = statSync(resolve(__dirname, "../assets"))
    if (globalAssets.isDirectory()) return resolve(__dirname, "../assets")
    else return resolve(__dirname, "src/assets")
  } catch (error) {
    return resolve(__dirname, "src/assets")
  }
}

/**
 * @typedef {NonNullable<
 *   NonNullable<
 *     NonNullable<import("@sveltejs/kit").Config["kit"]>["csp"]
 *   >["directives"]
 * >} CspDirectives
 */
/** @typedef {NonNullable<CspDirectives["img-src"]>} CspSources */

/** @type {CspSources} */
const LOCAL_FILE_IMAGE_SOURCES = ["https://*.wjfiles.localhost"]
/** @type {CspSources} */
const LOCAL_FILE_STYLE_SOURCES = ["https://*.wjfiles.localhost"]
/** @type {CspSources} */
const WIKIDOT_LEGACY_IMAGE_SOURCES = ["https://d3g0gp89917ko0.cloudfront.net"]
/** @type {CspSources} */
const WIKIDOT_IMAGE_SOURCES = [
  "https://*.wdfiles.com",
  "https://cdn.scpwiki.com",
  "https://scp-wiki-cdn.nyc3.cdn.digitaloceanspaces.com"
]
/** @type {CspSources} */
const WIKIDOT_STYLE_SOURCES = [
  "https://*.wdfiles.com",
  "https://cdn.scpwiki.com",
  "https://cdn.jsdelivr.net",
  "https://d3g0gp89917ko0.cloudfront.net",
  "https://fonts.bunny.net",
  "https://maxcdn.bootstrapcdn.com",
  "https://rsms.me",
  "https://scp-wiki-cdn.nyc3.cdn.digitaloceanspaces.com"
]
/** @type {CspSources} */
const WIKIDOT_FONT_SOURCES = [
  "https://*.wdfiles.com",
  "https://cdn.scpwiki.com",
  "https://cdn.jsdelivr.net",
  "https://fonts.bunny.net",
  "https://maxcdn.bootstrapcdn.com",
  "https://rsms.me",
  "https://scp-wiki-cdn.nyc3.cdn.digitaloceanspaces.com"
]

function isLocalEnvironment() {
  return process.env.FRAMERAIL_ENV === "local" || process.env.NODE_ENV === "development"
}

/** @returns {CspSources} */
function imageSources() {
  /** @type {CspSources} */
  const sources = ["self", "data:", "blob:", ...WIKIDOT_IMAGE_SOURCES]

  if (isLocalEnvironment()) {
    sources.push(...LOCAL_FILE_IMAGE_SOURCES)
    sources.push(...WIKIDOT_LEGACY_IMAGE_SOURCES)
  }

  return sources
}

/** @returns {CspSources} */
function styleSources() {
  /** @type {CspSources} */
  const sources = ["self", "unsafe-inline", ...WIKIDOT_STYLE_SOURCES]

  if (isLocalEnvironment()) {
    sources.push(...LOCAL_FILE_STYLE_SOURCES)
  }

  return sources
}

/** @returns {CspSources} */
function fontSources() {
  return ["self", "data:", ...WIKIDOT_FONT_SOURCES]
}

/** @type {import("@sveltejs/kit").Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: sveltePreprocess(),

  kit: {
    adapter: adapter(),
    csrf: {
      // Allow flexible hosts on local, since we don't have real DNS
      checkOrigin: process.env.FRAMERAIL_ENV !== "local"
    },
    csp: {
      mode: "auto",
      directives: {
        "default-src": ["self"],
        "base-uri": ["self"],
        "object-src": ["none"],
        "frame-ancestors": ["none"],
        "form-action": ["self"],
        "img-src": imageSources(),
        "font-src": fontSources(),
        "style-src": styleSources(),
        "script-src": ["self"],
        "connect-src": ["self"],
        "worker-src": ["self", "blob:"],
        "manifest-src": ["self"]
      }
    },
    alias: {
      "$static": resolve(__dirname, "static"),
      "$assets": resolveAssets()
    }
  }
}

export default config
