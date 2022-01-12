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
import { Terminal } from "./terminal"
import { closing, isBuild, isClean, isServe, pnpm, starting } from "./util"
import { Vite } from "./vite"

// TODO: add shortcut for rebuilding a container, and then restarting it

export class DevCLI {
  declare vite?: Vite
  declare mockoon?: Mockoon
  declare containers?: Containers
  declare terminal?: Terminal

  stopped = false
  starting = false

  async clean() {
    if (this.stopped) return
    this.stopped = true

    await this.containers?.stopLogging()

    this.terminal?.close()

    // try to stop anything that might still be running
    for (const proc of processes) {
      if (!proc.killed) {
        // we're gonna _REALLY_ try to kill it
        // `docker-compose` in particular is a pain
        proc.kill("SIGINT")
        proc.kill("SIGKILL")
      }
    }

    // gonna wait a second if we're still starting up
    if (this.starting) {
      this.starting = false
      await new Promise(resolve => setTimeout(resolve, 500))
      linebreak()
      warn("Aborting startup!")
      linebreak()
    }

    const viteIsRunning = await Vite.isRunning()
    const mockoonIsRunning = await Mockoon.isRunning()
    const containersIsRunning = await Containers.isRunning()

    if (viteIsRunning || mockoonIsRunning || containersIsRunning) {
      info("Cleaning up services...")
      separator()
      if (viteIsRunning) await closing("Vite   ", Vite.clean())
      if (mockoonIsRunning) await closing("Mockoon", Mockoon.clean())
      if (containersIsRunning) await Containers.clean()

      infoline("Cleanup complete.")
    } else {
      info("Nothing to clean up.")
      linebreak()
    }
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

        if (dev.stopped) return dev

        await dev.containers?.startLogging()

        const close = dev.close.bind(dev)
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

    this.terminal?.close()

    if (this.vite || this.mockoon || this.containers) {
      await this.containers?.stopLogging()

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
    this.printURLs()
    linebreak()
  }

  private printURLs() {
    if (this.vite) {
      console.log(`  Vite:     ${pc.cyan(`http://localhost:${pc.bold("3000")}`)}`)
    }
    if (this.mockoon) {
      console.log(`  Mockoon:  ${pc.cyan(`http://localhost:${pc.bold("3500")}`)}`)
    }
    if (this.containers) {
      console.log(`  Postgres: ${pc.cyan(`http://localhost:${pc.bold("5432")}`)}`)
      console.log(`  Deepwell: ${pc.cyan(`http://localhost:${pc.bold("2747")}`)}`)
      console.log(`  Wikijump: ${pc.cyan(`http://www.wikijump.localhost`)}`)
    }
  }

  private async hijackTerminal() {
    if (process.stdin.isTTY) {
      readline.emitKeypressEvents(process.stdin)
      process.stdin.setRawMode(true)

      this.terminal = new Terminal()

      this.terminal.addCommand("vite", async action => {
        if (action === "stop") {
          if (this.vite) await closing("Vite", this.vite.close())
          this.vite = undefined
        } else if (action === "start") {
          if (this.vite) await closing("Vite", this.vite.close())
          this.vite = await starting("Vite", Vite.create())
        } else {
          info("Please specify an action:")
          console.log("  stop")
          console.log("  start")
        }
      })

      this.terminal.addCommand("mockoon", async action => {
        if (action === "stop") {
          if (this.mockoon) await closing("Mockoon", this.mockoon.close())
          this.mockoon = undefined
        } else if (action === "start") {
          if (this.mockoon) await closing("Mockoon", this.mockoon.close())
          this.mockoon = await starting("Mockoon", Mockoon.create())
        } else {
          info("Please specify an action:")
          console.log("  stop")
          console.log("  start")
        }
      })

      this.terminal.addCommand("urls", () => {
        info("URLs:")
        this.printURLs()
      })

      this.terminal.addCommand("quit", async () => {
        await this.close()
        process.exit(0)
      })

      this.terminal.addCommand("containers", async () => {
        const services = await Containers.services()
        info("Containers:")
        services.forEach(service => console.log(`  ${service}`))
      })

      this.terminal.addCommand("restart", async service => {
        const services = await Containers.services()
        if (!service || !services.includes(service)) {
          info("Please specify a container to restart:")
          services.forEach(service => console.log(`  ${service}`))
        } else {
          await this.containers?.stopLogging()
          linebreak()
          await this.containers?.restartService(service)
          linebreak()
          await this.containers?.startLogging()
        }
      })

      this.terminal.addCommand("rebuild", async service => {
        const services = await Containers.services()
        if (!service || !services.includes(service)) {
          info("Please specify a container to rebuild:")
          services.forEach(service => console.log(`  ${service}`))
        } else {
          await this.containers?.stopLogging()
          linebreak()
          await this.containers?.buildService(service)
          linebreak()
          await this.containers?.restartService(service)
          linebreak()
          await this.containers?.startLogging()
        }
      })
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
