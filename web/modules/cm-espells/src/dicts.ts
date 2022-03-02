// organize-imports-ignore

// these are imported as URLs, not full files
// check the vite config, `.dic` and `.aff` are added as "assets"

import dicEN from "dictionary-en/index.dic"
import affEN from "dictionary-en/index.aff"

import dicDE1 from "../vendor/de-transam.dic"
import dicDE2 from "../vendor/de-bjoern.dic"
import dicDE3 from "../vendor/de-chrome.dic"
import affDE from "../vendor/de.aff"

import dicES from "dictionary-es/index.dic"
import affES from "dictionary-es/index.aff"

import dicFR from "dictionary-fr/index.dic"
import affFR from "dictionary-fr/index.aff"

import dicIT from "dictionary-it/index.dic"
import affIT from "dictionary-it/index.aff"

import dicRU from "dictionary-ru/index.dic"
import affRU from "dictionary-ru/index.aff"

import dicKO from "dictionary-ko/index.dic"
import affKO from "dictionary-ko/index.aff"

import dicPL from "dictionary-pl/index.dic"
import affPL from "dictionary-pl/index.aff"

import dicUK from "dictionary-uk/index.dic"
import affUK from "dictionary-uk/index.aff"

import dicPT from "dictionary-pt/index.dic"
import affPT from "dictionary-pt/index.aff"

import dicCS from "dictionary-cs/index.dic"
import affCS from "dictionary-cs/index.aff"

import dicVI from "dictionary-vi/index.dic"
import affVI from "dictionary-vi/index.aff"

import dicEL from "dictionary-el/index.dic"
import affEL from "dictionary-el/index.aff"

import dicTR from "dictionary-tr/index.dic"
import affTR from "dictionary-tr/index.aff"

import dicDA from "dictionary-da/index.dic"
import affDA from "dictionary-da/index.aff"

import dicNB from "dictionary-nb/index.dic"
import affNB from "dictionary-nb/index.aff"

import dicNN from "dictionary-nn/index.dic"
import affNN from "dictionary-nn/index.aff"

import dicSV from "dictionary-sv/index.dic"
import affSV from "dictionary-sv/index.aff"

import dicFO from "dictionary-fo/index.dic"
import affFO from "dictionary-fo/index.aff"

import dicNL from "dictionary-nl/index.dic"
import affNL from "dictionary-nl/index.aff"

import dicHU from "dictionary-hu/index.dic"
import affHU from "dictionary-hu/index.aff"

import dicRO from "dictionary-ro/index.dic"
import affRO from "dictionary-ro/index.aff"

export const DICTIONARIES: Record<string, { aff: string; dic: Arrayable<string> }> = {
  "en": { aff: affEN, dic: dicEN },
  "de": { aff: affDE, dic: [dicDE1, dicDE2, dicDE3] },
  "es": { aff: affES, dic: dicES },
  "fr": { aff: affFR, dic: dicFR },
  "it": { aff: affIT, dic: dicIT },
  "ru": { aff: affRU, dic: dicRU },
  "ko": { aff: affKO, dic: dicKO },
  "pl": { aff: affPL, dic: dicPL },
  "uk": { aff: affUK, dic: dicUK },
  "pt": { aff: affPT, dic: dicPT },
  "cs": { aff: affCS, dic: dicCS },
  "vi": { aff: affVI, dic: dicVI },
  "el": { aff: affEL, dic: dicEL },
  "tr": { aff: affTR, dic: dicTR },
  "da": { aff: affDA, dic: dicDA },
  "nb": { aff: affNB, dic: dicNB },
  "nn": { aff: affNN, dic: dicNN },
  "sv": { aff: affSV, dic: dicSV },
  "fo": { aff: affFO, dic: dicFO },
  "nl": { aff: affNL, dic: dicNL },
  "hu": { aff: affHU, dic: dicHU },
  "ro": { aff: affRO, dic: dicRO }
}

export default DICTIONARIES
