import { pnpm } from "./util"

export class Mockoon {
  stopped = false

  static async clean() {
    await pnpm("mock-stop", false, "scripts/dev")
  }

  static async create() {
    const mockoon = new Mockoon()
    if (!(await mockoon.isRunning())) {
      await pnpm("mock-start", false, "scripts/dev")
    }
    return mockoon
  }

  async close() {
    if (this.stopped) return
    this.stopped = true
    await pnpm("mock-stop", false, "scripts/dev")
  }

  async isRunning() {
    try {
      const out = await pnpm("pnpm mockoon-cli list", false, "scripts/dev")
      return !out.toString().includes("No process is running")
    } catch {
      return false
    }
  }
}
