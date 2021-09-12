import { decode, transfer, WorkerModule } from "threads-worker-module"
import type { ContentModuleInterface } from "./content.worker"

async function importWorker() {
  return (await import("./content.worker?worker")).default
}

export class ContentWorker extends WorkerModule<ContentModuleInterface> {
  constructor() {
    super("ftml-lang-content-worker", importWorker, { persist: true })
  }

  /**
   * Extracts the "content" of a string of wikitext, as in a string with
   * all markup removed and replaced with an equivalent number of whitespaces.
   *
   * @param str - The string of wikitext to extract the content of.
   * @param raw - If true, an encoded ArrayBuffer will be returned. This is
   *   useful if the results of this function are to be immediately passed
   *   into another worker.
   */
  async extract(str: string | ArrayBuffer): Promise<string>
  async extract(str: string | ArrayBuffer, raw: true): Promise<ArrayBuffer>
  async extract(str: string | ArrayBuffer, raw = false): Promise<string | ArrayBuffer> {
    const result = await this.invoke("extract", transfer(str))
    return raw ? result : decode(result)
  }

  /**
   * Returns the "stats" for a string of wikitext. This includes its byte
   * count and word count.
   *
   * @param str - The string of wikitext to get the stats of.
   */
  async stats(str: string | ArrayBuffer) {
    return await this.invoke("stats", transfer(str))
  }
}

export default new ContentWorker()
