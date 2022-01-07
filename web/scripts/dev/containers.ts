import type { ChildProcessWithoutNullStreams } from "child_process"
import { formatLogs } from "../pretty-docker-logs"
import { compose, isSudo, pnpm } from "./util"

export class Containers {
  declare logger?: ChildProcessWithoutNullStreams

  stopped = false

  static async create() {
    const containers = new Containers()
    compose("up -d")
    return containers
  }

  async close() {
    if (this.stopped) return
    this.stopped = true
    this.logger?.kill()
    await compose("stop")
  }

  async startLogging() {
    this.logger = await compose("logs -f --tail 10 --no-color", true)
    this.logger.stdout.on("data", data => console.log(formatLogs(data)))
    this.logger.stderr.on("data", data => console.log(formatLogs(data)))
  }

  static async build() {
    if (isSudo) await pnpm("build-sudo:local")
    else await pnpm("build:local")
  }
}
