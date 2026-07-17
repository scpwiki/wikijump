import adapter from "@sveltejs/adapter-node"
import { statSync } from "fs"
import { dirname, resolve } from "path"
import { sveltePreprocess } from "svelte-preprocess"
import { fileURLToPath } from "url"
import { parseDeploymentEnvironment } from "./src/lib/server/deployment-environment.js"

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

const CURRENT_SITE_FILE_ORIGIN = "https://wikijump-current-site.invalid"
/** @type {CspSources} */
const WIKIDOT_LEGACY_IMAGE_SOURCES = ["https://d3g0gp89917ko0.cloudfront.net"]
/** @type {CspSources} */
const WIKIDOT_IMAGE_SOURCES = [
  "https://scp-wiki.wikidot.com",
  "https://scp-jp-storage.wikidot.com",
  "https://scpsandboxcn.wikidot.com",
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
  "https://fonts.googleapis.com",
  "https://maxcdn.bootstrapcdn.com",
  "https://nu-scptheme.github.io",
  "https://rsms.me",
  "https://scp-wiki-cdn.nyc3.cdn.digitaloceanspaces.com"
]
/** @type {CspSources} */
const WIKIDOT_FONT_SOURCES = [
  "https://*.wdfiles.com",
  "https://cdn.scpwiki.com",
  "https://cdn.jsdelivr.net",
  "https://fonts.bunny.net",
  "https://fonts.gstatic.com",
  "https://maxcdn.bootstrapcdn.com",
  "https://nu-scptheme.github.io",
  "https://rsms.me",
  "https://scp-wiki-cdn.nyc3.cdn.digitaloceanspaces.com"
]

const deploymentEnvironment = parseDeploymentEnvironment()

/** @returns {CspSources} */
function imageSources() {
  /** @type {CspSources} */
  const sources = [
    "self",
    "data:",
    "blob:",
    CURRENT_SITE_FILE_ORIGIN,
    ...WIKIDOT_IMAGE_SOURCES
  ]

  if (deploymentEnvironment === "local") {
    sources.push(...WIKIDOT_LEGACY_IMAGE_SOURCES)
  }

  return sources
}

/** @returns {CspSources} */
function styleSources() {
  /** @type {CspSources} */
  return ["self", "unsafe-inline", CURRENT_SITE_FILE_ORIGIN, ...WIKIDOT_STYLE_SOURCES]
}

/** @returns {CspSources} */
function fontSources() {
  return ["self", "data:", ...WIKIDOT_FONT_SOURCES]
}

/** @returns {CspSources} */
function frameSources() {
  return ["self"]
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
      checkOrigin: deploymentEnvironment !== "local"
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
        "frame-src": frameSources(),
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
