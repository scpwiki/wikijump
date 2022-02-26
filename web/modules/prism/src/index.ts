import { AbstractWorkerBase } from "@wikijump/comlink"
import type { PrismModule } from "./worker"
import PrismRemoteWorker from "./worker?worker"

export class PrismWorker extends AbstractWorkerBase.of<PrismModule>([
  "disableWorkerMessageHandler",
  "getLanguages",
  "highlight",
  "manual"
]) {
  createWorker() {
    return new PrismRemoteWorker()
  }
}

export default new PrismWorker()
