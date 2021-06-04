import { decode, transfer, WorkerModule } from "worker-module"
import type { ContentModuleInterface } from "./worker/content.worker"

async function importWorker() {
  return (await import("./worker/content.worker?bundled-worker")).default
}

export class ContentWorker extends WorkerModule<ContentModuleInterface> {
  constructor() {
    super("ftml-lang-content-worker", importWorker, { persist: true })
  }

  async extract(str: string | ArrayBuffer): Promise<string>
  async extract(str: string | ArrayBuffer, raw: true): Promise<ArrayBuffer>
  async extract(str: string | ArrayBuffer, raw = false): Promise<string | ArrayBuffer> {
    const result = await this.invoke("extract", transfer(str))
    return raw ? result : decode(result)
  }

  async stats(str: string | ArrayBuffer) {
    return await this.invoke("stats", transfer(str))
  }
}

export default new ContentWorker()
