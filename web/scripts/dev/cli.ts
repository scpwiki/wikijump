import { performance } from "perf_hooks"
import formatMS from "pretty-ms"
import readline from "readline"
import {
  answerYesOrNo,
  info,
  infoline,
  linebreak,
  pc,
  question,
  section,
  separator
} from "../pretty-logs"
import { Containers } from "./containers"
import { Mockoon } from "./mockoon"
import { closing, isBuild, isServe, pnpm, starting } from "./util"
import { Vite } from "./vite"

// TODO: add shortcut for rebuilding a container, and then restarting it
// TODO: check for Mockoon and Vite already running
// TODO: check for if the containers are already running

export class DevCLI {
  declare vite?: Vite
  declare mockoon: Mockoon
  declare containers: Containers

  stopped = false

  constructor() {
    const close = this.close.bind(this)
    // make very sure we stop the damn server
    process.once("beforeExit", close)
    process.once("SIGINT", close)
    process.once("SIGTERM", close)
    process.once("SIGHUP", close)
  }

  static async create() {
    const dev = new DevCLI()

    linebreak()

    const doBuild =
      isBuild ||
      answerYesOrNo(await question("Build containers? This can take a while. [y/N] -> "))

    if (doBuild) await dev.build()

    if (!isBuild) {
      await dev.startup()
      await dev.hijackTerminal()
      dev.containers.startLogging()
    }

    return dev
  }

  async close() {
    if (this.stopped) return
    this.stopped = true

    section("SHUTDOWN", true)

    info("Stopping non-container services...")
    separator()

    if (this.vite) await closing("Vite   ", this.vite.close())
    await closing("Mockoon", this.mockoon.close())

    linebreak()

    info("Stopping containers...")
    separator()
    await this.containers.close()

    infoline("Shutdown complete.")
  }

  private async build() {
    section("BUILD", true)

    const start = performance.now()

    info("Building legacy frontend first...")
    pnpm("build:legacy")

    linebreak()

    info("Building containers...")
    separator()
    Containers.build()

    infoline(
      "Finished building containers.",
      `Took: ${formatMS(performance.now() - start)}`
    )
  }

  private async startup() {
    section("STARTUP", true)

    info("Starting non-container services...")
    separator()

    if (!isServe) this.vite = await starting("Vite   ", Vite.create())

    this.mockoon = await starting("Mockoon", Mockoon.create())

    linebreak()

    info("Started:")
    if (this.vite) this.vite.instance.printUrls()
    console.log(`  > Mockoon:   ${pc.cyan(`http://localhost:${pc.bold("3500")}`)}`)

    linebreak()

    info("Starting containers...")
    separator()
    this.containers = await Containers.create()

    infoline("Development servers started.")
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
        } else {
          await this.close()
        }
        process.exit(0)
      }
    })
  }
}
