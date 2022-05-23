import { AbstractWorkerBase } from "@wikijump/comlink"
import type { PrismModule } from "./worker"

export type Prism = typeof globalThis.Prism

export class PrismWorker extends AbstractWorkerBase.of<PrismModule>([
  "disableWorkerMessageHandler",
  "getLanguages",
  "highlight",
  "manual"
]) {
  protected _baseGetWorker() {
    return new Worker(new URL("./worker", import.meta.url), { type: "module" })
  }
}

export default new PrismWorker()
