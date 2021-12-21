const fs = require("fs")
const vite = require("vite")
const c2k = require("koa-connect")
const { TEST_CONFIG } = require("./scripts/vite-utils.js")

const ignoredBrowserLogs = ["[vite] connecting...", "[vite] connected."]

// get modules so we can make them into test groups
// this way you can do: pnpm test -- --group util
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
        ...TEST_CONFIG,
        configFile: false,
        logLevel: "error"
      })
      app.use(c2k(server.middlewares))
    },

    async serverStop() {
      return server.close()
    }
  }
}
