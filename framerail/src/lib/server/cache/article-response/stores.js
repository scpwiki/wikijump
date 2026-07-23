import { createMemoryArticleResponseCacheStore } from "./index.js"
import { createRedisCacheStore } from "../redis-store.js"

export const createArticleResponseCacheStores = ({
  responseStore = createMemoryArticleResponseCacheStore(),
  tokenStore = createRedisCacheStore()
} = {}) => ({ responseStore, tokenStore })
