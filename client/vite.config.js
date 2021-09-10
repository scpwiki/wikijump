import { svelte } from "@sveltejs/vite-plugin-svelte"
import sveltePreprocess from "svelte-preprocess"
import tsconfigPaths from "vite-tsconfig-paths"
import tomlPlugin from "./scripts/vite-plugin-toml.js"
import yamlPlugin from "./scripts/vite-plugin-yaml.js"

/** @type {import("vite").UserConfig} */
const config = {
  publicDir: "./public",
  root: "./",
  logLevel: "error",
  server: {
    hmr: false,
    fs: {
      strict: false
    }
  },

  build: {
    assetsDir: "./",
    sourcemap: "inline",
    target: "esnext",
    minify: false,
    brotliSize: false,
    cssCodeSplit: false,

    rollupOptions: {
      plugins: [
        // because esbuild rips out comments, esbuild will not preserve
        // the comments we need for ignoring lines
        // however, it does preserve legal comments, /*! or //!
        // but c8 doesn't recognize those!
        // so we have to transform those back into normal comments
        // before we let c8 parse them
        {
          transform(code, id) {
            // use two spaces so we don't change the length of the document
            code = code.replaceAll("/*! c8", "/*  c8")
            // null map informs rollup to preserve the current sourcemap
            return { code, map: null }
          }
        }
      ],

      treeshake: false
    }
  },

  optimizeDeps: {
    entries: [
      "modules/*/src/**/*.{svelte,js,jsx,ts,tsx}",
      "modules/*/tests/*.{js,jsx,ts,tsx}"
    ],
    include: ["@esm-bundle/chai", "@testing-library/svelte"]
  },

  plugins: [
    tsconfigPaths({
      projects: ["./"],
      loose: true
    }),
    tomlPlugin(),
    yamlPlugin(),
    svelte({
      onwarn: (warning, handler) => {
        if (warning.code === "unused-export-let") return
        if (handler) handler(warning)
      },
      emitCss: false,
      compilerOptions: { cssHash: () => "svelte" },
      preprocess: [sveltePreprocess({ sourceMap: true })]
    })
  ]
}

export default config
