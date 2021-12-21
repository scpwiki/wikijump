const path = require("path")
const { performance } = require("perf_hooks")
const readline = require("readline")
const formatMS = require("pretty-ms")
const {
  error,
  infoline,
  section,
  linebreak,
  question,
  cmd,
  shell
} = require("./pretty-logs")

const DIR = path.resolve(__dirname, "../")
process.chdir(DIR)

const args = process.argv.slice(2)
const isSudo = args.includes("sudo")

const { createServer } = require("vite")

let vite

let stopping = false

async function stop() {
  if (stopping) return
  stopping = true

  section("SHUTDOWN", true)

  try {
    if (vite) {
      await vite.close
      vite = null
    }
    compose("stop")
  } catch (err) {
    console.error(err)
    linebreak()
    error(
      "Error stopping server.",
      "Make sure to check if all of the Docker containers were closed."
    )
  }

  infoline("Shutdown finished.")
}

// manually handle Ctrl+C
process.stdin.on("keypress", async (str, key) => {
  if (key.name === "c" && key.ctrl) {
    // spamming Ctrl+C again will turn off raw mode,
    // and will try to exit the process again
    if (stopping) {
      process.stdin.setRawMode(false)
    } else {
      await stop()
    }
    process.exit(0)
  }
})

// make very sure we stop the damn server
process.once("beforeExit", stop)
process.once("SIGINT", stop)
process.once("SIGTERM", stop)
process.once("SIGHUP", stop)

// main function
;(async () => {
  const start = performance.now()

  const doBuild = await question("Build containers? This can take a while. [y/N] -> ")

  if (doBuild.trim().toLowerCase() === "y") {
    section("BUILD", true)

    if (isSudo) cmd("pnpm -s build-sudo:local")
    else cmd("pnpm -s build:local")

    infoline(
      "Finished building containers.",
      `Took: ${formatMS(performance.now() - start)}`
    )

    cmd("node scripts/build-legacy.js build")

    linebreak()
  } else {
    infoline("Skipping build.")
  }

  section("DEV", false, true)

  vite = await createServer({ base: "/files--dev/" })
  await vite.listen()

  // check for config not resolving correctly
  // this property should be defined
  if (vite.config?.server?.fs?.strict !== false) {
    error("Vite didn't find its configuration file!")
    await vite.close()
    return
  }

  infoline(
    "Development server started.",
    "Don't forget that you can also start a Mockoon server to mock unimplemented API paths."
  )

  section("STARTUP", false, true)

  compose("up -d")

  infoline("Containers started.")

  section("RUNNING", false, true)

  if (process.stdin.isTTY) {
    readline.emitKeypressEvents(process.stdin)
    process.stdin.setRawMode(true)
  }

  compose("logs -f --tail 10", true)
})()

function compose(args, asShell) {
  const str = `pnpm -s compose${isSudo ? "-sudo" : ""} -- ${args}`
  return asShell ? shell(str) : cmd(str)
}
