import { strict as assert } from "node:assert"
import test from "node:test"

import {
  configureArticleResponseCacheStores,
  getArticleResponseCacheStores
} from "../src/lib/server/article-response-cache-runtime.js"

test("runtime cache stores have explicit configuration and reset ownership", () => {
  const stores = {
    responseStore: { name: "responses" },
    tokenStore: { name: "tokens" }
  }
  const reset = configureArticleResponseCacheStores(stores)

  assert.equal(getArticleResponseCacheStores(), stores)
  reset()
  assert.deepEqual(getArticleResponseCacheStores(), {
    responseStore: null,
    tokenStore: null
  })
})
