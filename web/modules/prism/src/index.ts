import { AbstractWorkerBase } from "@wikijump/comlink"
import type { PrismModule } from "./worker"
import PrismRemoteWorker from "./worker?worker"

export type Prism = typeof globalThis.Prism

export class PrismWorker extends AbstractWorkerBase.of<PrismModule>([
  "disableWorkerMessageHandler",
  "getLanguages",
  "highlight",
  "manual"
]) {
  protected _baseGetWorker() {
    return new PrismRemoteWorker()
  }
}

export default new PrismWorker()
