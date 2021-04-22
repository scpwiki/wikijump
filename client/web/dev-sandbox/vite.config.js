/**
 * @type {import('vite').UserConfig}
 */
const config = {
  publicDir: "../public",
  root: "./src",
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    assetsDir: "static/lib",
    manifest: true,
    sourcemap: true,
    target: "esnext",
    minify: "esbuild",
    brotliSize: false
  }
}

export default config
