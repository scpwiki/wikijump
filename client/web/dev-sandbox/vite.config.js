/**
 * @type {import('vite').UserConfig}
 */
const config = {
  publicDir: "../public",
  root: "./src",
  resolve: {
    dedupe: ["@codemirror/state"]
  },
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    assetsDir: "static/assets",
    manifest: true,
    sourcemap: true,
    target: "esnext",
    minify: "esbuild",
    brotliSize: false
  }
}

export default config
