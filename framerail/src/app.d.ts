// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
// and what to do when importing types

import type { Layout } from "$lib/types"
import type { Locales } from "./types"

declare global {
  declare namespace App {
    // interface Locals {}
    interface PageData {
      /** Data about the site. */
      site: {
        site_id: number
        created_at: string
        updated_at: string | null
        deleted_at: string | null
        from_wikidot: boolean
        name: string
        slug: string
        tagline: string | null
        locale: string
        default_page: string | null
        preferred_domain: string | null
        layout: Layout
        license: string
        [anySite: any]: unknown
      }
      site_file_domain: string
      license_name: string
      license_url: string
      /** Data about current logged in user. */
      user_session: {
        session: {
          session_token: string
          user_id: number
          created_at: string
          expires_at: string
          ip_address: string
          user_agent: string
          restricted: boolean
          [anySession: any]: unknown
        }
        user: {
          user_id: number
          user_type: string
          created_at: string
          updated_at: string | null
          deleted_at: string | null
          from_wikidot: boolean
          name: string
          slug: string
          name_changes_left: number
          last_name_change_added_at: string | null
          last_renamed_at: string | null
          email: string
          email_verified_at: string | null
          email_validation_info: any | null
          email_validation_at: any | null
          locales: string[]
          avatar_s3_hash: number[]
          real_name: string | null
          gender: string | null
          birthday: string | null
          location: string | null
          biography: string | null
          user_page: string | null
        }
      } | null
      /**
       * Locale fallback list, includes user locale, site locale and
       * browser locale.
       */
      locales: string[]
      /** Data about the page itself. */
      page?: {
        page_id: number
        page_created_at: string
        page_updated_at: string | null
        page_deleted_at: string | null
        page_revision_count: number
        site_id: number
        page_category_id: number
        page_category_slug: string
        discussion_thread_id: number | null
        revision_id: number
        revision_type: any
        revision_created_at: string
        revision_number: number
        revision_user_id: number
        wikitext: string | null
        compiled_html: string | null
        compiled_at: string
        compiled_generator: string
        revision_comments: string
        hidden_fields: string[]
        title: string
        alt_title: string | null
        slug: string
        tags: string[]
        rating: any
        layout: Layout
        [anyPage: any]: unknown
      }
      /** Page options as booleans. */
      options?: {
        edit: boolean
        title: string | null
        parent: string | null
        tags: string | null
        no_redirect: boolean
        no_render: boolean
        debug: boolean
        renderer: boolean
        comments: boolean
        history: boolean
        offset: number | null
        data: string
        /** @deprecated Use `no_render` instead. */
        noRender: boolean
        [anyOptions: any]: unknown
      }
      /** Rendered Wikitext */
      wikitext?: string
      /**
       * Error internationalization as defined in the translation keys for
       * the page. Look at /lib/types.ts for the keys type definitions.
       */
      internationalization?: Locales
      /** Compiled HTML */
      compiled_body_html?: string
      /** Page revision */
      page_revision?: {
        revision_id: number
        revision_type: any
        created_at: string
        updated_at: string
        from_wikidot: boolean
        revision_number: number
        page_id: number
        site_id: number
        user_id: number
        changes: string[]
        wikitext: string | null
        compiled_html: string | null
        compiled_at: string | null
        compiled_generator: string
        comments: string
        hidden: string[]
        title: string | null
        alt_title: string | null
        sliug: string | null
        tags: string[] | null
        [anyPageRevision: any]: unknown
      }
    }

    interface Error extends PageData {
      /** Error message */
      message: string
      /** Error type for page/user/admin view */
      view: string
      /**
       * Error internationalization as defined in the translation keys for
       * the page. Look at /lib/types.ts for the keys type definitions.
       */
      internationalization?: Locales
      /** Compiled HTML */
      compiled_body_html?: string
      [anyError: any]: unknown
    }
    // interface Platform {}
  }
}
