import { decode, transfer, WorkerModule } from "worker-module"
import type { ContentModuleInterface } from "./worker/content.worker"

async function importWorker() {
  return (await import("./worker/content.worker?bundled-worker")).default
}

export class ContentWorker extends WorkerModule<ContentModuleInterface> {
  constructor() {
    super("ftml-lang-content-worker", importWorker, { persist: true })
  }

  async extract(str: string) {
    return decode(await this.invoke("extract", transfer(str)))
  }

  async stats(str: string) {
    return await this.invoke("stats", transfer(str))
  }
}

export default new ContentWorker()
