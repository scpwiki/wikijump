import { performance } from "perf_hooks"
import formatMS from "pretty-ms"
import readline from "readline"
import {
  answerYesOrNo,
  error,
  info,
  infoline,
  linebreak,
  pc,
  processes,
  question,
  section,
  separator,
  warn
} from "../pretty-logs"
import { Containers } from "./containers"
import { Mockoon } from "./mockoon"
import { closing, isBuild, isClean, isServe, pnpm, starting } from "./util"
import { Vite } from "./vite"

// TODO: add shortcut for rebuilding a container, and then restarting it

export class DevCLI {
  declare vite?: Vite
  declare mockoon: Mockoon
  declare containers: Containers

  stopped = false
  starting = false

  async clean() {
    if (this.stopped) return
    this.stopped = true

    section("CLEAN", true)

    // try to stop anything that might be running
    for (const proc of processes) {
      if (!proc.killed) proc.kill("SIGINT")
    }

    info("Cleaning up services...")
    separator()
    await closing("Vite   ", Vite.clean())
    await closing("Mockoon", Mockoon.clean())
    await Containers.clean()

    infoline("Cleanup complete.")
  }

  static async create() {
    const dev = new DevCLI()

    // skip everything if we're cleaning
    if (isClean) {
      await dev.clean()
      return dev
    }

    const doBuild =
      isBuild ||
      answerYesOrNo(await question("Build containers? This can take a while. [y/N] -> "))

    if (doBuild) await dev.build()

    if (!isBuild) {
      try {
        dev.starting = true

        await dev.hijackTerminal()
        await dev.startup()
        await dev.containers.startLogging()

        const close = dev.close.bind(this)
        process.once("beforeExit", close)
        process.once("SIGINT", close)
        process.once("SIGTERM", close)
        process.once("SIGHUP", close)

        dev.starting = false
      } catch (err) {
        // clean up if the startup failed
        console.error(err)
        if (!dev.stopped) {
          linebreak()
          error("Failed to start! Closing anything that may have started...")
          await dev.clean()
          process.exit(1)
        }
      }
    }

    return dev
  }

  async close() {
    if (this.stopped) return
    this.stopped = true

    if (this.vite || this.mockoon || this.containers) {
      section("SHUTDOWN", true)

      info("Stopping services...")
      separator()

      if (this.vite) await closing("Vite   ", this.vite.close())
      if (this.mockoon) await closing("Mockoon", this.mockoon.close())
      if (this.containers) await this.containers.close()

      infoline("Shutdown complete.")
    }
  }

  private async build() {
    section("BUILD", !isBuild, true)

    const start = performance.now()

    info("Building legacy frontend first...")
    await pnpm("build", true, "web")

    linebreak()

    info("Building containers...")
    separator()
    await Containers.build()

    linebreak()

    info(
      "Finished building containers.",
      `Took: ${pc.cyan(formatMS(performance.now() - start))}`
    )
  }

  private async startup() {
    section("STARTUP", true)

    info("Starting services...")
    separator()

    if (!isServe) this.vite = await starting("Vite   ", Vite.create())
    if (this.stopped) return
    this.mockoon = await starting("Mockoon", Mockoon.create())
    if (this.stopped) return
    this.containers = await Containers.create()
    if (this.stopped) return

    linebreak()
    info("Development services started.")
    if (this.vite) {
      console.log(` > Vite:     ${pc.cyan(`http://localhost:${pc.bold("3000")}`)}`)
    }
    console.log(` > Mockoon:  ${pc.cyan(`http://localhost:${pc.bold("3500")}`)}`)
    console.log(` > Postgres: ${pc.cyan(`http://localhost:${pc.bold("5432")}`)}`)
    console.log(` > Deepwell: ${pc.cyan(`http://localhost:${pc.bold("2747")}`)}`)
    console.log(` > Wikijump: ${pc.cyan(`http://www.wikijump.localhost`)}`)
    linebreak()
  }

  private async hijackTerminal() {
    if (process.stdin.isTTY) {
      readline.emitKeypressEvents(process.stdin)
      process.stdin.setRawMode(true)
    }

    process.stdin.on("keypress", async (_str, key) => {
      // manually handle Ctrl+C
      if (key.name === "c" && key.ctrl) {
        // spamming Ctrl+C again will turn off raw mode,
        // and will try to exit the process again
        if (this.stopped) {
          process.stdin.setRawMode(false)
          error("Ctrl+C pressed twice! Exiting...")
          process.exit(1)
        }
        // aborting startup
        else if (this.starting) {
          linebreak()
          linebreak()
          warn("Aborting startup!")
          this.starting = false
          await this.clean()
          process.exit(0)
        }
        // gracefully close
        else {
          await this.close()
          process.exit(0)
        }
      }
    })
  }
}
