const sourcemaps = require("rollup-plugin-sourcemaps")

module.exports = {
  workspaceRoot: "../../modules/",
  mount: {
    src: "/",
    public: {
      url: "/",
      static: true,
      resolve: false
    }
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
  // alias: {},
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
