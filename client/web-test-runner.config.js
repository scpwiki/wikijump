const ignoredBrowserLogs = ["[vite] connecting...", "[vite] connected."]

/** @type {import("@web/test-runner").TestRunnerConfig} */
module.exports = {
  files: ["modules/*/tests/*.ts", "scripts/import-glob.ts"],

  plugins: [vitePlugin()],

  coverageConfig: {
    report: true,
    include: ["modules/*/src/**/*.{js,jsx,ts,tsx}"]
  },

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

const vite = require("vite")
const c2k = require("koa-connect")

/** @returns {import("@web/test-runner").TestRunnerPlugin} */
function vitePlugin() {
  /** @type {import("vite").ViteDevServer} */
  let server
  return {
    name: "vite-plugin",

    async serverStart({ app }) {
      server = await vite.createServer({
        configFile: "./vite.config.js",
        server: { middlewareMode: "ssr" },
        clearScreen: false
      })
      app.use(c2k(server.middlewares))
    },

    async serverStop() {
      return server.close()
    }
  }
}
