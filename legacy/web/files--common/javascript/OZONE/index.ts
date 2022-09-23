import { ajax } from "./ajax";
import { dialog } from "./dialog";
import { dialogs } from "./dialogs";
import { dom } from "./dom";
import { forms } from "./forms";
import { loc, Language } from "./loc";
import { utils } from "./utils";
import { visuals } from "./visuals";

const OZONE = {
  ajax,
  dialog,
  dialogs,
  dom,
  forms,
  loc,
  utils,
  visuals,

  // default language
  lang: 'en' as Language,
  // This is modified during WIKIREQUEST in the layout template
  /* TODO Having both timestamp and date is superfluous - right?
     Server time vs. client time, maybe? If that's the case, be explicit about
     it, at least */
  request: {
    timestamp: 0,
    date: new Date()
  },
  // XXX This was an empty function (unused?)
  init: {}
};

export default OZONE;
