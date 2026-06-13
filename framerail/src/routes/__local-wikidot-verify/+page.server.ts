import {
  createDependencyScenario,
  createListPagesScenario,
  createThemeNavCssScenario,
  exportBundle,
  getHistory,
  getPage,
  getParents,
  getScenarioPages,
  getSite,
  importBundle,
  normalizeSlug,
  normalizeTags,
  previewWarnings,
  renderPreviewPage,
  removeListPagesGamma,
  runProofSummary,
  savePage,
  updateTags,
  uploadPageFile
} from "$lib/server/local-wikidot-verify/lab"
import { fail } from "@sveltejs/kit"

import type { Actions } from "./$types"

export async function load({ url }) {
  const site = await getSite()
  const slug = normalizeSlug(url.searchParams.get("slug") || "ui-authoring-basic")
  const page = await getPage(site.site_id, slug)
  const parents = page ? await getParents(site.site_id, slug) : []
  const history = page ? await getHistory(site.site_id, page.page_id) : []
  const scenarioPages = await getScenarioPages(site.site_id, slug)

  return {
    site,
    lab: {
      selectedSlug: slug,
      page,
      parents,
      history,
      scenarioPages
    },
    internationalization: {
      docs: "Docs",
      "terms-conditions": "Terms",
      privacy: "Privacy",
      security: "Security",
      "footer-powered-by": "Powered by Wikijump"
    }
  }
}

export const actions: Actions = {
  preview: async ({ request }) => {
    try {
      const formData = await request.formData()
      const slug = normalizeSlug(formData.get("slug"))
      const title = String(formData.get("title") ?? "").trim()
      const tags = normalizeTags(formData.get("tags"))
      const parent = normalizeSlug(formData.get("parent"))
      const wikitext = String(formData.get("wikitext") ?? "")
      const preview = await renderPreviewPage({
        slug,
        title: title || slug,
        wikitext,
        tags
      })
      return {
        type: "preview",
        slug,
        title,
        tags,
        parent,
        wikitext,
        preview,
        previewHtml: preview.html,
        warnings: previewWarnings(wikitext)
      }
    } catch (error) {
      return fail(500, {
        type: "preview",
        message: error instanceof Error ? error.message : String(error)
      })
    }
  },

  savePage: async ({ request }) => {
    try {
      const formData = await request.formData()
      const slug = normalizeSlug(formData.get("slug"))
      const title = String(formData.get("title") ?? "").trim()
      const wikitext = String(formData.get("wikitext") ?? "")
      const parent = normalizeSlug(formData.get("parent"))
      if (!slug || !title)
        return fail(400, { type: "savePage", message: "Slug and title are required." })

      const saved = await savePage({
        slug,
        title,
        wikitext,
        tags: normalizeTags(formData.get("tags")),
        parent: parent || undefined
      })

      return {
        type: "savePage",
        slug,
        title,
        tags: normalizeTags(formData.get("tags")),
        parent,
        wikitext,
        saved,
        history: await getHistory(saved.page.site_id, saved.page.page_id)
      }
    } catch (error) {
      return fail(500, {
        type: "savePage",
        message: error instanceof Error ? error.message : String(error)
      })
    }
  },

  updateTags: async ({ request }) => {
    try {
      const formData = await request.formData()
      const slug = normalizeSlug(formData.get("slug"))
      if (!slug) return fail(400, { type: "updateTags", message: "Slug is required." })
      const page = await updateTags(slug, normalizeTags(formData.get("tags")))
      return {
        type: "updateTags",
        slug,
        page,
        parents: await getParents(page.site_id, slug),
        history: await getHistory(page.site_id, page.page_id)
      }
    } catch (error) {
      return fail(500, {
        type: "updateTags",
        message: error instanceof Error ? error.message : String(error)
      })
    }
  },

  uploadFile: async ({ request }) => {
    try {
      const formData = await request.formData()
      const slug = normalizeSlug(formData.get("slug"))
      const file = formData.get("file")
      const name = String(formData.get("name") ?? "").trim()
      if (!slug || !(file instanceof File)) {
        return fail(400, { type: "uploadFile", message: "Slug and file are required." })
      }
      const files = await uploadPageFile(slug, file, name || undefined)
      return { type: "uploadFile", slug, files }
    } catch (error) {
      return fail(500, {
        type: "uploadFile",
        message: error instanceof Error ? error.message : String(error)
      })
    }
  },

  createDependencies: async () => {
    try {
      const dependencies = await createDependencyScenario()
      return { type: "createDependencies", dependencies }
    } catch (error) {
      return fail(500, {
        type: "createDependencies",
        message: error instanceof Error ? error.message : String(error)
      })
    }
  },

  createListPages: async () => {
    try {
      const listPages = await createListPagesScenario()
      return { type: "createListPages", listPages }
    } catch (error) {
      return fail(500, {
        type: "createListPages",
        message: error instanceof Error ? error.message : String(error)
      })
    }
  },

  removeListPagesGamma: async () => {
    try {
      const target = await removeListPagesGamma()
      const site = await getSite()
      const index = await getPage(site.site_id, "ui-authoring-listpages-index")
      return {
        type: "removeListPagesGamma",
        target,
        indexHtml: index?.compiled_body_html ?? ""
      }
    } catch (error) {
      return fail(500, {
        type: "removeListPagesGamma",
        message: error instanceof Error ? error.message : String(error)
      })
    }
  },

  createThemeNavCss: async () => {
    try {
      const themeNavCss = await createThemeNavCssScenario()
      return { type: "createThemeNavCss", themeNavCss }
    } catch (error) {
      return fail(500, {
        type: "createThemeNavCss",
        message: error instanceof Error ? error.message : String(error)
      })
    }
  },

  runProofSummary: async ({ request }) => {
    try {
      const formData = await request.formData()
      const slug = normalizeSlug(formData.get("slug"))
      const proofSummary = await runProofSummary(slug)
      return { type: "runProofSummary", slug, proofSummary }
    } catch (error) {
      return fail(500, {
        type: "runProofSummary",
        message: error instanceof Error ? error.message : String(error)
      })
    }
  },

  exportBundle: async ({ request }) => {
    try {
      const formData = await request.formData()
      const slugs = String(formData.get("slugs") ?? "")
        .split(/[,\s]+/)
        .map((slug) => normalizeSlug(slug))
        .filter(Boolean)
      const bundle = await exportBundle(slugs)
      return {
        type: "exportBundle",
        bundleText: JSON.stringify(bundle, null, 2)
      }
    } catch (error) {
      return fail(500, {
        type: "exportBundle",
        message: error instanceof Error ? error.message : String(error)
      })
    }
  },

  importBundle: async ({ request }) => {
    try {
      const formData = await request.formData()
      const prefix = normalizeSlug(formData.get("prefix")) || "ui-authoring-import-"
      const bundleText = String(formData.get("bundle") ?? "")
      const bundle = JSON.parse(bundleText)
      const imported = await importBundle(bundle, prefix)
      return { type: "importBundle", imported }
    } catch (error) {
      return fail(500, {
        type: "importBundle",
        message: error instanceof Error ? error.message : String(error)
      })
    }
  }
}
