export const ogettext = function (mid: string): string {
  /**
   * Rudimentary localisation function. Searches registered messages for a
   * translations in the OZONE object's language.
   *
   * @param mid: The string to translate.
   */
  return OZONE.loc.getMessage(mid, OZONE.lang);
};

export const loc = {
  // TODO What is loc? Localisation messages?
  messages: {},
  addMessages: function (mlist, lang) {
    /**
     * ??
     *
     * @param mlist: ??
     * @param lang: ??
     */
    if (!OZONE.loc.messages[lang]) {
      OZONE.loc.messages[lang] = {};
    }
    for (const i in mlist) {
      OZONE.loc.messages[lang][i] = mlist[i];
    }
  },
  addMessage: function (mid, mtr, lang) {
    /**
     * ??
     *
     * @param mid: ??
     * @param mtr: ??
     * @param lang: ??
     */
    if (!OZONE.loc.messages[lang]) {
      OZONE.loc.messages[lang] = {};
    }
    OZONE.loc.messages[lang][mid] = mtr;
  },
  getMessage: function (mid, lang) {
    /**
     * ??
     *
     * @param mid: ??
     * @param lang: ??
     */
    if (OZONE.loc.messages[lang]) {
      if (OZONE.loc.messages[lang][mid]) {
        return OZONE.loc.messages[lang][mid];
      }
    }
    // fall back to default
    return mid;
  }
};
