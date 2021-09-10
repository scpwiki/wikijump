const fs = require("fs")
const vite = require("vite")
const c2k = require("koa-connect")
const { svelte } = require("@sveltejs/vite-plugin-svelte")
const sveltePreprocess = require("svelte-preprocess")
const { default: tsconfigPaths } = require("vite-tsconfig-paths")
const tomlPlugin = require("./scripts/vite-plugin-toml.js")
const yamlPlugin = require("./scripts/vite-plugin-yaml.js")

const ignoredBrowserLogs = ["[vite] connecting...", "[vite] connected."]

/** @type {import("vite").UserConfig} */
const viteConfig = {
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
    tsconfigPaths({ projects: ["./"], loose: true }),
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

// get modules so we can make them into test groups
// this way you can do: pnpm test -- --group wj-util
const groups = fs
  .readdirSync("modules")
  .filter(dir => fs.statSync(`modules/${dir}`).isDirectory())
  .filter(dir => fs.existsSync(`modules/${dir}/tests`))
  .map(module => ({
    name: module,
    files: `modules/${module}/tests/*.ts`
  }))

/** @type {import("@web/test-runner").TestRunnerConfig} */
module.exports = {
  files: ["scripts/import-glob.ts"],
  groups,

  coverageConfig: {
    report: true,
    include: ["modules/*/src/**/*.{js,jsx,ts,tsx}"]
  },

  plugins: [vitePlugin()],

  testRunnerHtml: testFramework => `
    <html>
      <head>
        <script type="module">
          // Note: globals expected by @testing-library/svelte
          window.global = window;
          window.process = { env: {} };
        </script>
        <script type="module" src="${testFramework}"></script>
      </head>
    </html>
  `,

  filterBrowserLogs: ({ args }) => {
    return !args.some(arg => ignoredBrowserLogs.includes(arg))
  }
}

/** @returns {import("@web/test-runner").TestRunnerPlugin} */
function vitePlugin() {
  /** @type {import("vite").ViteDevServer} */
  let server
  return {
    name: "vite-plugin",

    async serverStart({ app }) {
      server = await vite.createServer({
        configFile: false,
        logLevel: "error",
        server: {
          middlewareMode: "ssr",
          hmr: false,
          fs: {
            strict: false
          }
        },
        clearScreen: false,
        ...viteConfig
      })
      app.use(c2k(server.middlewares))
    },

    async serverStop() {
      return server.close()
    }
  }
}
