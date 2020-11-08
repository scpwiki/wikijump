// This is the interface for the WIKIREQUEST object that appears in the
// WikiLayout template.

declare module "wikirequest" {
  const WIKIREQUEST: {
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
  };
}
