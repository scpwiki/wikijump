import { createMemoryArticleResponseCacheStore } from "./article-response-cache.js"
import { createRedisCacheStore } from "./redis-cache-store.js"

const RESPONSE_STORE_SYMBOL = Symbol.for(
  "wikijump.framerail.article-response-cache-store"
)
const TOKEN_STORE_SYMBOL = Symbol.for("wikijump.framerail.article-response-token-store")

const globalStore = (symbol, createStore) => {
  if (!globalThis[symbol]) {
    globalThis[symbol] = createStore()
  }

  return globalThis[symbol]
}

export const articleResponseCacheStore = globalStore(
  RESPONSE_STORE_SYMBOL,
  createMemoryArticleResponseCacheStore
)

export const articleResponseTokenStore = globalStore(
  TOKEN_STORE_SYMBOL,
  createRedisCacheStore
)
