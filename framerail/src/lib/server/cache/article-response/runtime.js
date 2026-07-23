import { createArticleResponseCacheStores } from "./stores.js"

const RUNTIME_STORES_SYMBOL = Symbol.for(
  "wikijump.framerail.article-response-cache-runtime"
)
const DEVELOPMENT_STORES_SYMBOL = Symbol.for(
  "wikijump.framerail.article-response-cache-development"
)
const EMPTY_STORES = Object.freeze({ responseStore: null, tokenStore: null })

export const configureArticleResponseCacheStores = (stores) => {
  globalThis[RUNTIME_STORES_SYMBOL] = stores

  return () => {
    if (globalThis[RUNTIME_STORES_SYMBOL] === stores) {
      delete globalThis[RUNTIME_STORES_SYMBOL]
    }
  }
}

const getDevelopmentArticleResponseCacheStores = () => {
  if (process.env.NODE_ENV !== "development") return EMPTY_STORES
  if (!globalThis[DEVELOPMENT_STORES_SYMBOL]) {
    globalThis[DEVELOPMENT_STORES_SYMBOL] = createArticleResponseCacheStores()
  }
  return globalThis[DEVELOPMENT_STORES_SYMBOL]
}

export const getArticleResponseCacheStores = () =>
  globalThis[RUNTIME_STORES_SYMBOL] ?? getDevelopmentArticleResponseCacheStores()

export const resetDevelopmentArticleResponseCacheStores = () => {
  const stores = globalThis[DEVELOPMENT_STORES_SYMBOL]
  stores?.tokenStore?.reset?.()
  delete globalThis[DEVELOPMENT_STORES_SYMBOL]
}
