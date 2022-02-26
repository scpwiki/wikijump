import { AbstractWorkerBase } from "@wikijump/comlink"
import type { ContentModule } from "./worker"
import RemoteContentWorker from "./worker?worker"

export class ContentWorker extends AbstractWorkerBase.of<ContentModule>([
  "extractContent",
  "words"
]) {
  protected createWorker() {
    return new RemoteContentWorker()
  }
}

export default new ContentWorker()
