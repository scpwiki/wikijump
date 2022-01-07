import { createServer, type ViteDevServer } from "vite"
import { error } from "../pretty-logs"

export class Vite {
  declare instance: ViteDevServer

  stopped = false

  static async create() {
    const server = await createServer({ base: "/files--dev/" })
    await server.listen()

    // check for config not resolving correctly
    // this property should be defined
    if (server.config?.server?.fs?.strict !== false) {
      error("Vite didn't find its configuration file!")
      await server.close()
      return process.exit()
    }

    const vite = new Vite()
    vite.instance = server
    return vite
  }

  async close() {
    if (this.stopped) return
    this.stopped = true
    await this.instance.close()
  }
}
