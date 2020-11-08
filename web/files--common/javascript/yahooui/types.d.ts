// This is a quick, temporary declaration for the YAHOO UI library.
// It is not type-safe.

declare const YAHOO: any;

export default YAHOO;

export type YahooResponse = {
  // TODO Work out what else should be here
  status: any
  statusText: string
  responseText: string
  body: string
  mode: string
  locked: boolean
  lock_id: unknown
  lock_secret: unknown
  page_revision_id: unknown
  timeLeft: unknown
  section: null | unknown
  rangeStart: unknown
  rangeEnd: unknown
  key: unknown
  seed: unknown
  thread_id: unknown
  thread_unix_title: unknown
  message: string
}
export type YahooCallback = (response: YahooResponse, arg?: unknown) => void
