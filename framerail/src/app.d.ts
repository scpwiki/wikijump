// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
// and what to do when importing types

declare namespace App {
  // interface Locals {}
  interface PageData {
    /** Data about the site. */
    site: {
      site_id: string
      [anySite: any]: unknown
    }
    /** Data about the page itself. */
    page: {
      page_id: string
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
      layout: any
      [anyPage: any]: unknown
    }
    /** Page options as booleans. */
    options: {
      edit: boolean
      title: string | null
      parent: string | null
      tags: string | null
      no_redirect: boolean
      no_render: boolean
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
    wikitext: string
    /** Internalization as defined in translation keys for the page. */
    internationalization: any
    /** Compiled HTML */
    compiled_html: string
    /** Page revision */
    page_revision: {
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

  interface Error {
    /** Error message */
    message: string
    /**
     * Error internationalization as defined in the translation keys for
     * the page. Look at /lib/types.ts for the keys type definitions.
     */
    internationalization: any
    [anyError: any]: unknown
  }
  // interface Platform {}
}
