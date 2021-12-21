const path = require("path")
const { performance } = require("perf_hooks")
const readline = require("readline")
const formatMS = require("pretty-ms")
const {
  chalk,
  error,
  info,
  infoline,
  section,
  linebreak,
  question,
  cmd,
  shell,
  separator
} = require("./pretty-logs")

const DIR = path.resolve(__dirname, "../")
process.chdir(DIR)

const args = process.argv.slice(2)
const isSudo = args.includes("sudo")

const { createServer } = require("vite")

// -- MAIN

let shuttingDown = false

async function startup() {
  const doBuild = await question("Build containers? This can take a while. [y/N] -> ")

  if (doBuild.trim().toLowerCase() === "y") {
    await buildContainers()
    linebreak()
  } else {
    infoline("Skipping build.")
  }

  section("STARTUP", false, true)

  await startVite()

  linebreak()

  await startContainers()

  infoline(
    "Development servers started.",
    "Don't forget that you can also start a Mockoon server to mock unimplemented API paths."
  )

  section("RUNNING", false, true)

  if (process.stdin.isTTY) {
    readline.emitKeypressEvents(process.stdin)
    process.stdin.setRawMode(true)
  }

  logContainers()
}

async function shutdown() {
  if (shuttingDown) return
  shuttingDown = true

  section("SHUTDOWN", true)

  await stopVite()

  linebreak()

  await stopContainers()

  infoline("Shutdown finished.")
}

// -- VITE LIFECYCLE

let vite
let viteStopping = false

async function startVite() {
  if (vite) return

  vite = await createServer({ base: "/files--dev/" })
  await vite.listen()

  // check for config not resolving correctly
  // this property should be defined
  if (vite.config?.server?.fs?.strict !== false) {
    error("Vite didn't find its configuration file!")
    await vite.close()
    return process.exit()
  }

  console.log(`${chalk.green("Vite started")} (${chalk.blueBright(vite.config.mode)})`)
  vite.printUrls()
}

async function stopVite() {
  if (!vite || viteStopping) return

  vite = null
  viteStopping = true

  // prettier-ignore
  try { await vite.close() } catch {}

  info("Vite stopped.")
}

// -- CONTAINER LIFECYCLE

let containersStarted = false
let containersStopping = false
let containerLogger = null

async function buildContainers() {
  section("BUILD", true)

  const start = performance.now()

  if (isSudo) cmd("pnpm -s build-sudo:local")
  else cmd("pnpm -s build:local")

  infoline(
    "Finished building containers.",
    `Took: ${formatMS(performance.now() - start)}`
  )

  cmd("node scripts/build-legacy.js build")
}

async function startContainers() {
  if (containersStarted) return
  containersStarted = true
  info("Starting containers...")
  separator()
  compose("up -d")
}

async function stopContainers() {
  if (containersStopping) return
  containersStopping = true
  info("Stopping containers...")
  separator()

  if (containerLogger) {
    containerLogger.kill()
    containerLogger = null
  }

  compose("stop")
}

async function logContainers() {
  if (containerLogger) return
  containerLogger = compose("logs -f --tail 10", true)
}

function compose(args, asShell) {
  const str = `pnpm -s compose${isSudo ? "-sudo" : ""} -- ${args}`
  return asShell ? shell(str) : cmd(str)
}

// -- RUN EVERYTHING

// manually handle Ctrl+C
process.stdin.on("keypress", async (str, key) => {
  if (key.name === "c" && key.ctrl) {
    // spamming Ctrl+C again will turn off raw mode,
    // and will try to exit the process again
    if (shuttingDown) {
      process.stdin.setRawMode(false)
    } else {
      await shutdown()
    }
    process.exit(0)
  }
})

// make very sure we stop the damn server
process.once("beforeExit", shutdown)
process.once("SIGINT", shutdown)
process.once("SIGTERM", shutdown)
process.once("SIGHUP", shutdown)

startup()
