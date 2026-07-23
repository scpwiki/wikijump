import { createMemoryArticleResponseCacheStore } from "./article-response-cache.js"
import { createRedisCacheStore } from "./redis-cache-store.js"

export const createArticleResponseCacheStores = ({
  responseStore = createMemoryArticleResponseCacheStore(),
  tokenStore = createRedisCacheStore()
} = {}) => ({ responseStore, tokenStore })
