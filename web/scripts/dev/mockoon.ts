import { cmdAsync } from "../pretty-logs"
import { pnpm } from "./util"

export class Mockoon {
  stopped = false

  static async clean() {
    await pnpm("mock-stop", false, "scripts/dev")
  }

  static async create() {
    const mockoon = new Mockoon()
    if (!(await Mockoon.isRunning())) {
      await pnpm("mock-start", false, "scripts/dev")
    }
    return mockoon
  }

  async close() {
    if (this.stopped) return
    this.stopped = true
    await pnpm("mock-stop", false, "scripts/dev")
  }

  static async isRunning() {
    try {
      // have to use `cmdAsync`instead of `pnpm` because the `-s` flag
      // acts bizarre when running a cli tool rather than a script
      const out = await cmdAsync(`cd "scripts/dev" && pnpm mockoon-cli list`, false)
      return !out.includes("No process is running")
    } catch {
      return false
    }
  }
}
