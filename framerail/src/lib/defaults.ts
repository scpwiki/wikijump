const defaults = {
  fallbackLocale: "en",
  translateKeys: {
    // Error
    "error": {},
    "close": {},

    // Footer
    "footer-powered-by": {},
    "terms-conditions": {},
    "privacy": {},
    "docs": {},
    "security": {},
    "footer-license-unless": {},

    // Spinny
    "spinny-label.active": {},
    "spinny-label.error": {},
    "spinny-label.success": {},
    "spinny-label.warning": {},
    "message-loading": {},

    // Form fields
    "field-required": {}
  },
  translateStripKeys: ["footer-license-unless"],
  page: {
    history: {
      revisionNumber: -1,
      limit: 20
    }
  }
}

export default defaults
