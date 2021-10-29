const path = require("path")
const { performance } = require("perf_hooks")
const formatMS = require("pretty-ms")
const {
  error,
  info,
  infoline,
  section,
  linebreak,
  question,
  shell,
  cmd
} = require("./pretty-logs")

const DIR = path.resolve(__dirname, "../")
process.chdir(DIR)

const args = process.argv.slice(2)
const isSudo = args.includes("sudo")

const { createServer } = require("vite")

;(async () => {
  const start = performance.now()

  const doBuild = await question("Build containers? This can take a while. [y/N] -> ")
  linebreak()

  if (doBuild.trim().toLowerCase() === "y") {
    section("BUILD")

    if (isSudo) cmd("pnpm build-sudo:local")
    else cmd("pnpm run build:local")

    infoline(
      "Finished building containers.",
      `Took: ${formatMS(performance.now() - start)}`
    )
  } else {
    infoline("Skipping build.")
  }

  section("DEV")
  linebreak()

  const legacyShell = shell("node scripts/build-legacy.js dev")

  const server = await createServer({ base: "/files--dev/" })
  await server.listen()

  // check for config not resolving correctly
  // this property should be defined
  if (server.config?.server?.fs?.strict !== false) {
    error("Vite didn't find its configuration file!")
    process.exit(1)
  }

  infoline(
    "Development servers (Vite, Legacy) started.",
    "Don't forget that you can start a Mockoon server to mock unimplemented API paths."
  )

  section("COMPOSE")

  const composeShell = isSudo ? shell("pnpm compose-sudo up") : shell("pnpm compose up")

  // setup cleanup handlers
  // we do this _past_ starting everything so that an early abort doesn't
  // call these handlers and cause carnage in your terminal

  process.once("cleanup", () => {
    infoline("Cleaning up...")

    if (process.platform === "win32") {
      info(
        "Since you're on Windows, the process might exit before the containers could stop.",
        "Make sure to let the containers finish exiting prior to running 'pnpm dev' again.",
        "If you want to always exit the program without dealing with any prompts,",
        "make sure to HOLD Ctrl+C, rather than tapping it."
      )
      linebreak()
    }

    server.close()
    legacyShell.kill()
    composeShell.kill()
  })

  // do app specific cleaning before exiting
  process.once("beforeExit", () => {
    process.emit("cleanup")
  })

  // catch closing terminal on Windows
  process.once("SIGHUP", () => {
    process.emit("cleanup")
  })

  // catch ctrl+c event
  process.once("SIGINT", () => {
    process.emit("cleanup")
  })

  // catch uncaught exceptions
  process.once("uncaughtException", err => {
    process.emit("cleanup")
    throw new Error(err)
  })
})()
