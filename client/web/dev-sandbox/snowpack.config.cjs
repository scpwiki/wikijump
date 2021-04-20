const sourcemaps = require("rollup-plugin-sourcemaps")

module.exports = {
  workspaceRoot: "../../modules/",
  exclude: ["**/node_modules/**/*", "**/*.d.ts.map"],
  mount: {
    src: "/",
    public: {
      url: "/",
      static: true,
      resolve: false
    },
    "node_modules/ftml-wasm-worker/dist": {
      url: "/static/lib/ftml-wasm-worker",
      static: true
    }
  },
  alias: {
    "ftml-wasm-worker": "/static/lib/ftml-wasm-worker/index.js"
  },
  packageOptions: {
    rollup: {
      plugins: [sourcemaps()]
    },
    polyfillNode: true,
    packageLookupFields: ["svelte"]
  },
  devOptions: {
    open: "none",
    output: "stream"
  },
  buildOptions: {
    out: "dist",
    clean: true,
    metaUrlPath: "static/snowpack",
    sourcemap: true
  },
  // routes: [],
  optimize: {
    // preload: true,
    minify: true,
    treeshake: true,
    splitting: true,
    sourcemap: true,
    manifest: true,
    target: "es2020"
  },
  plugins: ["@snowpack/plugin-svelte"]
}
