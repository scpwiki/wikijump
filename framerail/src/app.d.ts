// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
// and what to do when importing types

import type { PageModel, PageOptions, PageRevisionModel } from "$lib/types"
import type { PreloadData } from "$lib/server/deepwell/views"
import type { RequestContext } from "$lib/server/request-context"
import type { buildWikidotRequestInfo } from "$lib/server/wikidot-request-info"
import type { Locales } from "./types"

declare global {
  declare namespace App {
    // interface Locals {}
    interface PageData extends PreloadData {
      /** Data about the page itself. */
      page?: PageModel
      /** Page options as booleans. */
      options?: PageOptions
      /** Rendered Wikitext */
      wikitext?: string
      /**
       * Error internationalization as defined in the translation keys for
       * the page. Look at /lib/types.ts for the keys type definitions.
       */
      internationalization?: Partial<Locales>
      /** Compiled HTML */
      compiled_body_html?: string
      compiled_body_styles?: string[]
      compiled_top_bar_html?: string | null
      compiled_side_bar_html?: string | null
      /** Page revision */
      page_revision?: PageRevisionModel
    }

    interface Error extends PageData {
      /** Error message, when the error source provides one. */
      message?: string
      /** Error type for page/user/admin view */
      view: string
      /**
       * Error internationalization as defined in the translation keys for
       * the page. Look at /lib/types.ts for the keys type definitions.
       */
      internationalization?: Partial<Locales>
      /** Compiled HTML */
      compiled_body_html?: string
    }
    // interface Platform {}

    interface Locals {
      requestContext: RequestContext
      siteLocale?: string
      wikidotRequestInfo?: ReturnType<typeof buildWikidotRequestInfo>
      anonymousArticleResponseCacheMetadata?: {
        siteId: number
        siteSlug: string
        requestHost: string
        requestLocales: string[]
        backendLocales: string[]
        deepwellArticlePageCacheKey: string
        publicContentFence: string
        permissionFence: string
      }
    }
  }
}
