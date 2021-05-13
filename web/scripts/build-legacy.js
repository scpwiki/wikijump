const path = require("path")
const esbuild = require("esbuild")

const DIR = path.resolve(__dirname, "../")

buildLegacyBundle()

async function buildLegacyBundle() {
  console.log("[legacy] Building legacy bundle...")

  const result = await esbuild.build({
    absWorkingDir: DIR,
    tsconfig: "./tsconfig.legacy.json",
    entryPoints: ["web/files--common/javascript/index.ts"],
    outfile: "web/files--common/dist/bundle.js",
    bundle: true,
    minify: false,
    sourcemap: true,
    splitting: false,
    platform: "browser",
    format: "iife",
    target: "es6"
  })

  console.log("[legacy] Legacy bundle built.")
}
