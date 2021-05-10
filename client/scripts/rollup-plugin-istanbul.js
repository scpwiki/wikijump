const { createBabelInputPluginFactory } = require("@rollup/plugin-babel")
const babelIstanbul = require("babel-plugin-istanbul")

const istanbul = createBabelInputPluginFactory(babel => {
  /** @type import("@rollup/plugin-babel").RollupBabelCustomInputPlugin */
  const plugin = {
    config(cfg, opts) {
      const sourcemap = this.getCombinedSourcemap()
      return {
        ...cfg.options,
        // have to spread the sourcemap because otherwise Babel fails to
        // recognize it as a plain object
        inputSourceMap: { ...sourcemap },
        plugins: [babelIstanbul.default]
      }
    }
  }
  return plugin
})

module.exports = istanbul
