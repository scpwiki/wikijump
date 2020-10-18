// This is the interface for the WIKIREQUEST object that appears in the
// WikiLayout template.

declare module "wikirequest" {
  interface WikiRequest {
    info: {
      domain: string
      siteId: number
      categoryId: number
      themeId: number
      requestPageName: string
      lang: string
      pageUnixName?: string
      pageId?: number
    }
  }
}
