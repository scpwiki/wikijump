import OZONE from ".";

export type Language = "en" | "pl"

export const ogettext = function (messageId: string): string {
  /**
   * Rudimentary localisation function. Searches registered messages for a
   * translations in the OZONE object's language.
   *
   * @param messageId: The string to translate.
   */
  return OZONE.loc.getMessage(messageId, OZONE.lang);
};

export const loc = {
  // messages is the internal story of localisation strings
  messages: {} as Record<Language, Record<string, string>>,
  addMessages: function (
    messageList: Record<string, string>,
    lang: Language
  ): void {
    /**
     * Shortcut for storing multiple localisation strings from an object.
     *
     * @param messageList: An object with message ID keys and localisation
     * string values.
     * @param lang: The language of the values.
     */
    // If the language doesn't exist, add it
    if (!(lang in OZONE.loc.messages)) {
      OZONE.loc.messages[lang] = {};
    }
    Object.entries(messageList).forEach(([messageID, messageTranslation]) => {
      OZONE.loc.messages[lang][messageID] = messageTranslation;
    });
  },
  addMessage: function (
    messageId: string,
    messageTranslation: string,
    lang: Language
  ): void {
    /**
     * Store a localisation string.
     *
     * @param messageId: The ID of the localisation string to store.
     * @param messageTranslation: The localisation string.
     * @param lang: The language of the localisation string.
     */
    if (!(lang in OZONE.loc.messages)) {
      OZONE.loc.messages[lang] = {};
    }
    OZONE.loc.messages[lang][messageId] = messageTranslation;
  },
  getMessage: function (messageId: string, lang: Language): string {
    /**
     * Retrieve a localisation string.
     *
     * @param messageId: The ID of the localisation string to retrieve.
     * @param lang: The language of the wanted localisation string.
     */
    // Use the message ID as default if the lang/message doesn't exist
    return OZONE.loc.messages[lang]?.[messageId] ?? messageId;
  }
};
