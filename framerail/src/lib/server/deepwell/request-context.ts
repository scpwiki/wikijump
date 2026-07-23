interface RequestContextFields {
  sessionToken?: string
  siteId?: number
  page?: string | number
}

export type RequestContext = RequestContextFields | void
