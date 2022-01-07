import { pnpm } from "./util"

export class Mockoon {
  stopped = false

  static async create() {
    // we don't actually need this function (it's not async)
    // but just to be consistent with the other classes, we'll use this pattern
    const mockoon = new Mockoon()
    pnpm("mock-start", false, "scripts/dev")
    return mockoon
  }

  async close() {
    if (this.stopped) return
    this.stopped = true
    pnpm("mock-stop", false, "scripts/dev")
  }
}
