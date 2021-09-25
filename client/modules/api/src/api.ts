import { Api } from "../vendor/api"

export class WikijumpAPI extends Api<void> {
  // TODO: allow giving a specific site here
  // TODO: temporary baseUrl
  constructor(baseUrl = "localhost:3500") {
    super({ baseUrl })
  }
}
